use kid_types::Uuid;
use serde::{Deserialize, Serialize};

/// Server-to-client events, streamed via SSE.
///
/// Tagged enum so that new variants can be added without breaking
/// existing consumers.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    Flush(FlushOutcome),
    TaskChanged { id: Uuid, actor: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum FlushOutcome {
    Success { count: usize },
    Error { message: String },
}
