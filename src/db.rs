use std::{cmp, convert::TryInto, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use human_bytes::human_bytes;
use log::error;
use once_cell::sync::Lazy;
use rand::RngCore;
use rusqlite::{named_params, Connection};
use sled::{Config, Db, IVec, Mode};
use tokio::sync::Mutex;

use crate::{
    config::ENV_CFG,
    hash::{parse_bao_hash, parse_blake3_hash},
};

/// # Databases

/// ## Sled keystore

/// ### Trees / Keys
const USR_CONFIG_TREE: &str = "usr_cfg:";
const USR_CONFIG_FILE_SALT: &str = "file_salt";

const PATHS_TREE: &str = "paths:";
const HASH_TREE: &str = "hash:";

static DB_KV: Lazy<Arc<Db>> = Lazy::new(|| {
    Arc::new(
        Config::default()
            .path(ENV_CFG.forage_cfg_dir.join("sled_kv"))
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

/// ## SQLite datastore

/// ### Creates schemas, keeps a connection
static DB_SQL: Lazy<Arc<Mutex<Connection>>> = Lazy::new(|| {
    let conn = Connection::open(ENV_CFG.forage_cfg_dir.join("sqlite_db").join("forage.db3"))
        .unwrap_or_else(|e| {
            error!(
                "Trouble opening SQLite database: {}. Using a temporary in-memory database.",
                e
            );
            Connection::open_in_memory().unwrap()
        });

    conn.execute_batch(
        "   BEGIN;
                CREATE TABLE IF NOT EXISTS files (
                    blake3_hash         CHARACTER(64) PRIMARY KEY,
                    bao_hash            CHARACTER(64) NOT NULL,
                    size                BIGINT NOT NULL,
                    path                TEXT NOT NULL,
                    parent_rev          CHARACTER(64),
                    mime_type           VARCHAR(255) NOT NULL,
                    date_created        DATETIME NOT NULL,
                    date_modified       DATETIME NOT NULL,
                    date_accessed       DATETIME NOT NULL
                );
                CREATE TABLE IF NOT EXISTS peer (
                    tor_v3              TEXT NOT NULL,
                    label               TEXT,
                    date_created        DATETIME NOT NULL,
                    client              BOOLEAN NOT NULL,
                    provider            BOOLEAN NOT NULL
                );
                CREATE UNIQUE INDEX IF NOT EXISTS idx_file_blake3_hash ON files (blake3_hash);
                CREATE UNIQUE INDEX IF NOT EXISTS idx_peer_tor_v3 ON peer (tor_v3);
                COMMIT;",
    )
    .unwrap();

    Arc::new(Mutex::new(conn))
});

/// ## Persisted User Config
pub struct UsrCfg {
    pub file_salt: Vec<u8>,
}

/// ### Salt for file hashes is generated and then persisted so data can be de-duplicated without revealing its original hash
fn init_usr_cfg() -> Result<UsrCfg> {
    let file_salt = match DB_KV
        .open_tree(USR_CONFIG_TREE)?
        .get(USR_CONFIG_FILE_SALT)?
    {
        Some(fs) => fs.to_vec(),
        None => {
            let mut rng = rand::thread_rng();
            let mut file_salt = vec![0; 32];
            rng.fill_bytes(&mut file_salt);

            DB_KV
                .open_tree(USR_CONFIG_TREE)?
                .insert(USR_CONFIG_FILE_SALT, file_salt.as_slice())?;

            file_salt
        }
    };

    Ok(UsrCfg { file_salt })
}

pub static USR_CONFIG: Lazy<UsrCfg> = Lazy::new(|| init_usr_cfg().unwrap());

/// # Queries

/// ## Files

/// ### File Info struct
pub struct FileInfo {
    pub blake3_hash: blake3::Hash, // Primary key
    pub bao_hash: bao::Hash,
    pub size: u64, // bytes on disk
    pub path: PathBuf,
    pub parent_rev: Option<blake3::Hash>,
    pub mime_type: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub date_accessed: DateTime<Utc>,
}

/// ### Adds a file to SQL DB
pub async fn insert_file(file: FileInfo) -> Result<()> {
    let blake3_hash: String = file.blake3_hash.to_hex().to_string();
    let bao_hash: String = file.bao_hash.to_hex().to_string();
    let size: u64 = file.size;
    let path: String = file.path.to_str().unwrap().to_owned();
    let parent_rev: Option<String> = file.parent_rev.map(|rev| rev.to_hex().to_string());
    let mime_type: String = file.mime_type;
    let date_created: i64 = file.date_created.timestamp_millis();
    let date_modified: i64 = file.date_modified.timestamp_millis();
    let date_accessed: i64 = file.date_accessed.timestamp_millis();

    let conn = DB_SQL.lock().await;

    let mut stmt = conn.prepare_cached(
        "INSERT INTO files (
        blake3_hash,
        bao_hash,
        size,
        path,
        parent_rev,
        mime_type,
        date_created,
        date_modified,
        date_accessed
    ) VALUES (
        :blake3_hash,
        :bao_hash,
        :size,
        :path,
        :parent_rev,
        :mime_type,
        :date_created,
        :date_modified,
        :date_accessed
    )",
    )?;

    stmt.execute(named_params! {
        ":blake3_hash": blake3_hash,
        ":bao_hash": bao_hash,
        ":size": size,
        ":path": path,
        ":parent_rev": parent_rev,
        ":mime_type": mime_type,
        ":date_created": date_created,
        ":date_modified": date_modified,
        ":date_accessed": date_accessed,
    })?;

    Ok(())
}

pub fn ivec_to_blake3_hash(hash_bytes: IVec) -> Result<blake3::Hash> {
    let hash_array: [u8; blake3::OUT_LEN] = hash_bytes[..].try_into()?;
    Ok(hash_array.into())
}

pub fn upsert_path(file_path: &str, hash_bytes: &[u8]) -> Result<Option<blake3::Hash>> {
    Ok(DB_KV
        .open_tree(PATHS_TREE)?
        .insert(file_path, hash_bytes)?
        .map(|v| ivec_to_blake3_hash(v).unwrap()))
}

pub fn insert_hash(hash_bytes: &[u8]) -> Result<()> {
    DB_KV
        .open_tree(HASH_TREE)?
        .insert(hash_bytes, IVec::default())?;
    Ok(())
}

pub fn contains_hash(hash_bytes: &[u8]) -> Result<bool> {
    Ok(DB_KV.open_tree(HASH_TREE)?.contains_key(hash_bytes)?)
}

pub fn flush_kv() -> Result<()> {
    DB_KV.flush()?;
    Ok(())
}

pub async fn get_files() -> Result<Vec<FileInfo>> {
    let conn = DB_SQL.lock().await;
    let mut stmt = conn.prepare_cached("SELECT * FROM files")?;

    let results = stmt.query_map([], |row| {
        let blake3_hash: String = row.get(0)?;
        let bao_hash: String = row.get(1)?;
        let size = row.get(2)?;
        let path: String = row.get(3)?;
        let parent_rev: Option<String> = row.get(4)?;
        let mime_type = row.get(5)?;
        let date_created: i64 = row.get(6)?;
        let date_modified: i64 = row.get(7)?;
        let date_accessed: i64 = row.get(8)?;

        let blake3_hash = parse_blake3_hash(&blake3_hash).unwrap();
        let bao_hash = parse_bao_hash(&bao_hash).unwrap();
        let path = PathBuf::from_str(&path).unwrap();
        let parent_rev = parent_rev.map(|pr| parse_blake3_hash(&pr).unwrap());
        let date_created = DateTime::from_utc(NaiveDateTime::from_timestamp(date_created, 0), Utc);
        let date_modified =
            DateTime::from_utc(NaiveDateTime::from_timestamp(date_modified, 0), Utc);
        let date_accessed =
            DateTime::from_utc(NaiveDateTime::from_timestamp(date_accessed, 0), Utc);

        Ok(FileInfo {
            blake3_hash,
            bao_hash,
            size,
            path,
            parent_rev,
            mime_type,
            date_created,
            date_modified,
            date_accessed,
        })
    })?;

    Ok(results.map(|res_fi| res_fi.unwrap()).collect())
}

pub async fn list_files() -> Result<Vec<String>> {
    let max_path_len = get_files().await?.iter().fold(0, |acc, info| {
        cmp::max(acc, info.path.to_string_lossy().len())
    });

    Ok(get_files()
        .await?
        .iter()
        .map(|info| {
            let path = info.path.to_string_lossy();

            format!(
                "{size}\t\t{mime_type}\t{path}{path_space}",
                size = human_bytes(info.size as f64),
                mime_type = info.mime_type,
                path = path,
                path_space = " ".repeat(max_path_len - path.len()),
            )
        })
        .collect())
}
