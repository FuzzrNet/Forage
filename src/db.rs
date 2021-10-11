#![allow(dead_code)]
use std::{cmp, collections::HashSet, convert::TryInto, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use human_bytes::human_bytes;
use log::error;
use once_cell::sync::Lazy;
use rand::{Rng, RngCore};
use rusqlite::{named_params, params, Connection, OptionalExtension};
use sled::{Config, Db, IVec, Mode};
use tokio::sync::Mutex;

use crate::{
    config::ENV_CFG,
    hash::{parse_bao_hash, parse_blake3_hash},
};

const HASH_KEY_CONTEXT: &str = "Forage Storage User Hash Key";

/// # Databases

/// ## Sled keystore

/// ### Trees / Keys
const USR_CFG_TREE: &str = "usr_cfg";
const USR_CFG_HASH_KEY: &str = "hash_key";

const PATHS_TREE: &str = "paths";
const HASH_TREE: &str = "hash";

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
                    bytes_read          BIGINT NOT NULL,
                    bytes_written       BIGINT NOT NULL,
                    min_slice           BIGINT NOT NULL,
                    max_slice           BIGINT NOT NULL,
                    path                TEXT NOT NULL,
                    parent_rev          CHARACTER(64),
                    mime_type           VARCHAR(255) NOT NULL,
                    date_created        DATETIME NOT NULL,
                    date_modified       DATETIME NOT NULL,
                    date_accessed       DATETIME NOT NULL,
                    dropped             BOOLEAN NOT NULL,
                    removed             BOOLEAN NOT NULL
                );
                CREATE TABLE IF NOT EXISTS peers (
                    tor_v3              TEXT NOT NULL,
                    label               TEXT,
                    date_created        DATETIME NOT NULL,
                    client              BOOLEAN NOT NULL,
                    provider            BOOLEAN NOT NULL
                );
                CREATE UNIQUE INDEX IF NOT EXISTS idx_file_blake3_hash ON files (blake3_hash);
                CREATE UNIQUE INDEX IF NOT EXISTS idx_peer_tor_v3 ON peers (tor_v3);
                COMMIT;",
    )
    .unwrap();

    Arc::new(Mutex::new(conn))
});

/// ## Persisted User Config
pub struct UsrCfg {
    pub hash_key: [u8; 32],
}

fn fix_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut fixed_arr: [u8; N] = [0; N];
    fixed_arr.copy_from_slice(slice);
    fixed_arr
}

/// ### Hash key for keyed hashes is generated and then persisted so data can be de-duplicated deterministically without revealing the original hash
fn init_usr_cfg() -> Result<UsrCfg> {
    let hash_key: [u8; 32] = match DB_KV.open_tree(USR_CFG_TREE)?.get(USR_CFG_HASH_KEY)? {
        Some(fs) => fix_slice::<32>(&fs),
        None => {
            let mut rng = rand::thread_rng();
            let mut key_material = vec![0; 32];
            rng.fill_bytes(&mut key_material);

            let hash_key = blake3::derive_key(HASH_KEY_CONTEXT, &key_material);

            DB_KV
                .open_tree(USR_CFG_TREE)?
                .insert(USR_CFG_HASH_KEY, IVec::from(&hash_key))?;

            hash_key
        }
    };

    Ok(UsrCfg { hash_key })
}

pub static USR_CONFIG: Lazy<UsrCfg> = Lazy::new(|| init_usr_cfg().unwrap());

/// # Queries

/// ## Files

/// ### File Info struct
pub struct FileInfo {
    pub blake3_hash: blake3::Hash, // Primary key
    pub bao_hash: bao::Hash,
    pub bytes_read: u64,    // original bytes on disk
    pub bytes_written: u64, // bao-encoded bytes on disk
    pub min_slice: u64,     // starting slice index
    pub max_slice: u64,     // ending slice index
    pub path: PathBuf,
    pub parent_rev: Option<blake3::Hash>,
    pub mime_type: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub date_accessed: DateTime<Utc>,
    pub dropped: bool, // Dropped from storage client
    pub removed: bool, // Removed from storage provider (but still tracked for verification)
}

/// ### Adds a file to SQL DB
pub async fn insert_file(file: FileInfo) -> Result<()> {
    let blake3_hash: String = file.blake3_hash.to_hex().to_string();
    let bao_hash: String = file.bao_hash.to_hex().to_string();
    let bytes_read: u64 = file.bytes_read;
    let bytes_written: u64 = file.bytes_written;
    let min_slice: u64 = file.min_slice;
    let max_slice: u64 = file.max_slice;
    let path: String = file.path.to_str().unwrap().to_owned();
    let parent_rev: Option<String> = file.parent_rev.map(|rev| rev.to_hex().to_string());
    let mime_type: String = file.mime_type;
    let date_created: i64 = file.date_created.timestamp_millis();
    let date_modified: i64 = file.date_modified.timestamp_millis();
    let date_accessed: i64 = file.date_accessed.timestamp_millis();
    let dropped: bool = file.dropped;
    let removed: bool = file.removed;

    let conn = DB_SQL.lock().await;

    let mut stmt = conn.prepare_cached(
        "   INSERT INTO files (
                    blake3_hash,
                    bao_hash,
                    bytes_read,
                    bytes_written,
                    min_slice,
                    max_slice,
                    path,
                    parent_rev,
                    mime_type,
                    date_created,
                    date_modified,
                    date_accessed,
                    dropped,
                    removed
                ) VALUES (
                    :blake3_hash,
                    :bao_hash,
                    :bytes_read,
                    :bytes_written,
                    :min_slice,
                    :max_slice,
                    :path,
                    :parent_rev,
                    :mime_type,
                    :date_created,
                    :date_modified,
                    :date_accessed,
                    :dropped,
                    :removed
                )",
    )?;

    stmt.execute(named_params! {
        ":blake3_hash": blake3_hash,
        ":bao_hash": bao_hash,
        ":bytes_read": bytes_read,
        ":bytes_written": bytes_written,
        ":min_slice": min_slice,
        ":max_slice": max_slice,
        ":path": path,
        ":parent_rev": parent_rev,
        ":mime_type": mime_type,
        ":date_created": date_created,
        ":date_modified": date_modified,
        ":date_accessed": date_accessed,
        ":dropped": dropped,
        ":removed": removed,
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

type BlakeHashSet = HashSet<blake3::Hash>;

fn join_set(set: BlakeHashSet) -> String {
    format!(
        "\"{}\"",
        &set.iter()
            .map(|h| h.to_hex().to_string())
            .collect::<Vec<String>>()
            .join("\", \""),
    )
}

fn diff_set(set_a: BlakeHashSet, set_b: &BlakeHashSet) -> BlakeHashSet {
    set_a.difference(set_b).into_iter().copied().collect()
}

/// Accepts optional comma-separated strings for specific hashes to retrieve, or omit
pub async fn get_files(
    include: Option<HashSet<blake3::Hash>>,
    exclude: Option<HashSet<blake3::Hash>>,
) -> Result<Vec<FileInfo>> {
    let conn = DB_SQL.lock().await;
    let mut query = "SELECT * FROM files WHERE dropped = FALSE".to_owned();

    if let Some(include_set) = include {
        let in_str = if let Some(exclude_set) = exclude.as_ref() {
            join_set(diff_set(include_set, exclude_set))
        } else {
            join_set(include_set)
        };
        query += &format!(" AND blake3_hash IN ({})", in_str);
    }

    if let Some(exclude_set) = exclude {
        query += &format!(" AND blake3_hash NOT IN ({})", join_set(exclude_set));
    }

    let mut stmt = conn.prepare(&query)?;

    let results = stmt.query_map([], |row| {
        let blake3_hash: String = row.get("blake3_hash")?;
        let bao_hash: String = row.get("bao_hash")?;
        let bytes_read: u64 = row.get("bytes_read")?;
        let bytes_written: u64 = row.get("bytes_written")?;
        let min_slice: u64 = row.get("min_slice")?;
        let max_slice: u64 = row.get("max_slice")?;
        let path: String = row.get("path")?;
        let parent_rev: Option<String> = row.get("parent_rev")?;
        let mime_type = row.get("mime_type")?;
        let date_created: i64 = row.get("date_created")?;
        let date_modified: i64 = row.get("date_modified")?;
        let date_accessed: i64 = row.get("date_accessed")?;
        let dropped: bool = row.get("dropped")?;
        let removed: bool = row.get("removed")?;

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
            bytes_read,
            bytes_written,
            min_slice,
            max_slice,
            path,
            parent_rev,
            mime_type,
            date_created,
            date_modified,
            date_accessed,
            dropped,
            removed,
        })
    })?;

    Ok(results.map(|res_fi| res_fi.unwrap()).collect())
}

pub async fn list_files() -> Result<Vec<String>> {
    let files = get_files(None, None).await?;

    let max_path_len = files.iter().fold(0, |acc, info| {
        cmp::max(acc, info.path.to_string_lossy().len())
    });

    Ok(files
        .iter()
        .map(|info| {
            let path = info.path.to_string_lossy();

            format!(
                "{size}\t\t{mime_type}\t{path}{path_space}",
                size = human_bytes(info.bytes_read as f64),
                mime_type = info.mime_type,
                path = path,
                path_space = " ".repeat(max_path_len - path.len()),
            )
        })
        .collect())
}

pub async fn mark_as_dropped(blake3_hash: blake3::Hash) -> Result<()> {
    let conn = DB_SQL.lock().await;
    let mut stmt = conn.prepare_cached(
        "   UPDATE files
                SET dropped = true
                WHERE blake3_hash = :blake3_hash",
    )?;

    stmt.execute(named_params! {
        ":blake3_hash": blake3_hash.to_hex().to_string(),
    })?;

    Ok(())
}

/// File is removed from both storage clients and storage providers, but still tracked so gaps can be accounted for
pub async fn mark_as_removed(blake3_hash: blake3::Hash) -> Result<()> {
    let conn = DB_SQL.lock().await;
    let mut stmt = conn.prepare_cached(
        "   UPDATE files
                SET dropped = TRUE, removed = TRUE
                WHERE blake3_hash = :blake3_hash",
    )?;

    stmt.execute(named_params! {
        ":blake3_hash": blake3_hash.to_hex().to_string(),
    })?;

    Ok(())
}

pub fn remove_hash(hash: blake3::Hash) -> Result<()> {
    DB_KV.open_tree(HASH_TREE)?.remove(hash.as_bytes())?;
    Ok(())
}

pub struct SliceIndexInfo {
    pub blake3_hash: String,
    pub bao_hash: String,
    pub file_slice_index: u64,
    pub data_dir_path: String,
}

pub async fn get_max_slice() -> Result<u64> {
    let conn = DB_SQL.lock().await;
    let mut stmt = conn.prepare_cached(
        "   SELECT MAX(max_slice)
                FROM files
                WHERE removed = FALSE",
    )?;

    let max_slice = stmt
        .query_row(params![], |row| row.get(0))
        .optional()
        .unwrap_or(Some(0))
        .unwrap();

    Ok(max_slice)
}

pub async fn get_random_slice_index(max_slice: u64) -> Result<SliceIndexInfo> {
    let conn = DB_SQL.lock().await;
    let mut stmt = conn.prepare_cached(
        "   SELECT blake3_hash, bao_hash, path
                FROM files
                WHERE
                    min_slice <= :slice_index AND
                    max_slice >= :slice_index AND
                    removed = FALSE",
    )?;

    // TODO: replace all RNGs with CSPRNGs
    let mut rng = rand::thread_rng();
    let file_slice_index = rng.gen_range(0..max_slice);

    let result = stmt.query_row(
        named_params! {
            ":slice_index": file_slice_index,
        },
        |row| {
            let blake3_hash: String = row.get("blake3_hash")?;
            let bao_hash: String = row.get("bao_hash")?;
            let data_dir_path: String = row.get("path")?;

            Ok(SliceIndexInfo {
                blake3_hash,
                bao_hash,
                file_slice_index,
                data_dir_path,
            })
        },
    )?;

    Ok(result)
}

pub async fn get_hashes_by_prefix(
    prefix: &str,
    exclude: &HashSet<blake3::Hash>,
) -> Result<HashSet<blake3::Hash>> {
    let mut hashes = HashSet::new();

    for try_hash in DB_KV.open_tree(PATHS_TREE)?.scan_prefix(prefix).values() {
        let blake3_hash = ivec_to_blake3_hash(try_hash?)?;
        if !exclude.contains(&blake3_hash) {
            hashes.insert(blake3_hash);
        }
    }

    Ok(hashes)
}
