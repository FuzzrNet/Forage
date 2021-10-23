use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::Path,
};

use anyhow::Result;
use bao::{
    decode::{Decoder, SliceDecoder},
    encode::{encoded_size, Encoder, SliceExtractor},
};
use blake3::Hasher;
use human_bytes::human_bytes;
use log::{debug, error};
use tokio::fs::create_dir_all;

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
    let read = copy_reader_to_writer(&mut file, &mut encoder, 0)?;

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
    encoded_file_path: &Path,
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

    if let Some(parent_dir) = out.to_path_buf().parent() {
        // Will probably error if a file exists where a directory should be... TODO: Handle this case gracefully
        if !Path::new(parent_dir).exists() {
            create_dir_all(parent_dir).await?;
        }
    }

    let mut extracted_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true) // Warning! Will overwrite data
        .open(out)?;

    let extractor = SliceExtractor::new(encoded_file, 0, file_size);
    let mut decoder = Decoder::new(extractor, bao_hash);
    let bytes_read = copy_reader_to_writer(&mut decoder, &mut extracted_file, file_size as usize)?;

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
pub fn hash_file(path: &Path, hash_key: &[u8; 32]) -> Result<blake3::Hash> {
    let mut file_reader = File::open(path)?;
    let mut hasher = Hasher::new_keyed(hash_key);
    let bytes_read = copy_reader_to_writer(&mut file_reader, &mut hasher, 0)?;
    let file_hash = hasher.finalize();
    debug!("path: {}, size: {}", path.to_string_lossy(), bytes_read);
    Ok(file_hash)
}

pub fn infer_mime_type(path: &Path) -> Result<String> {
    let mime_type = infer::get_from_path(path)?
        .map_or("application/octet-stream", |t| t.mime_type())
        .to_owned();

    debug!("path: {}, type: {}", path.to_string_lossy(), mime_type);

    Ok(mime_type)
}

// Limit of 0 means there's no limit
fn copy_reader_to_writer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    limit: usize,
) -> Result<usize> {
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

        if limit != 0 && read + len > limit {
            writer.write_all(&buf[..limit - read])?;
            return Ok(limit);
        } else {
            writer.write_all(&buf[..len])?;
        }

        read += len;
    }
}
