use crate::SharedTaskCache;
use kid_app::server::ssr::FallbackUser;
use kid_app::{App, shell};

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use miette::{IntoDiagnostic, Result};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub struct HttpServer;

impl HttpServer {
    pub async fn serve(
        listener: TcpListener,
        leptos_options: LeptosOptions,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
    ) -> Result<()> {
        let fallback_user = FallbackUser::new(
            std::env::var("KID_FALLBACK_USER").ok(),
        );
        let fallback_label = match fallback_user.as_deref() {
            Some(user) => format!("fallback: {user}"),
            None => "no fallback, Remote-User required".into(),
        };
        tracing::info!(
            "HTTP server listening on http://{} ({fallback_label})",
            listener.local_addr().unwrap()
        );

        let app = {
            let routes = generate_route_list(App);
            let task_cache = task_cache.clone();
            Router::new()
                .leptos_routes_with_context(
                    &leptos_options,
                    routes,
                    move || {
                        provide_context(task_cache.clone());
                        provide_context(fallback_user.clone());
                    },
                    {
                        let leptos_options = leptos_options.clone();
                        move || shell(leptos_options.clone())
                    },
                )
                .fallback(leptos_axum::file_and_error_handler(shell))
                .with_state(leptos_options)
        };

        let rest_api = {
            let task_cache = task_cache.clone();
            Router::new()
                .route("/test/cache/test", get(test_cache))
                .route("/api/cache/reload", post(reload_cache))
                .with_state(task_cache)
        };

        axum::serve(listener, app.merge(rest_api).into_make_service())
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
                tracing::info!("Web server shutting down");
            })
            .await
            .into_diagnostic()?;
        Ok(())
    }
}

async fn test_cache(State(cache): State<SharedTaskCache>) -> &'static str {
    let _cache = cache.write().await;
    "test"
}

async fn reload_cache(State(cache): State<SharedTaskCache>) -> StatusCode {
    let mut cache = cache.write().await;

    match cache.flush().await {
        Ok(num) => {
            if num > 0 {
                tracing::info!("{num} tasks successfully flushed before reload");
            } else {
                tracing::debug!("no tasks to flush before reload")
            }
        }
        Err(e) => {
            tracing::error!("task cache failed to reload: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    match cache.reload().await {
        Ok((num_loaded, num_to_migrate)) => {
            if num_loaded > 0 {
                tracing::info!("{num_loaded} tasks loaded");
                if num_to_migrate > 0 {
                    tracing::info!("{num_to_migrate} tasks has to be migrated with next flush");
                }
            } else {
                tracing::warn!("no tasks loaded");
            }
            StatusCode::NO_CONTENT
        }
        Err(e) => {
            tracing::error!("task cache failed to reload: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
