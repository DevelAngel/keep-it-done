use serde::{Deserialize, Serialize};
use tarpc::service;
use uuid::Uuid;

use std::vec::Vec;

#[service]
pub trait TaskService {
    /// Returns a greeting for name.
    async fn list() -> Vec<Task>;
}

pub trait TaskProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn summary(&'a self) -> &'a str;
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        let id = Uuid::now_v7();
        let summary = summary.to_string();
        Self { id, summary }
    }
}
