use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use log::{info, warn};
use structopt::StructOpt;
use tokio::signal;

use crate::file::process_path;

#[allow(dead_code)]
#[derive(StructOpt, Debug)]
#[structopt(name = "forage")]
/// A node for facilitating Storage Channels over the Lightning Network.
enum Commands {
    /// Create a new Onion URL and auth code for an authorized storage client
    NewClient {
        /// Internal label to associate with client
        label: String,
        /// Storage cap for client
        cap: Option<u64>,
    },
    /// Open a storage channel to a permissioned storage provider. Will prompt for auth.
    OpenChannel {
        /// Tor Onion v3 address to authorized storage node.
        address: String,
        // /// How many sats to dedicate to this channel?
        // balance: usize,
        // /// Rate to use for opening a channel on L1, in sats/vB.
        // rate: usize,
    },
    /// List storage channels.
    #[structopt(skip)]
    ListChannels {
        #[structopt(long, short)]
        providers: bool,
        #[structopt(long, short)]
        clients: bool,
    },
    /// Close a channel
    #[structopt(skip)]
    CloseChannel {
        /// Tor Onion v3 address to peer node.
        address: String,
        /// Force
        #[structopt(long, short)]
        force: bool,
    },
    /// Store a file or directory
    Store {
        /// File or directory to store over the storage channel
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// List files stored over storage channel
    #[structopt(skip)]
    ListFiles {
        /// Filter paths by prefix
        #[structopt(default_value = "/")]
        prefix: String,
        /// Recursive directory listing depth (if 0, list all files under the prefix recursively)
        #[structopt(default_value = "1")]
        depth: usize,
    },
    /// Retrieve a file by hash over available storage channels
    Retrieve {
        /// Path prefix. Multiple path matches will be saved to separate files and folders.
        prefix: String,
        /// Where to save the retrieved data
        #[structopt(parse(from_os_str))]
        out: PathBuf,
    },
    /// Issues a challenge to check if a provider is still hosting data for this storage channel.
    Check {
        /// Tor Onion v3 address to authorized storage node.
        address: String,
    },
    /// Allocate storage as an available storage provider.
    #[structopt(skip)]
    Allocate {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        size: usize,
    },
    /// Transfer data this node is providing to another node.
    #[structopt(skip)]
    Transfer {
        /// Tor Onion v3 address to authorized storage node.
        address: String,
    },
    /// Start storage node
    Start,
    /// Get node status
    Status,
}

pub async fn try_main() -> Result<()> {
    #[allow(unused_variables)]
    match Commands::from_args() {
        Commands::NewClient { label, cap } => {
            info!(
                "Creating a new channel for {} with a cap of {:?}",
                label, cap
            );
            warn!("Not yet implemented");
            todo!();
        }
        Commands::OpenChannel { address } => {
            info!("Opening a channel to {}", address);
            warn!("Not yet implemented");
            todo!();
        }
        Commands::ListChannels { providers, clients } => unimplemented!(),
        Commands::CloseChannel { address, force } => unimplemented!(),
        Commands::Store { path } => {
            info!("Storing a file at path {}...", path.to_str().unwrap());
            process_path(&path, current_dir()?).await?;
            Ok(())
        }
        Commands::ListFiles { prefix, depth } => unimplemented!(),
        Commands::Retrieve { prefix, out } => {
            info!("Retrieving files starting with {} over available storage channels and placing at {}...", prefix, out.to_str().unwrap());
            warn!("Not yet implemented");
            todo!();
        }
        Commands::Check { address } => {
            info!(
                "Check that all files are being stored on an existing storage channel at {}...",
                address
            );
            warn!("Not yet implemented");
            todo!();
        }
        Commands::Allocate { path, size } => unimplemented!(),
        Commands::Transfer { address } => unimplemented!(),
        Commands::Start => {
            info!("Starting Forage node...");
            signal::ctrl_c().await?;
            Ok(())
        }
        Commands::Status => {
            info!("Status from Forage node... Press CTRL-C to stop");
            warn!("Not yet implemented");
            todo!();
        }
    }
}
