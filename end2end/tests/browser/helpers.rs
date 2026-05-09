use kid_types::ViewSlug;
use strum::{Display, EnumString};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

pub const RPC_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    9000,
);

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
