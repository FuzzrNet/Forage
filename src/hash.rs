use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::Path,
};

use anyhow::Error;
use bao::{
    encode::{Encoder, SliceExtractor},
    Hash,
};
use log::{debug, error};
use rand::Rng;

pub fn encode(path: &Path) -> Result<Hash, Error> {
    let mut file = File::open(path)?;
    let mut file_buf = vec![0u8; 16384];

    let encoded_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/forage_test")?;
    let mut encoder = Encoder::new(&encoded_file);

    loop {
        let n = file.read(&mut file_buf)?;

        if n == 0 {
            debug!("done reading");
            break;
        }

        debug!("bytes read: {}", n);

        encoder.write_all(&file_buf)?;
    }

    let hash = encoder.finalize()?;

    Ok(hash)
}

pub fn decode_hash(hash_hex: &str) -> Result<Hash, Error> {
    let hash_bytes = hex::decode(hash_hex)?;
    let hash_array: [u8; blake3::OUT_LEN] = hash_bytes[..].try_into()?;
    Ok(hash_array.into())
}

const SLICE_LEN: u64 = 4096;

pub fn check(hash_hex: &str) -> Result<(), Error> {
    // Client
    let encoded_file = OpenOptions::new().read(true).open("/tmp/forage_test")?;
    let range = encoded_file.metadata()?.len() / 1024;
    let mut rng = rand::thread_rng();
    let slice_start = rng.gen_range(0..range);

    // Provider
    let mut extractor = SliceExtractor::new(encoded_file, slice_start, SLICE_LEN);
    let mut slice = vec![];
    extractor.read_to_end(&mut slice)?;

    // Client
    let hash = decode_hash(hash_hex)?;
    let mut decoder = bao::decode::SliceDecoder::new(&*slice, &hash, slice_start, SLICE_LEN);
    match decoder.read_to_end(&mut Vec::new()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    const FORAGE_HASH: &str = "34948c5602139c92c1f75b405544e181bf381470e1ff2796cfa729122663b288";

    #[test]
    fn test_encode_and_check() -> Result<(), Error> {
        let path = Path::new("forage.jpg");
        let hash = encode(path)?;
        assert_eq!(hash.to_hex().as_str(), FORAGE_HASH);
        check(FORAGE_HASH)?;
        Ok(())
    }
}
