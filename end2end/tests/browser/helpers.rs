use strum::{Display, EnumString};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const RPC_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    9000,
);

#[derive(Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ViewSwitch {
    Next,
    Prev,
}
