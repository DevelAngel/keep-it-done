#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
pub use uuid::Uuid;

pub trait TaskProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn summary(&'a self) -> &'a str;
    fn created(&'a self) -> DateTime<Utc>;
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
    fn created(&'a self) -> DateTime<Utc> {
        assert_eq!(self.id.get_version_num(), 7);
        let timestamp = self.id.get_timestamp().expect("UUID v7 expected");
        let timestamp: SystemTime = timestamp.into();
        timestamp.into()
    }
}
