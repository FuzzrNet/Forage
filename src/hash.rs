use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    os::unix::prelude::MetadataExt,
    path::Path,
};

use anyhow::Result;
use bao::{
    decode::SliceDecoder,
    encode::{Encoder, SliceExtractor},
};
use log::{debug, error};
use rand::Rng;

use crate::config::get_storage_path;

pub struct EncodedFileInfo {
    pub bao_hash: bao::Hash,
    pub read: u64,
    pub written: u64,
    pub offset: u64,
}

/// Encode a file by its path using bao encoding.
/// Returns bao hash, bytes read, bytes written, and the offset from which the bytes were written.
pub async fn encode(path: &Path, hash_hex: &str) -> Result<EncodedFileInfo> {
    let mut file = File::open(path)?;

    // Eventually this will need to be moved into a different function and replaced with a network call
    let mut encoded_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(get_storage_path().await?.join(hash_hex))?;

    encoded_file.seek(SeekFrom::End(0))?;
    let offset = encoded_file.stream_position()?;
    let mut encoder = Encoder::new(&encoded_file);

    let read = copy_reader_to_writer(&mut file, &mut encoder)? as u64;
    let bao_hash = encoder.finalize()?;
    let written = encoded_file.metadata()?.size() - offset;

    Ok(EncodedFileInfo {
        bao_hash,
        read,
        written,
        offset,
    })
}

const SLICE_LEN: u64 = 1024;

pub async fn verify(bao_hash: &bao::Hash, blake3_hash: &str) -> Result<()> {
    // Client
    // TODO: data::get_file
    let encoded_file = File::open(get_storage_path().await?.join(blake3_hash))?;
    let range = encoded_file.metadata()?.len() / SLICE_LEN;
    let mut rng = rand::thread_rng();
    let slice_start = rng.gen_range(0..range);

    // Provider
    let mut extractor = SliceExtractor::new(encoded_file, slice_start, SLICE_LEN);
    let mut slice = vec![];
    extractor.read_to_end(&mut slice)?;

    // Client
    let mut decoder = SliceDecoder::new(&*slice, bao_hash, slice_start, SLICE_LEN);

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

pub async fn extract(out: &Path, bao_hash: &bao::Hash, blake3_hash: &str) -> Result<usize> {
    let encoded_file = File::open(get_storage_path().await?.join(blake3_hash))?;
    let mut extracted_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true) // Warning! Will overwrite data TODO: Add a check
        .open(out)?;

    let encoded_len = encoded_file.metadata()?.len();

    let extractor = SliceExtractor::new(encoded_file, 0, encoded_len);
    let mut decoder = SliceDecoder::new(extractor, bao_hash, 0, encoded_len);
    let bytes_read = copy_reader_to_writer(&mut decoder, &mut extracted_file)?;

    debug!("bytes written: {}", bytes_read);

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
pub fn hash_file(path: &Path, salt: &mut Vec<u8>) -> Result<blake3::Hash> {
    let mut contents = vec![];
    File::open(path)?.read_to_end(&mut contents)?;
    contents.append(salt);
    let file_hash = blake3::hash(&contents);
    debug!("path: {}, size: {}", path.to_str().unwrap(), contents.len(),);
    Ok(file_hash)
}

pub fn infer_mime_type(path: &Path) -> Result<String> {
    let mime_type = infer::get_from_path(path)?
        .map_or("application/octet-stream", |t| t.mime_type())
        .to_owned();

    debug!("path: {}, type: {}", path.to_str().unwrap(), mime_type);

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

    const BLAKE3_HASH: &str = "bce2e13684b952c97d76484689ca2da88abe251820f7dcb4bec5b4b3a218e3b3";
    const BAO_HASH: &str = "621aa075e15290f8730e9a1a09e5aa07a7ba5fd7ab3e0980258538ff751a8010";
    const SALT: &str = "d970c0e931dc490a842e04f4e9daa8e5e55d9875f53327e4ecc5e3280e7122ed";

    #[tokio::test]
    async fn integration() -> Result<()> {
        let mut salt1 = hex::decode(SALT)?;
        let mut salt2 = hex::decode(SALT)?;

        let orig_path = Path::new("forage.jpg");
        let blake3_hash = hash_file(orig_path, &mut salt1)?.to_hex();
        assert_eq!(&blake3_hash, BLAKE3_HASH);

        let EncodedFileInfo {
            bao_hash,
            read,
            written,
            offset,
        } = encode(orig_path, &blake3_hash).await?;

        assert_eq!(read, 81155);
        assert_eq!(written, 86219);
        assert_eq!(offset, 0);
        assert_eq!(bao_hash.to_hex().as_str(), BAO_HASH);

        verify(&bao_hash, &blake3_hash).await?;

        let out_path = Path::new("/tmp/forage.jpg");
        extract(out_path, &bao_hash, &blake3_hash).await?;

        assert_eq!(
            hash_file(out_path, &mut salt2)?.to_hex().as_str(),
            BLAKE3_HASH
        );

        Ok(())
    }
}
