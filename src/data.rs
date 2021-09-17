use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{error, info};
use once_cell::sync::Lazy;
use rusqlite::{named_params, Connection};
use sled::{Config, Db, Mode};
use tokio::sync::Mutex;

use crate::{
    config::CONFIG,
    file::{walk_dir, Offset},
    hash::{encode, hash_file, infer_mime_type, ivec_to_blake3_hash},
};

/// Databases

/// Sled keystore
const PATHS_TREE: &str = "paths:";

static DB_KV: Lazy<Arc<Db>> = Lazy::new(|| {
    Arc::new(
        Config::default()
            .path(CONFIG.config_dir.join("sled_kv"))
            .mode(Mode::LowSpace) // Since this uses Tor, disk IO will not be a bottleneck
            .use_compression(true)
            .compression_factor(19)
            .open()
            .unwrap_or_else(|e| {
                error!(
                    "Trouble opening Sled keystore: {}. Using a temporary in-memory database.",
                    e
                );
                Config::default().temporary(true).open().unwrap()
            }),
    )
});

/// SQLite datastore
static DB_SQL: Lazy<Arc<Mutex<Connection>>> = Lazy::new(|| {
    let conn = Connection::open(CONFIG.config_dir.join("sqlite").join("forage.db3"))
        .unwrap_or_else(|e| {
            error!(
                "Trouble opening SQLite database: {}. Using a temporary in-memory database.",
                e
            );
            Connection::open_in_memory().unwrap()
        });

    conn.execute_batch(
        "
        BEGIN;
        CREATE TABLE file (
            blake3_hash         CHARACTER(64) PRIMARY KEY
            bao_hash            CHARACTER(64) NOT NULL
            offset              INT NOT NULL
            len                 INT NOT NULL
            size                BIGINT NOT NULL
            cwd                 TEXT NOT NULL
            absolute_path       TEXT NOT NULL
            parent_rev          CHARACTER(64)
            mime_type           VARCHAR(255) NOT NULL
            date_created        DATETIME NOT NULL
            date_modified       DATETIME NOT NULL
            date_accessed       DATETIME NOT NULL
        );
        CREATE TABLE peer (
            tor_v3              TEXT NOT NULL
            label               TEXT
            date_created        DATETIME NOT NULL
            client              BOOLEAN NOT NULL
            provider            BOOLEAN NOT NULL
            market              BOOLEAN NOT NULL
        );
        COMMIT;
    ",
    )
    .unwrap();

    Arc::new(Mutex::new(conn))
});

pub struct FileInfo {
    blake3_hash: blake3::Hash, // Primary key
    bao_hash: bao::Hash,
    offset: Offset, // Forage account data offset format
    len: u32,       // 1KB chunks
    size: u64,      // bytes on disk
    cwd: PathBuf,
    absolute_path: PathBuf,
    parent_rev: Option<blake3::Hash>,
    mime_type: String,
    date_created: DateTime<Utc>,
    date_modified: DateTime<Utc>,
    date_accessed: DateTime<Utc>,
}

pub async fn add_file(file: FileInfo) -> Result<()> {
    let blake3_hash: String = file.blake3_hash.to_hex().to_string();
    let bao_hash: String = file.bao_hash.to_hex().to_string();
    let offset: u32 = file.offset.span();
    let len: u32 = file.len;
    let size: u64 = file.size;
    let cwd: String = file.cwd.to_str().unwrap().to_owned();
    let absolute_path: String = file.absolute_path.to_str().unwrap().to_owned();
    let parent_rev: Option<String> = match file.parent_rev {
        Some(rev) => Some(rev.to_hex().to_string()),
        None => None,
    };
    let mime_type: String = file.mime_type;
    let date_created: i64 = file.date_created.timestamp_millis();
    let date_modified: i64 = file.date_modified.timestamp_millis();
    let date_accessed: i64 = file.date_accessed.timestamp_millis();

    let conn = DB_SQL.lock().await;

    let mut stmt = conn.prepare_cached(
        "INSERT INTO files (
        blake3_hash,
        bao_hash,
        offset,
        len,
        size,
        cwd,
        absolute_path,
        parent_rev,
        mime_type,
        date_created,
        date_modified,
        date_accessed,
    ) VALUES (
        :blake3_hash,
        :bao_hash,
        :offset,
        :len,
        :size,
        :cwd,
        :absolute_path,
        :parent_rev,
        :mime_type,
        :date_created,
        :date_modified,
        :date_accessed,
    )",
    )?;

    stmt.execute(named_params! {
        ":blake3_hash": blake3_hash,
        ":bao_hash": bao_hash,
        ":offset": offset,
        ":len": len,
        ":size": size,
        ":cwd": cwd,
        ":absolute_path": absolute_path,
        ":parent_rev": parent_rev,
        ":mime_type": mime_type,
        ":date_created": date_created,
        ":date_modified": date_modified,
        ":date_accessed": date_accessed,
    })?;

    Ok(())
}

/// Adds all files under a path.
pub async fn add_path(path: &Path, cwd: PathBuf) -> Result<()> {
    let start = Instant::now();
    let files = walk_dir(path);
    let files_len = files.len();
    let mut bytes = 0;

    for file in files {
        let blake3_hash = hash_file(&file)?;
        let (bao_hash, size, len, offset) = encode(&file)?;

        let parent_rev = DB_KV
            .open_tree(PATHS_TREE)?
            .insert(file.to_str().unwrap(), blake3_hash.as_bytes())?
            .map(|v| ivec_to_blake3_hash(v).unwrap());

        let mime_type = infer_mime_type(&file)?;

        let metadata = File::open(&file)?.metadata()?;

        let file = FileInfo {
            blake3_hash,
            bao_hash,
            offset: Offset::new(offset),
            len: (len / 1024) as u32,
            size: size as u64,
            cwd: cwd.to_owned(),
            absolute_path: file,
            parent_rev,
            mime_type,
            date_created: DateTime::from(metadata.created()?),
            date_modified: DateTime::from(metadata.modified()?),
            date_accessed: DateTime::from(metadata.accessed()?),
        };

        add_file(file).await?;

        bytes += len;
    }

    info!(
        "{} files with added in {:.2?}. {} bytes written.",
        files_len,
        start.elapsed(),
        bytes
    );

    Ok(())
}
