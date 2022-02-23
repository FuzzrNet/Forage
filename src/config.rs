use std::{env, path::PathBuf};

use anyhow::Result;
use directories_next::{BaseDirs, UserDirs};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct EnvCfg {
    pub usr_home_dir: PathBuf,
    pub forage_cfg_dir: PathBuf,
    pub forage_cfg_file: PathBuf,
}

fn init_env_cfg() -> Result<EnvCfg> {
    let user_dirs = UserDirs::new().unwrap();
    let base_dirs = BaseDirs::new().unwrap();

    let usr_home_dir = user_dirs.home_dir().to_path_buf();

    let forage_cfg_dir = env::var("FORAGE_CFG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.config_dir().join("forage"));

    let forage_cfg_file = forage_cfg_dir.join("cfg.toml");

    Ok(EnvCfg {
        usr_home_dir,
        forage_cfg_dir,
        forage_cfg_file,
    })
}

pub static ENV_CFG: Lazy<EnvCfg> = Lazy::new(|| init_env_cfg().unwrap());

#[derive(Serialize, Deserialize)]
pub struct Volume {
    path: PathBuf,  // Path to mounted volume
    allocated: u64, // Allocated capacity in megabytes
}

#[derive(Deserialize)]
struct SysCfgFile {
    forage_data_dir: Option<String>,
    volume: Option<Vec<Volume>>,
}

#[derive(Serialize)]
pub struct SysCfg {
    pub forage_data_dir: PathBuf,
    pub volumes: Vec<Volume>,
}

pub async fn get_cfg() -> Result<SysCfg> {
    create_dir_all(&ENV_CFG.forage_cfg_dir).await?;
    create_dir_all(&ENV_CFG.forage_cfg_dir.join("sqlite_db")).await?;

    let mut cfg_contents = vec![];

    // Creates new empty config file if it doesn't exist
    let mut cfg_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&ENV_CFG.forage_cfg_file)
        .await?;

    cfg_file.read_to_end(&mut cfg_contents).await?;

    let sys_cfg: SysCfgFile = toml::from_slice(&cfg_contents)?;

    let volumes = sys_cfg
        .volume
        .map(|vols| {
            vols.iter()
                .map(|vol| Volume {
                    path: PathBuf::from(&vol.path),
                    allocated: vol.allocated,
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![Volume {
                path: PathBuf::from("/tmp/forage_data"),
                allocated: 1,
            }]
        });

    for vol in volumes.iter() {
        create_dir_all(&vol.path).await?;
    }

    let forage_data_dir = sys_cfg
        .forage_data_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| ENV_CFG.usr_home_dir.join("Forage Data"));

    create_dir_all(&forage_data_dir).await?;

    let config = SysCfg {
        forage_data_dir,
        volumes,
    };

    // Write parsed config back out to config file
    let toml = toml::to_string_pretty(&config)?;
    cfg_file.write_all(toml.as_bytes()).await?;

    Ok(config)
}

pub async fn get_storage_path() -> Result<PathBuf> {
    let cfg = get_cfg().await?;

    if cfg.volumes.len() > 1 {
        unimplemented!();
    } else {
        Ok(PathBuf::from(&cfg.volumes[0].path))
    }
}

pub async fn get_data_dir() -> Result<PathBuf> {
    let cfg = get_cfg().await?;

    Ok(cfg.forage_data_dir)
}
