use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use bao::{
    decode::{Decoder, SliceDecoder},
    encode::{encoded_size, Encoder, SliceExtractor},
};
use human_bytes::human_bytes;
use log::{debug, error};

use crate::config::get_storage_path;

pub struct EncodedFileInfo {
    pub bao_hash: bao::Hash,
    pub read: u64,
    pub written: u64,
}

/// Encode a file by its path using bao encoding.
/// Returns bao hash, bytes read, bytes written, and the offset from which the bytes were written.
pub async fn encode(path: &Path, hash_hex: &str) -> Result<EncodedFileInfo> {
    let mut file = File::open(path)?;

    // Eventually this will need to be moved into a different function and replaced with a network call
    let encoded_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(get_storage_path().await?.join(hash_hex))?;

    let mut encoder = Encoder::new(&encoded_file);
    let read = copy_reader_to_writer(&mut file, &mut encoder)?;

    // Generate filler bytes for remainder of 1024 byte slice
    let len = 1024 - read % 1024;
    let buf = vec![0u8; len];
    let written = encoded_size((read + len) as u64) as u64;

    encoder.write_all(&buf)?;
    encoder.flush()?;

    let bao_hash = encoder.finalize()?;

    Ok(EncodedFileInfo {
        bao_hash,
        read: read as u64,
        written,
    })
}

const SLICE_LEN: u64 = 1024;

pub async fn verify(
    bao_hash: &bao::Hash,
    encoded_file_path: &PathBuf,
    slice_index: u64,
) -> Result<()> {
    // Client
    let encoded_file = File::open(encoded_file_path)?;

    // Provider
    let mut extractor = SliceExtractor::new(encoded_file, slice_index * SLICE_LEN, SLICE_LEN);
    let mut slice = vec![];
    extractor.read_to_end(&mut slice)?;

    // Client
    let mut decoder = SliceDecoder::new(&*slice, bao_hash, slice_index * SLICE_LEN, SLICE_LEN);

    let mut decoded = vec![];
    match decoder.read_to_end(&mut decoded) {
        Ok(_) => Ok(()),
        Err(err) => match err.kind() {
            ErrorKind::InvalidData => {
                error!("Invalid data for hash: {}", bao_hash.to_hex());
                Err(err.into())
            }
            _ => {
                error!("Unexpected error: {}", err);
                Err(err.into())
            }
        },
    }
}

pub async fn extract(
    out: &Path,
    bao_hash: &bao::Hash,
    blake3_hash: &str,
    file_size: u64,
) -> Result<usize> {
    let encoded_file = File::open(get_storage_path().await?.join(blake3_hash))?;
    let mut extracted_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true) // Warning! Will overwrite data TODO: Add a check
        .open(out)?;

    let extractor = SliceExtractor::new(encoded_file, 0, file_size);
    let mut decoder = Decoder::new(extractor, bao_hash);
    let bytes_read = copy_reader_to_writer(&mut decoder, &mut extracted_file)?;

    debug!("bytes written: {}", human_bytes(bytes_read as f64));

    Ok(bytes_read)
}

pub fn parse_bao_hash(hash_hex: &str) -> Result<bao::Hash> {
    let hash_bytes = hex::decode(hash_hex)?;
    let hash_array: [u8; bao::HASH_SIZE] = hash_bytes[..].try_into()?;
    Ok(hash_array.into())
}

pub fn parse_blake3_hash(hash_hex: &str) -> Result<blake3::Hash> {
    let hash_bytes = hex::decode(hash_hex)?;
    let hash_array: [u8; blake3::OUT_LEN] = hash_bytes[..].try_into()?;
    Ok(hash_array.into())
}

// TODO: Make this use file streaming w/ hash digest
// TODO: Also, make this use blake3 keyed hash instead of "salt"
pub fn hash_file(path: &Path, salt: &mut Vec<u8>) -> Result<blake3::Hash> {
    let mut contents = vec![];
    File::open(path)?.read_to_end(&mut contents)?;
    contents.append(salt);
    let file_hash = blake3::hash(&contents);

    debug!("path: {}, size: {}", path.to_string_lossy(), contents.len(),);

    Ok(file_hash)
}

pub fn infer_mime_type(path: &Path) -> Result<String> {
    let mime_type = infer::get_from_path(path)?
        .map_or("application/octet-stream", |t| t.mime_type())
        .to_owned();

    debug!("path: {}, type: {}", path.to_string_lossy(), mime_type);

    Ok(mime_type)
}

fn copy_reader_to_writer(reader: &mut impl Read, writer: &mut impl Write) -> Result<usize> {
    // At least 16 KiB is necessary to use AVX-512 with BLAKE3.
    let mut buf = [0; 65536];
    let mut read = 0;

    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(read),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };

        read += len;

        writer.write_all(&buf[..len])?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::prelude::MetadataExt;

    const BLAKE3_HASH: &str = "bce2e13684b952c97d76484689ca2da88abe251820f7dcb4bec5b4b3a218e3b3";
    const BAO_HASH: &str = "2bfebb57a5acf7348f7ef7338c1083e7454027373bec8693bc7b4beb206458f8";
    const SALT: &str = "d970c0e931dc490a842e04f4e9daa8e5e55d9875f53327e4ecc5e3280e7122ed";

    #[tokio::test]
    async fn integration() -> Result<()> {
        let mut salt1 = hex::decode(SALT)?;
        let mut salt2 = hex::decode(SALT)?;

        let orig_path = Path::new("forage.jpg");
        let blake3_hash = hash_file(orig_path, &mut salt1)?.to_hex();
        assert_eq!(
            &blake3_hash, BLAKE3_HASH,
            "test file matches hardcoded blake3 hash"
        );

        let EncodedFileInfo {
            bao_hash,
            read,
            written,
        } = encode(orig_path, &blake3_hash).await?;

        let storage_path = get_storage_path().await?;
        let encoded_file_path = storage_path.join(blake3_hash.as_str());
        let bytes_on_disk = File::open(&encoded_file_path)?.metadata()?.size();

        assert_eq!(read, 81155, "bytes read from original file");
        assert_eq!(written, 86984, "bytes computed to be written to disk");
        assert_eq!(
            bytes_on_disk, 86984,
            "actual file size must match computed size"
        );
        assert_eq!(bao_hash.to_hex().as_str(), BAO_HASH, "bao hash must match");

        verify(&bao_hash, &encoded_file_path, 5).await?;

        let out_path = Path::new("/tmp/forage.jpg");
        extract(out_path, &bao_hash, &blake3_hash, read).await?;

        let decoded_bytes_on_disk = File::open(&encoded_file_path)?.metadata()?.size();
        assert_eq!(
            decoded_bytes_on_disk, 81155,
            "decoded file matches original length"
        );

        assert_eq!(
            hash_file(out_path, &mut salt2)?.to_hex().as_str(),
            BLAKE3_HASH,
            "extracted file matches original file blake3 hash"
        );

        Ok(())
    }
}
