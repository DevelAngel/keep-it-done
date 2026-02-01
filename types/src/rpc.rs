use crate::Task;

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<Task>;
    async fn add(summary: String);
}
