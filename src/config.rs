use std::path::PathBuf;

use directories_next::{BaseDirs, UserDirs};
use once_cell::sync::Lazy;

pub struct Config {
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let user_dirs = UserDirs::new().unwrap();
    let base_dirs = BaseDirs::new().unwrap();

    let home_dir = user_dirs.home_dir().join("Forage");
    let config_dir = base_dirs.config_dir().join("forage");

    Config {
        home_dir,
        config_dir,
    }
});
