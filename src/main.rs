mod cli;
mod config;
mod db;
mod file;
mod hash;
mod net;

use std::{env, error::Error, process};

use log::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();

    if let Err(err) = cli::try_main().await {
        error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        process::exit(1);
    }

    Ok(())
}
