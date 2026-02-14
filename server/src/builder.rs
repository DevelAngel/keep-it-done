use crate::http::HttpServer;
use crate::rpc::RpcServer;
use crate::{SharedTaskCache, TaskCacheFlush};

use leptos::prelude::*;
use miette::{IntoDiagnostic, Result};
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::try_join;
use tokio_util::sync::CancellationToken;

use std::net::SocketAddr;

pub struct ServerHandles {
    task_flush: JoinHandle<()>,
    rpc_service: JoinHandle<Result<()>>,
    http_service: JoinHandle<Result<()>>,
}

pub struct ServerBuilder<RPC, HTTP, LeptosOptions> {
    rpc_addr: RPC,
    http_addr: HTTP,
    leptos_options: LeptosOptions,
    shutdown: CancellationToken,
    task_cache: SharedTaskCache,
}

#[doc(hidden)]
pub(super) struct Unset;

impl ServerBuilder<Unset, Unset, Unset> {
    pub fn new(shutdown: &CancellationToken, task_cache: &SharedTaskCache) -> Self {
        Self {
            rpc_addr: Unset,
            http_addr: Unset,
            leptos_options: Unset,
            shutdown: shutdown.clone(),
            task_cache: task_cache.clone(),
        }
    }
}

impl<W, L> ServerBuilder<Unset, W, L> {
    pub fn with_rpc_addr(self, addr: &SocketAddr) -> ServerBuilder<SocketAddr, W, L> {
        ServerBuilder {
            rpc_addr: *addr,
            http_addr: self.http_addr,
            leptos_options: self.leptos_options,
            shutdown: self.shutdown,
            task_cache: self.task_cache,
        }
    }
}

impl<R, L> ServerBuilder<R, Unset, L> {
    pub fn with_http_addr(self, addr: &SocketAddr) -> ServerBuilder<R, SocketAddr, L> {
        ServerBuilder {
            rpc_addr: self.rpc_addr,
            http_addr: *addr,
            leptos_options: self.leptos_options,
            shutdown: self.shutdown,
            task_cache: self.task_cache,
        }
    }
}

impl<R, W> ServerBuilder<R, W, Unset> {
    pub fn with_leptos_options(
        self,
        options: &LeptosOptions,
    ) -> ServerBuilder<R, W, LeptosOptions> {
        ServerBuilder {
            rpc_addr: self.rpc_addr,
            http_addr: self.http_addr,
            leptos_options: options.clone(),
            shutdown: self.shutdown,
            task_cache: self.task_cache,
        }
    }
}

impl ServerBuilder<SocketAddr, SocketAddr, LeptosOptions> {
    pub async fn try_spawn(self) -> Result<ServerHandles> {
        let (rpc_listener, http_listener) = try_join!(
            TcpListener::bind(&self.rpc_addr),
            TcpListener::bind(&self.http_addr),
        )
        .into_diagnostic()?;

        let http_service = spawn(HttpServer::serve(
            http_listener,
            self.leptos_options,
            self.shutdown.clone(),
            self.task_cache.clone(),
        ));
        let rpc_service = spawn(RpcServer::serve(
            rpc_listener,
            self.shutdown.clone(),
            self.task_cache.clone(),
        ));
        let task_flush =
            { spawn(async move { self.task_cache.background_flush(self.shutdown).await }) };
        Ok(ServerHandles {
            task_flush,
            rpc_service,
            http_service,
        })
    }
}

impl ServerHandles {
    pub async fn join(self) -> Result<()> {
        let _ =
            try_join!(self.task_flush, self.rpc_service, self.http_service).into_diagnostic()?;
        Ok(())
    }
}
