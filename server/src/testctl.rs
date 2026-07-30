//! Admin channel for the e2e browser test harness only.
//!
//! Exposes the pure test-control operations the old tarpc `RpcService`
//! used to provide (`switch_dir`, `count`, `set_time_offset`,
//! `reset_time_offset`, `flush`). Never compiled into a production
//! build — gated behind the `test-control` Cargo feature and, even
//! then, only bound when a listen address is explicitly configured.
//! See "MVP scope — server-control operations open" in
//! `docs/adr/rmcp-mcp-server.md`.

use crate::cache::SharedEventBus;
use crate::SharedTaskCache;
use crate::SharedTimeOffset;

use kid_app::events::{FlushOutcome, ServerEvent};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::Level;

use std::env;
use std::path::PathBuf;

pub struct TestControlServer;

#[derive(Clone)]
struct TestControlState {
    task_cache: SharedTaskCache,
    time_offset: SharedTimeOffset,
    event_bus: SharedEventBus,
}

#[derive(Deserialize)]
struct SwitchDirRequest {
    dir: PathBuf,
}

#[derive(Serialize)]
struct SwitchDirResponse {
    loaded: usize,
}

#[derive(Deserialize)]
struct SetTimeOffsetRequest {
    seconds: i64,
}

#[derive(Serialize)]
struct CountResponse {
    count: usize,
}

#[derive(Serialize)]
struct FlushResponse {
    flushed: usize,
}

impl TestControlServer {
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
        time_offset: SharedTimeOffset,
        event_bus: SharedEventBus,
    ) -> Result<()> {
        tracing::warn!(
            "test-control admin channel listening on: http://{} — \
             e2e test harness only, never enable in production",
            listener.local_addr().unwrap()
        );

        let state = TestControlState { task_cache, time_offset, event_bus };
        let app = Router::new()
            .route("/switch-dir", post(switch_dir))
            .route("/count", get(count))
            .route("/set-time-offset", post(set_time_offset))
            .route("/reset-time-offset", post(reset_time_offset))
            .route("/flush", post(flush))
            .with_state(state);

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
                tracing::info!("test-control admin channel shutting down");
            })
            .await
            .into_diagnostic()?;
        Ok(())
    }
}

async fn switch_dir(
    State(state): State<TestControlState>,
    Json(SwitchDirRequest { dir }): Json<SwitchDirRequest>,
) -> Result<Json<SwitchDirResponse>, (StatusCode, String)> {
    let internal_error = |e: &dyn std::fmt::Display| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    let mut cache = state.task_cache.write().await;
    cache.flush().await.map_err(|e| internal_error(&e))?;
    if dir.is_absolute() {
        tracing::info!("new tasks directory: {}", dir.display());
    } else if tracing::enabled!(Level::INFO) {
        tracing::info!("new tasks directory (relative): {}", dir.display());
        let cwd = env::current_dir().expect("CWD available");
        let dir = cwd.join(&dir);
        tracing::info!("new tasks directory (absolute): {}", dir.display());
    }
    cache.reset(dir);
    let (loaded, _) = cache.load().await.map_err(|e| internal_error(&e))?;
    Ok(Json(SwitchDirResponse { loaded }))
}

async fn count(State(state): State<TestControlState>) -> Json<CountResponse> {
    let cache = state.task_cache.read().await;
    Json(CountResponse { count: cache.len() })
}

async fn set_time_offset(
    State(state): State<TestControlState>,
    Json(SetTimeOffsetRequest { seconds }): Json<SetTimeOffsetRequest>,
) {
    tracing::info!("setting time offset to {seconds}s");
    state.time_offset.set(seconds);
}

async fn reset_time_offset(State(state): State<TestControlState>) {
    tracing::info!("resetting time offset");
    state.time_offset.reset();
}

async fn flush(State(state): State<TestControlState>) -> Json<FlushResponse> {
    tracing::info!("force-flushing task cache");
    let mut cache = state.task_cache.write().await;
    let flushed = match cache.flush().await {
        Ok(num) => {
            if num > 0 {
                let _ = state
                    .event_bus
                    .send(ServerEvent::Flush(FlushOutcome::Success { count: num }));
            }
            num
        }
        Err(e) => {
            tracing::error!("flush failed: {e}");
            let _ = state.event_bus.send(ServerEvent::Flush(FlushOutcome::Error {
                message: e.to_string(),
            }));
            0
        }
    };
    Json(FlushResponse { flushed })
}
