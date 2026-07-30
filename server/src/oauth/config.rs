use miette::{IntoDiagnostic, Result};
use secrecy::SecretString;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use url::Url;

/// A single OAuth client allowed to authenticate against the MCP server,
/// as read from the `--mcp-clients-file` TOML file.
#[derive(Clone, Debug, Deserialize)]
pub struct McpClientConfig {
    pub name: String,
    pub redirect_uri: Url,
    pub secret: SecretString,
}

/// Top-level shape of the `--mcp-clients-file` TOML file:
///
/// ```toml
/// [[client]]
/// name = "mcp-inspector"
/// redirect_uri = "http://localhost:6274/oauth/callback"
/// secret = "..."
/// ```
#[derive(Clone, Debug, Default, Deserialize)]
pub struct McpClientsConfig {
    #[serde(rename = "client", default)]
    pub clients: Vec<McpClientConfig>,
}

impl McpClientsConfig {
    /// Loads the client registry from `path`, or returns an empty registry
    /// (MCP OAuth disabled) if `path` is `None`.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            tracing::warn!(
                "no --mcp-clients-file given: MCP OAuth is disabled, no client can authenticate"
            );
            return Ok(Self::default());
        };

        let raw = fs::read_to_string(path).into_diagnostic()?;
        let config: Self = toml::from_str(&raw).into_diagnostic()?;

        if config.clients.is_empty() {
            tracing::error!(
                "{} lists no clients: MCP OAuth is disabled, no client can authenticate",
                path.display()
            );
        } else {
            tracing::info!(
                "{} MCP OAuth client(s) loaded from {}",
                config.clients.len(),
                path.display()
            );
        }

        Ok(config)
    }
}
