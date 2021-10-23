use std::{env, error::Error, path::PathBuf, process};

use anyhow::Result;
use log::error;
use structopt::StructOpt;

#[allow(dead_code)]
#[derive(StructOpt, Debug)]
#[structopt(name = "forage")]
/// Forage is for Storage.
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
    /// Retrieve a file by its path prefix over available storage channels (leave empty to retrieve all files, de-duplicating as necessary)
    Download {
        /// Path prefix. Multiple path matches will be saved to separate files and folders.
        #[structopt(default_value = "")]
        prefix: String,
    },
    /// Issues a challenge to verify if a provider is still hosting data for this storage channel.
    Verify,
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
        Commands::NewClient { label, cap } => forage::new_client(&label, cap),
        Commands::OpenChannel { address } => forage::open_channel(&address),
        Commands::ListChannels { providers, clients } => unimplemented!(),
        Commands::CloseChannel { address, force } => unimplemented!(),
        Commands::Upload { prefix } => forage::upload(&prefix).await?,
        Commands::Download { prefix } => forage::download(&prefix).await?,
        Commands::Verify => forage::verify().await?,
        Commands::ListFiles { prefix, depth } => forage::list_files(&prefix, depth).await?,
        Commands::Allocate { path, size } => unimplemented!(),
        Commands::Transfer { address } => unimplemented!(),
        Commands::Start => forage::start().await?,
        Commands::Status => forage::status(),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();

    if let Err(err) = try_main().await {
        error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        process::exit(1);
    }

    Ok(())
}
