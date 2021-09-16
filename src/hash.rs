use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::Path,
};

use anyhow::Error;
use bao::{
    decode::SliceDecoder,
    encode::{Encoder, SliceExtractor},
    Hash,
};
use log::{debug, error};
use rand::Rng;

#[allow(dead_code)]
pub fn encode(path: &Path) -> Result<(Hash, usize), Error> {
    let mut file = File::open(path)?;
    let mut file_buf = vec![0u8; 16384];

    let encoded_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/forage_test")?;

    let mut encoder = Encoder::new(&encoded_file);
    let mut len = 0;

    loop {
        let bytes_read = file.read(&mut file_buf)?;

        if bytes_read == 0 {
            debug!("done reading");
            break;
        }

        debug!("bytes read: {}", bytes_read);
        len += bytes_read;

        encoder.write_all(&file_buf)?;
    }

    let hash = encoder.finalize()?;

    Ok((hash, len))
}

const SLICE_LEN: u64 = 4096;

#[allow(dead_code)]
pub fn verify(hash_hex: &str) -> Result<(), Error> {
    // Client
    let encoded_file = File::open("/tmp/forage_test")?;
    let range = encoded_file.metadata()?.len() / 1024;
    let mut rng = rand::thread_rng();
    let slice_start = rng.gen_range(0..range);

    // Provider
    let mut extractor = SliceExtractor::new(encoded_file, slice_start, SLICE_LEN);
    let mut slice = vec![];
    extractor.read_to_end(&mut slice)?;

    // Client
    let hash = parse_hash(hash_hex)?;
    let mut decoder = SliceDecoder::new(&*slice, &hash, slice_start, SLICE_LEN);

    let mut decoded = vec![];
    match decoder.read_to_end(&mut decoded) {
        Ok(_) => Ok(()),
        Err(err) => match err.kind() {
            ErrorKind::InvalidData => {
                error!("Invalid data for hash: {}", hash.to_hex());
                Err(err.into())
            }
            _ => {
                error!("Unexpected error: {}", err);
                Err(err.into())
            }
        },
    }
}

#[allow(dead_code)]
pub fn extract(out: &Path, hash: &Hash, offset: u64, orig_len: usize) -> Result<usize, Error> {
    let encoded_file = File::open("/tmp/forage_test")?;
    let mut extracted_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out)?;

    let encoded_len = encoded_file.metadata()?.len();

    let extractor = SliceExtractor::new(encoded_file, offset, encoded_len);
    let mut decoder = SliceDecoder::new(extractor, hash, offset, encoded_len);
    let bytes_written = copy_reader_to_writer(&mut decoder, &mut extracted_file, orig_len)?;

    debug!("bytes written: {}", bytes_written);

    Ok(bytes_written)
}

#[allow(dead_code)]
pub fn parse_hash(hash_hex: &str) -> Result<Hash, Error> {
    let hash_bytes = hex::decode(hash_hex)?;
    let hash_array: [u8; bao::HASH_SIZE] = hash_bytes[..].try_into()?;
    Ok(hash_array.into())
}

#[allow(dead_code)]
pub fn hash_file(path: &Path) -> Result<Hash, Error> {
    let mut contents = vec![];
    File::open(path)?.read_to_end(&mut contents)?;
    println!("path: {}, len: {}", path.to_str().unwrap(), contents.len());
    let file_hash = blake3::hash(&contents);
    Ok(file_hash)
}

#[allow(dead_code)]
fn copy_reader_to_writer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    orig_len: usize,
) -> Result<usize, Error> {
    // At least 16 KiB is necessary to use AVX-512 with BLAKE3.
    let mut buf = [0; 65536];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };

        // Truncate output if we're about to exceed the original length
        if written + len > orig_len {
            writer.write_all(&buf[..orig_len - written])?;
            return Ok(written);
        } else {
            writer.write_all(&buf[..len])?;
            written += len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAO_HASH: &str = "34948c5602139c92c1f75b405544e181bf381470e1ff2796cfa729122663b288";
    const FILE_HASH: &str = "621aa075e15290f8730e9a1a09e5aa07a7ba5fd7ab3e0980258538ff751a8010";

    #[test]
    fn integration() -> Result<(), Error> {
        let orig_path = Path::new("forage.jpg");
        assert_eq!(hash_file(orig_path)?.to_hex().as_str(), FILE_HASH);

        let (bao_hash, orig_len) = encode(orig_path)?;
        assert_eq!(bao_hash.to_hex().as_str(), BAO_HASH);

        verify(BAO_HASH)?;

        let out_path = Path::new("/tmp/forage.jpg");
        extract(out_path, &bao_hash, 0, orig_len)?;
        assert_eq!(hash_file(out_path)?.to_hex().as_str(), FILE_HASH);

        Ok(())
    }
}
