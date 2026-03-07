use kid_types::{TaskStatus, Uuid};

pub use clap::Parser;
use clap::{Args, Subcommand};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Family task management with assistant-friendly CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Command
    #[command(subcommand)]
    pub cmd: Commands,

    #[command(flatten)]
    pub verbosity: Verbosity,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate JSON Schema for Task
    Schema {
        /// pretty-printed JSON
        #[clap(short, long, default_value_t = false)]
        pretty: bool,
        /// write to file instead of stdout
        #[clap(short, long = "out")]
        outfile: Option<PathBuf>,
    },
    /// List all tasks
    List {
        /// printed in JSON
        #[clap(short, long, default_value_t = false)]
        json: bool,
        /// pretty-printed JSON
        #[clap(short, long, default_value_t = false, requires("json"))]
        pretty: bool,
        /// server options
        #[clap(flatten)]
        server: ServerArgs,
    },
    /// Add a new task
    Add {
        /// Task summary
        #[clap(short, long)]
        summary: String,
        /// Task details as JSON string (see schema)
        #[clap(short, long)]
        details: Option<String>,
        /// server options
        #[clap(flatten)]
        server: ServerArgs,
    },
    /// Complete task (or uncomplete it)
    Complete {
        /// Task ID
        #[clap(short, long)]
        id: Uuid,
        /// New task status
        #[clap(short, long, default_value_t = TaskStatus::Done)]
        status: TaskStatus,
        /// server options
        #[clap(flatten)]
        server: ServerArgs,
    },
}

#[derive(Debug, Args)]
#[clap(next_help_heading = "Server Options")]
#[clap(after_help = "The Server Options are not relevant for AI assistants.")]
pub struct ServerArgs {
    /// Sets the server address to connect to.
    #[clap(long = "server", env = "KID_SERVER_ADDR", default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000))]
    pub addr: SocketAddr,
}

#[cfg(debug_assertions)]
pub type Verbosity = clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>;

#[cfg(not(debug_assertions))]
pub type Verbosity = clap_verbosity_flag::Verbosity;
