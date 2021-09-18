use std::path::PathBuf;

use anyhow::Result;
use directories_next::{BaseDirs, UserDirs};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

pub struct EnvCfg {
    pub home_dir: PathBuf,
    pub cfg_dir: PathBuf,
    pub cfg_path: PathBuf,
}

fn init_env_cfg() -> Result<EnvCfg> {
    let user_dirs = UserDirs::new().unwrap();
    let base_dirs = BaseDirs::new().unwrap();

    let home_dir = user_dirs.home_dir().join("Forage");
    let cfg_dir = base_dirs.config_dir().join("forage");

    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&cfg_dir)?;

    let cfg_path = cfg_dir.join("cfg.toml");

    Ok(EnvCfg {
        home_dir,
        cfg_dir,
        cfg_path,
    })
}

pub static ENV_CFG: Lazy<EnvCfg> = Lazy::new(|| init_env_cfg().unwrap());

#[derive(Deserialize)]
pub struct Volume {
    path: String,   // Path to mounted volume
    allocated: u64, // Allocated capacity in megabytes
}

#[derive(Deserialize)]
pub struct SysCfg {
    volume: Option<Vec<Volume>>,
}

pub async fn get_cfg() -> Result<SysCfg> {
    let mut cfg_contents = vec![];

    // Creates new empty config file if it doesn't exist
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&ENV_CFG.cfg_path)
        .await?
        .read_to_end(&mut cfg_contents)
        .await?;

    let mut sys_cfg: SysCfg = toml::from_slice(&cfg_contents)?;

    if sys_cfg.volume.is_none() {
        sys_cfg.volume = Some(vec![Volume {
            path: "/tmp/forage_data".to_owned(),
            allocated: 1,
        }]);
    }

    for vol in sys_cfg.volume.as_ref().unwrap() {
        tokio::fs::create_dir_all(&vol.path).await?;
    }

    Ok(sys_cfg)
}

pub async fn get_storage_path() -> Result<PathBuf> {
    let cfg = get_cfg().await?;

    if cfg.volume.as_ref().unwrap().len() > 1 {
        unimplemented!();
    } else {
        Ok(PathBuf::from(&cfg.volume.unwrap()[0].path))
    }
}
