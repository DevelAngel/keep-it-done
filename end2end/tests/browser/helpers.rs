use kid_types::ViewSlug;
use strum::{Display, EnumString};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// MCP server address — used for actual task mutations (`add`) in tests.
pub const MCP_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    9100,
);

/// Admin channel address for the e2e test harness's server-control
/// operations (`switch_dir`/`count`/`set_time_offset`/`reset_time_offset`/
/// `flush`). Only bound when `kid-server` runs with the `test-control`
/// feature — see `docs/adr/rmcp-mcp-server.md`.
pub const TEST_CONTROL_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    9200,
);

/// OAuth client identity used to authenticate against the MCP server's
/// authorization-code flow. Must match `end2end/mcp-clients.test.toml`,
/// which `justfile`'s `test-e2e` target points the server at via
/// `KID_MCP_CLIENTS_FILE`. `OAUTH_REDIRECT_URI` is never actually served —
/// see `oauth::fetch_access_token`.
pub const OAUTH_CLIENT_ID: &str = "kid-e2e";
pub const OAUTH_CLIENT_SECRET: &str = "e2e-test-secret";
pub const OAUTH_REDIRECT_URI: &str = "http://localhost:9299/callback";

#[derive(Display, EnumString)]
pub enum ViewName {
    #[strum(serialize = "Upcoming")]
    Upcoming,
    #[strum(serialize = "Quick Wins")]
    QuickWins,
    #[strum(serialize = "All Open")]
    AllOpen,
    #[strum(serialize = "What I Finished")]
    WhatIFinished,
    #[strum(serialize = "Recent Changes")]
    RecentChanges,
}

impl From<&ViewName> for ViewSlug {
    fn from(name: &ViewName) -> Self {
        match name {
            ViewName::Upcoming       => Self::Upcoming,
            ViewName::QuickWins      => Self::QuickWins,
            ViewName::AllOpen        => Self::AllOpen,
            ViewName::WhatIFinished  => Self::WhatIFinished,
            ViewName::RecentChanges  => Self::RecentlyChanged,
        }
    }
}

impl ViewName {
    pub fn url_param(&self) -> &'static str {
        ViewSlug::from(self).into()
    }

    pub fn screenshot_file(&self) -> PathBuf {
        let slug: String = self.to_string()
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        format!("task-list-{slug}.png").into()
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ViewSwitch {
    Next,
    Prev,
}
