use anyhow::Result;
use log::{error, info, warn};
use tokio::signal;

pub mod config;
pub mod db;
pub mod file;
pub mod hash;
pub mod net;
pub mod schema;

pub fn new_client(label: &str, cap: Option<u64>) {
    info!(
        "Creating a new channel for {} with a cap of {:?}",
        label, cap
    );
    warn!("Not yet implemented");
    todo!();
}

pub fn open_channel(address: &str) {
    info!("Opening a channel to {}", address);
    warn!("Not yet implemented");
    todo!();
}

pub async fn upload(prefix: &str) -> Result<()> {
    info!("Storing data in Forage Data directory over available storage channels...");
    let data_dir = config::get_data_dir().await?;
    file::upload_path(prefix, &data_dir).await?;

    Ok(())
}

pub async fn download(prefix: &str) -> Result<()> {
    info!("Retrieving unsynced files over available storage channels...");

    let data_dir = config::get_data_dir().await?;

    // Check paths of existing files in the Forage Data dir
    // If a file is absent, extract it to its relative path
    let updated = file::download_by_prefix(prefix, &data_dir).await?;

    info!(
        "{} files in {}/{} updated.",
        updated.len(),
        data_dir.to_string_lossy(),
        prefix
    );

    // TODO: Changed / dropped file handling:

    // TODO: If hashes differ, add the new revision and drop the old file

    // TODO: Get dropped files

    // TODO: If dropped hashes differ from any previous revision, add the new revision, otherwise, remove it

    Ok(())
}

pub async fn verify() -> Result<()> {
    info!("Verifying data possession on existing storage channels...");

    let slice_count = db::get_max_slice().await?;

    if slice_count == 0 {
        info!("No slices to verify. Try adding some files.");
    } else {
        let db::SliceIndexInfo {
            blake3_hash,
            bao_hash,
            file_slice_index: slice_index,
            data_dir_path,
        } = db::get_random_slice_index(slice_count).await?;

        let bao_hash = hash::parse_bao_hash(&bao_hash)?;
        let encoded_path = config::get_storage_path().await?.join(blake3_hash);
        info!(
            "File chosen: {}\tIndex: {} of {} slices",
            data_dir_path, slice_index, slice_count
        );

        match hash::verify(&bao_hash, &encoded_path, slice_index).await {
            Ok(()) => {
                info!("Verification successful.");
            }
            Err(e) => {
                error!("Verification unsuccessful.\tError: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn list_files(_prefix: &str, _depth: usize) -> Result<()> {
    // TODO: support prefix and depth parameters
    let data_dir = config::get_data_dir().await?;
    let files = db::list_files().await?;
    info!(
        "{} files stored in {}:\n{}",
        files.len(),
        data_dir.file_name().unwrap().to_string_lossy(),
        files.join("\n")
    );

    Ok(())
}

pub async fn start() -> Result<()> {
    info!("Starting Forage node...");
    signal::ctrl_c().await?;

    Ok(())
}

pub fn status() {
    info!("Status from Forage node... Press CTRL-C to stop");
    warn!("Not yet implemented");
    todo!();
}
