use crate::http::HttpServer;
use crate::mcp::McpServer;
use crate::oauth::McpClientsConfig;
use crate::cache::SharedEventBus;
use crate::{SharedTaskCache, SharedTimeOffset, TaskCacheFlush};

use leptos::prelude::*;
use miette::{IntoDiagnostic, Result};
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::try_join;
use tokio_util::sync::CancellationToken;
use url::Url;

use std::net::SocketAddr;

pub struct ServerHandles {
    task_flush: JoinHandle<()>,
    mcp_service: JoinHandle<Result<()>>,
    http_service: JoinHandle<Result<()>>,
}

pub struct ServerBuilder<MCP, HTTP, LeptosOptions> {
    mcp_addr: MCP,
    mcp_base_url: Option<Url>,
    mcp_allowed_origins: Vec<Url>,
    mcp_clients: McpClientsConfig,
    http_addr: HTTP,
    leptos_options: LeptosOptions,
    shutdown: CancellationToken,
    task_cache: SharedTaskCache,
    time_offset: SharedTimeOffset,
    event_bus: SharedEventBus,
}

#[doc(hidden)]
pub(super) struct Unset;

impl ServerBuilder<Unset, Unset, Unset> {
    pub fn new(
        shutdown: &CancellationToken,
        task_cache: &SharedTaskCache,
        time_offset: &SharedTimeOffset,
    ) -> Self {
        Self {
            mcp_addr: Unset,
            mcp_base_url: None,
            mcp_allowed_origins: Vec::new(),
            mcp_clients: McpClientsConfig::default(),
            http_addr: Unset,
            leptos_options: Unset,
            shutdown: shutdown.clone(),
            task_cache: task_cache.clone(),
            time_offset: time_offset.clone(),
            event_bus: SharedEventBus::new(),
        }
    }
}

impl<W, L> ServerBuilder<Unset, W, L> {
    pub fn with_mcp_addr(self, addr: &SocketAddr) -> ServerBuilder<SocketAddr, W, L> {
        ServerBuilder {
            mcp_addr: *addr,
            mcp_base_url: self.mcp_base_url,
            mcp_allowed_origins: self.mcp_allowed_origins,
            mcp_clients: self.mcp_clients,
            http_addr: self.http_addr,
            leptos_options: self.leptos_options,
            shutdown: self.shutdown,
            task_cache: self.task_cache,
            time_offset: self.time_offset,
            event_bus: self.event_bus,
        }
    }
}

impl<R, W, L> ServerBuilder<R, W, L> {
    /// This server's own public URL, used for OAuth metadata; see
    /// `cli::ServerArgs::mcp_base_url`.
    pub fn with_mcp_base_url(mut self, base_url: &Url) -> Self {
        self.mcp_base_url = Some(base_url.clone());
        self
    }

    /// Origins allowed to access the MCP server cross-origin; see
    /// `cli::ServerArgs::mcp_allowed_origins`.
    pub fn with_mcp_allowed_origins(mut self, origins: &[Url]) -> Self {
        self.mcp_allowed_origins = origins.to_vec();
        self
    }

    /// OAuth clients allowed to authenticate against the MCP server; see
    /// `cli::ServerArgs::mcp_clients_file`.
    pub fn with_mcp_clients(mut self, clients: McpClientsConfig) -> Self {
        self.mcp_clients = clients;
        self
    }
}

impl<R, L> ServerBuilder<R, Unset, L> {
    pub fn with_http_addr(self, addr: &SocketAddr) -> ServerBuilder<R, SocketAddr, L> {
        ServerBuilder {
            mcp_addr: self.mcp_addr,
            mcp_base_url: self.mcp_base_url,
            mcp_allowed_origins: self.mcp_allowed_origins,
            mcp_clients: self.mcp_clients,
            http_addr: *addr,
            leptos_options: self.leptos_options,
            shutdown: self.shutdown,
            task_cache: self.task_cache,
            time_offset: self.time_offset,
            event_bus: self.event_bus,
        }
    }
}

impl<R, W> ServerBuilder<R, W, Unset> {
    pub fn with_leptos_options(
        self,
        options: &LeptosOptions,
    ) -> ServerBuilder<R, W, LeptosOptions> {
        ServerBuilder {
            mcp_addr: self.mcp_addr,
            mcp_base_url: self.mcp_base_url,
            mcp_allowed_origins: self.mcp_allowed_origins,
            mcp_clients: self.mcp_clients,
            http_addr: self.http_addr,
            leptos_options: options.clone(),
            shutdown: self.shutdown,
            task_cache: self.task_cache,
            time_offset: self.time_offset,
            event_bus: self.event_bus,
        }
    }
}

impl ServerBuilder<SocketAddr, SocketAddr, LeptosOptions> {
    pub async fn try_spawn(self) -> Result<ServerHandles> {
        let (mcp_listener, http_listener) = try_join!(
            TcpListener::bind(&self.mcp_addr),
            TcpListener::bind(&self.http_addr),
        )
        .into_diagnostic()?;

        let http_service = spawn(HttpServer::serve(
            http_listener,
            self.leptos_options,
            self.shutdown.clone(),
            self.task_cache.clone(),
            self.time_offset.clone(),
            self.event_bus.clone(),
        ));
        let mcp_service = spawn(McpServer::serve(
            mcp_listener,
            self.shutdown.clone(),
            self.task_cache.clone(),
            self.mcp_base_url.expect("mcp_base_url must be set via with_mcp_base_url"),
            self.mcp_allowed_origins.clone(),
            self.mcp_clients.clone(),
        ));
        let task_flush = {
            let event_bus = self.event_bus;
            spawn(async move {
                self.task_cache
                    .background_flush(self.shutdown, &event_bus)
                    .await
            })
        };
        Ok(ServerHandles {
            task_flush,
            mcp_service,
            http_service,
        })
    }
}

impl ServerHandles {
    pub async fn join(self) -> Result<()> {
        let _ =
            try_join!(self.task_flush, self.mcp_service, self.http_service).into_diagnostic()?;
        Ok(())
    }
}
