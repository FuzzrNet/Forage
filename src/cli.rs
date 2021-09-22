use std::path::PathBuf;

use anyhow::Result;
use log::{info, warn};
use structopt::StructOpt;
use tokio::signal;

use crate::{db::list_files, file::upload_path};

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
    /// Uploads files in the Forage Data folder on available storage channels (de-duplicating and creating revisions as necessary)
    Upload {
        /// Restrict pruning to just paths with this prefix (relative to the Forage Data folder)
        #[structopt(default_value = "")]
        prefix: String,
    },
    /// Issues a challenge to verify if a provider is still hosting data for this storage channel.
    Verify,
    /// Retrieve a file by its path prefix over available storage channels (leave empty to retrieve all files, de-duplicating as necessary)
    Download {
        /// Path prefix. Multiple path matches will be saved to separate files and folders.
        #[structopt(default_value = "")]
        prefix: String,
    },
    /// List files stored over storage channel
    ListFiles {
        /// Filter paths by prefix
        #[structopt(default_value = "/")]
        prefix: String,
        /// Recursive directory listing depth (if 0, list all files under the prefix recursively)
        #[structopt(default_value = "1")]
        depth: usize,
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
        Commands::Upload { prefix } => {
            info!("Storing data in Forest Data directory over available storage channels...");
            upload_path(prefix).await?;
            Ok(())
        }
        Commands::Verify => {
            info!("Verify data possession on existing storage channels...",);
            warn!("Not yet implemented");
            todo!();
        }
        Commands::Download { prefix } => {
            info!(
                "Retrieving files under {} over available storage channels...",
                prefix
            );
            warn!("Not yet implemented");
            todo!();
        }
        Commands::ListFiles { prefix, depth } => {
            let files = list_files().await?;
            info!("{} files stored:\n{}", files.len(), files.join("\n"));
            Ok(())
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
