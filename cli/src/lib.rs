pub mod cli;
pub mod task;

pub use kid_types::rpc::TaskServiceClient;

use miette::{IntoDiagnostic, Result, WrapErr};
use tarpc::client;
use tarpc::serde_transport::tcp;
use tarpc::tokio_serde::formats::Json;

use std::net::SocketAddr;

/// Connect to a kid-server tarpc endpoint.
pub async fn connect(addr: &SocketAddr) -> Result<TaskServiceClient> {
    tracing::info!("CLI will connect to {}", addr);
    let mut transport = tcp::connect(addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let transport = transport
        .await
        .into_diagnostic()
        .wrap_err("failed to connect")?;

    let client = TaskServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}
