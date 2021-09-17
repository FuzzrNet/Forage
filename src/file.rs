use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use log::info;
use walkdir::WalkDir;

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

/// 32-bit Forage account data offset format
/// 24 bits for sectors (16777216 sectors * 1KB per sector = 16GB)
/// 8 bits for pages (256 pages * 16GB per page = max 4TB per onion v3 address)
pub struct Offset(u32);

impl Offset {
    pub fn new(offset: u64) -> Self {
        assert_eq!(offset % 1024, 0);
        Self((offset / 1024) as u32)
    }

    #[allow(dead_code)] // TODO: volume layout
    pub fn sector(&self) -> u32 {
        let s = self.0.to_le_bytes();
        u32::from_le_bytes([0, s[1], s[2], s[3]])
    }

    #[allow(dead_code)] // TODO: volume layout
    pub fn page(&self) -> u8 {
        u8::from_le_bytes([self.0.to_le_bytes()[0]])
    }

    pub fn span(&self) -> u32 {
        self.0
    }
}
