use serde::{Deserialize, Serialize};

/// Server-to-client events, streamed via SSE.
///
/// Tagged enum so that future variants (e.g. `TaskChanged`) can be
/// added without breaking existing consumers.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    Flush(FlushOutcome),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum FlushOutcome {
    Success { count: usize },
    Error { message: String },
}
