use std::{fs::File, os::unix::prelude::MetadataExt, path::Path};

use anyhow::Result;
use serial_test::serial;

const BLAKE3_HASH: &str = "42da460c6136a30d7e41d8437fca41483e4d8a3c202433b5aa5244acf4c192ef";
const BAO_HASH: &str = "2bfebb57a5acf7348f7ef7338c1083e7454027373bec8693bc7b4beb206458f8";
const HASH_KEY: &str = "8036656ceb7d0d35306d7b7737a4d3e56b4ce18d1f02733effda0958e05c2782";

#[tokio::test]
#[serial]
async fn hash() -> Result<()> {
    use forage::{
        config::get_storage_path,
        hash::{encode, extract, hash_file, verify, EncodedFileInfo},
    };

    let mut hash_key: [u8; 32] = Default::default();
    hash_key.copy_from_slice(&hex::decode(HASH_KEY)?);

    let orig_path = Path::new("forage.jpg");
    let blake3_hash = hash_file(orig_path, &hash_key)?.to_hex();
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

    let decoded_bytes_on_disk = File::open(&out_path)?.metadata()?.size();
    assert_eq!(
        decoded_bytes_on_disk, 81155,
        "decoded file matches original length"
    );

    assert_eq!(
        hash_file(out_path, &hash_key)?.to_hex().as_str(),
        BLAKE3_HASH,
        "extracted file matches original file blake3 hash"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn fresh_install() {
    use forage::{download, list_files, upload, verify};

    upload("").await.expect("uploaded");
    verify().await.expect("verified");
    download("").await.expect("downloaded");
    list_files("", 0).await.expect("listed");
}
