#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;

use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

pub trait TaskProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn summary(&'a self) -> &'a str;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    id: Uuid,
    summary: String,
}

impl<'a> TaskProperties<'a> for Task {
    fn id(&'a self) -> &'a Uuid {
        &self.id
    }
    fn summary(&'a self) -> &'a str {
        &self.summary
    }
}
