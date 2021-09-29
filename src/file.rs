use std::{env::current_dir, fs::File, path::PathBuf, time::Instant};

use anyhow::Result;
use chrono::DateTime;
use human_bytes::human_bytes;
use log::info;
use walkdir::WalkDir;

use crate::{
    db::{
        contains_hash, flush_kv, get_max_slice_index, insert_file, insert_hash, mark_as_dropped,
        remove_hash, upsert_path, FileInfo, USR_CONFIG,
    },
    hash::{encode, hash_file, infer_mime_type, EncodedFileInfo},
};

pub struct Offset(u64);

impl Offset {
    pub fn new(offset: u64) -> Self {
        assert_eq!(offset % 1024, 0);
        Self(offset / 1024)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

pub fn walk_dir(path: &PathBuf, prefix: String) -> Result<Vec<PathBuf>> {
    let start = Instant::now();
    let mut paths = vec![];
    let cwd = current_dir()?.to_string_lossy().to_string();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let entry_path = entry.into_path();

            if entry_path
                .to_string_lossy()
                .to_string()
                .replace(&cwd, "")
                .starts_with(&prefix)
            {
                paths.push(entry_path);
            }
        }
    }

    info!("{} files found in {:.2?}", paths.len(), start.elapsed());

    Ok(paths)
}

/// Uploads all files under a path to storage channels.
pub async fn upload_path(prefix: String, data_dir: PathBuf) -> Result<()> {
    let start = Instant::now();
    let files = walk_dir(&data_dir, prefix)?;
    let files_len = files.len();
    let mut bytes_read = 0;
    let mut bytes_written = 0;

    for file in files {
        let blake3_hash = hash_file(&file, &mut USR_CONFIG.file_salt.to_owned())?;
        let blake3_bytes = blake3_hash.as_bytes();

        if contains_hash(blake3_bytes)? {
            continue;
        } else {
            insert_hash(blake3_bytes)?;
        }

        let EncodedFileInfo {
            bao_hash,
            read,
            written,
        } = encode(&file, &blake3_hash.to_hex().to_string()).await?;

        let parent_rev = upsert_path(&file.to_string_lossy(), blake3_bytes)?;
        let mime_type = infer_mime_type(&file)?;
        let metadata = File::open(&file)?.metadata()?;

        // Relative path to Forest Data dir
        let path = file.strip_prefix(&data_dir)?.to_path_buf();

        let min_slice = get_max_slice_index().await?;
        let max_slice = 0; // TODO: compute actual max slice

        let file_info = FileInfo {
            blake3_hash,
            bao_hash,
            bytes_read: read,
            bytes_written: written,
            min_slice,
            max_slice,
            path,
            parent_rev,
            mime_type,
            date_created: DateTime::from(metadata.created()?),
            date_modified: DateTime::from(metadata.modified()?),
            date_accessed: DateTime::from(metadata.accessed()?),
            dropped: false,
            removed: false,
        };

        insert_file(file_info).await?;

        if let Some(parent_hash) = parent_rev {
            mark_as_dropped(parent_hash).await?;
        }

        bytes_read += read;
        bytes_written += written;
    }

    flush_kv()?;

    info!(
        "{} bytes read. {} files processed in {:.2?}. {} bytes written.",
        human_bytes(bytes_read as f64),
        files_len,
        start.elapsed(),
        human_bytes(bytes_written as f64),
    );

    if bytes_read > 0 {
        info!(
            "Write amplification was {:.2}%.",
            ((bytes_written as f64 / bytes_read as f64) - 1.0) * 100.0
        );
    }

    Ok(())
}

/// Verify oldest file, newest file, and three random files inbetween
pub async fn verify_data() -> Result<(String, u64, u64)> {
    todo!();
}

/// Fully delete a file from both storage client and storage provider, instead of just dropping it from the storage client
pub async fn delete_file(hash: blake3::Hash) -> Result<()> {
    remove_hash(hash)?;
    todo!();
}
