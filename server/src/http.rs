use crate::SharedTaskCache;
use kid_app::{App, shell};

use anyhow::Result;
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tokio::net::TcpListener;

pub struct HttpServer;

impl HttpServer {
    pub async fn serve(
        listener: TcpListener,
        leptos_options: LeptosOptions,
        task_cache: SharedTaskCache,
    ) -> Result<()> {
        let routes = generate_route_list(App);

        let app = Router::new()
            .leptos_routes_with_context(
                &leptos_options,
                routes,
                move || provide_context(task_cache.clone()),
                {
                    let leptos_options = leptos_options.clone();
                    move || shell(leptos_options.clone())
                },
            )
            .fallback(leptos_axum::file_and_error_handler(shell))
            .with_state(leptos_options);

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        tracing::info!(
            "Web listening on http://{}",
            &listener.local_addr().unwrap()
        );
        axum::serve(listener, app.into_make_service()).await?;
        Ok(())
    }
}
