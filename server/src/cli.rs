use clap::Args;
pub use clap::Parser;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Family task management server, with an MCP interface for AI assistants
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub server: ServerArgs,

    /// Directory where task JSON files are stored.
    /// Defaults to the current working directory.
    #[clap(long, env = "KID_TASKS_DIR")]
    pub tasks_dir: Option<PathBuf>,

    #[command(flatten)]
    pub verbosity: Verbosity,
}

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Sets the address the MCP server listens on.
    ///
    /// Kept on its own port, separate from the browser-facing HTTP
    /// server, so AI assistant clients never have to pass through
    /// Tinyauth's interactive login flow (see ADR: rmcp-mcp-server).
    #[clap(long = "mcp-listen", global = true, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9100))]
    pub mcp_addr: SocketAddr,
}

#[cfg(debug_assertions)]
pub type Verbosity = clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>;

#[cfg(not(debug_assertions))]
pub type Verbosity = clap_verbosity_flag::Verbosity;
