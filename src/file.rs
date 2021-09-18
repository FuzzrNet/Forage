use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;
use chrono::DateTime;
use log::info;
use walkdir::WalkDir;

use crate::{
    db::{add_file, get_parent_rev, FileInfo, USR_CONFIG},
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

pub fn walk_dir(path: &Path) -> Vec<PathBuf> {
    let start = Instant::now();
    let mut paths = vec![];

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            paths.push(entry.into_path());
        }
    }

    info!("{} files found in {:.2?}", paths.len(), start.elapsed());

    paths
}

/// Adds all files under a path.
pub async fn process_path(path: &Path, cwd: PathBuf) -> Result<()> {
    let start = Instant::now();
    let files = walk_dir(path);
    let files_len = files.len();
    let mut bytes = 0;

    for file in files {
        let blake3_hash = hash_file(&file, &mut USR_CONFIG.file_salt.to_owned())?;

        let EncodedFileInfo {
            bao_hash,
            read: size,
            written,
            offset,
        } = encode(&file, &blake3_hash.to_hex().to_string()).await?;

        let parent_rev = get_parent_rev(file.to_str().unwrap(), blake3_hash.as_bytes())?;
        let mime_type = infer_mime_type(&file)?;
        let metadata = File::open(&file)?.metadata()?;

        let file = FileInfo {
            blake3_hash,
            bao_hash,
            offset: Offset::new(offset),
            size,
            cwd: cwd.to_owned(),
            absolute_path: file,
            parent_rev,
            mime_type,
            date_created: DateTime::from(metadata.created()?),
            date_modified: DateTime::from(metadata.modified()?),
            date_accessed: DateTime::from(metadata.accessed()?),
        };

        add_file(file).await?;

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
