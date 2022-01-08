use std::{
    cmp,
    collections::HashSet,
    convert::TryInto,
    fs::File,
    io::{Seek, Write},
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use bincode::Options;
use chrono::{DateTime, NaiveDateTime, Utc};
use dipa::{Diffable, Patchable};
use fd_lock::RwLock;
use human_bytes::human_bytes;
use log::error;
use once_cell::sync::Lazy;
use rand::{Rng, RngCore};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sled::{Config, Db, IVec, Mode};
use tokio::sync::Mutex;

use crate::{
    config::ENV_CFG,
    hash::{parse_bao_hash, parse_blake3_hash},
};

static Index: Lazy<Arc<Db>> = Lazy::new(|| {
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

pub fn encode<T: Serialize>(record: T) -> Vec<u8> {
    // Serialize
    let bin = bincode::options().with_varint_encoding();
    let serialized = bin.serialize(&record).unwrap();

    // Compress

    // Encrypt

    serialized
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> T {
    // Decrypt

    // Decompress

    // Deserialize
    let bin = bincode::options().with_varint_encoding();
    let record: T = bin.deserialize(bytes).unwrap();

    record
}

pub struct LogDb {
    file: RwLock<File>,
}

impl<'a, 'b> LogDb {
    /// Open a database file
    pub fn open(path: &Path) -> Result<Self> {
        // Create a new file if one does not exist

        // Open file with advisory lock
        let file = RwLock::new(File::open(path)?);

        Ok(Self { file })
    }

    pub fn upsert<T>(&mut self, key: &[u8], value: T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Diffable<'a, 'b, T> + Patchable<T>,
        for<'de> <T as Diffable<'a, 'b, T>>::DeltaOwned: Deserialize<'de>,
    {
        // If key exists, compute deltas and push new loc delta onto index value
        if let Some(prev) = Index.get(key)? {
            // Get previous deltas and compute old record state
            let mut prev_locs: Vec<(u64, usize)> = decode(&prev);
            let mut old_record = None;

            for (offset, len) in prev_locs.iter() {
                let buf = Vec::with_capacity(*len);
                self.file.read()?.read_exact_at(&mut buf, *offset)?;
                let delta: <T as dipa::Diffable<'_, '_, T>>::DeltaOwned = decode(&buf);

                if let Some(patch) = old_record {
                    delta.apply_patch(patch);
                }

                old_record = Some(delta);
            }

            // Compute new delta
            if let Some(old_record) = old_record {
                let delta = old_record.create_delta_towards(&value);
            }

            // Write new delta
            let buf = encode(value);
            self.file.write()?.write_all(&buf)?;

            // Update index with new loc
            let loc = (self.file.read()?.stream_position()?, buf.len());
            prev_val.push(loc);
            Index.insert(key, encode(loc))?;
        } else {
            Index.insert(key)?;
        }

        Ok(())
    }

    pub fn get<T>(key: &[u8]) -> Result<T>
    where
        T: Diffable<'a, 'b, T>,
    {
        // If key exists, compute deltas and push new loc delta onto index value
        if let Some(prev) = Index.get(key)? {
            // Get previous deltas and compute old record state
            let mut prev_locs: Vec<(u64, usize)> = decode(&prev);
            let mut old_record = None;

            for (offset, len) in prev_locs.iter() {
                let buf = Vec::with_capacity(*len);
                self.file.read()?.read_exact_at(&mut buf, *offset)?;
                let delta: <T as Diffable<'_, '_, T>>::DeltaOwned = decode(&buf);

                if let Some(patch) = old_record {
                    delta.apply_patch(patch);
                }

                old_record = Some(delta);
            }

            // Compute new delta
            if let Some(old_record) = old_record {
                let delta = old_record.create_delta_towards(&value);
            }

            // Write new delta
            let buf = encode(value);
            self.file.write()?.write_all(&buf)?;

            // Update index with new loc
            let loc = (self.file.read()?.stream_position()?, buf.len());
            prev_val.push(loc);
            Index.insert(key, encode(loc))?;
        } else {
            Index.insert(key)?;
        }

        Ok(record)
    }

    pub fn nearest_lt() {
        todo!();
    }

    pub fn delete() {
        todo!();
    }
}
