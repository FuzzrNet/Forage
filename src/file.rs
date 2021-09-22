use std::{env::current_dir, fs::File, path::PathBuf, time::Instant};

use anyhow::Result;
use chrono::DateTime;
use log::info;
use walkdir::WalkDir;

use crate::{
    config::get_data_dir,
    db::{insert_file, upsert_path, FileInfo, USR_CONFIG},
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
pub async fn upload_path(prefix: String, cwd: PathBuf) -> Result<()> {
    let start = Instant::now();
    let files = walk_dir(&get_data_dir().await?, prefix)?;
    let files_len = files.len();
    let mut bytes = 0;

    for file in files {
        let blake3_hash = hash_file(&file, &mut USR_CONFIG.file_salt.to_owned())?;

        let EncodedFileInfo {
            bao_hash,
            read: size,
            written,
        } = encode(&file, &blake3_hash.to_hex().to_string()).await?;

        let parent_rev = upsert_path(file.to_str().unwrap(), blake3_hash.as_bytes())?;
        let mime_type = infer_mime_type(&file)?;
        let metadata = File::open(&file)?.metadata()?;

        let file_info = FileInfo {
            blake3_hash,
            bao_hash,
            size,
            path: file,
            parent_rev,
            mime_type,
            date_created: DateTime::from(metadata.created()?),
            date_modified: DateTime::from(metadata.modified()?),
            date_accessed: DateTime::from(metadata.accessed()?),
        };

        insert_file(file_info).await?;

        bytes += written;
    }

    info!(
        "{} files with added in {:.2?}. {} bytes written.",
        files_len,
        start.elapsed(),
        bytes
    );

    Ok(())
}
