pub use clap::Parser;
use clap::{Args, Subcommand};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Family task management with assistant-friendly CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Command
    #[command(subcommand)]
    pub cmd: Commands,

    #[clap(flatten)]
    pub server: ServerArgs,

    #[command(flatten)]
    pub verbosity: Verbosity,
}

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Sets the server address to connect to.
    #[clap(long = "server", global = true, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))]
    pub addr: SocketAddr,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all tasks
    List,
}

#[cfg(debug_assertions)]
pub type Verbosity = clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>;

#[cfg(not(debug_assertions))]
pub type Verbosity = clap_verbosity_flag::Verbosity;
