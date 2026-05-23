use crate::{SharedTaskCache, SharedTimeOffset};
use kid_app::server::ssr::FallbackUser;
use kid_app::{App, shell};

use axum::Router;
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
        time_offset: SharedTimeOffset,
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

        let routes = generate_route_list(App);

        let app = {
            let task_cache = task_cache.clone();
            let time_offset = time_offset.clone();
            Router::new()
                .leptos_routes_with_context(
                    &leptos_options,
                    routes,
                    move || {
                        provide_context(task_cache.clone());
                        provide_context(fallback_user.clone());
                        provide_context(time_offset.clone());
                    },
                    {
                        let leptos_options = leptos_options.clone();
                        move || shell(leptos_options.clone())
                    },
                )
                .fallback(leptos_axum::file_and_error_handler(shell))
                .with_state(leptos_options)
        };
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
                tracing::info!("Web server shutting down");
            })
            .await
            .into_diagnostic()?;
        Ok(())
    }
}
