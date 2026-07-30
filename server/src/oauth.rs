use rmcp::transport::auth::{AuthorizationMetadata, OAuthClientConfig};

use axum::Json;
use axum::body::{self, Body};
use axum::extract::{Form, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};

use askama::Template;
use base64::Engine;
use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From, FromStr};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tokio::time;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use url::Url;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

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
    pub fn load(path: Option<&Path>) -> miette::Result<Self> {
        use miette::IntoDiagnostic;

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

/// Stores access and refresh tokens per client
#[derive(Clone, Debug)]
pub struct McpOAuthStore {
    /// This server's own public URL, used when issuing OAuth metadata
    /// (issuer, authorization/token endpoints).
    base_url: Url,
    clients: Arc<RwLock<HashMap<ClientId, OAuthClientConfig>>>,
    auth_sessions: Arc<RwLock<HashMap<AuthCode, AdditionalData>>>,
    access_tokens: Arc<RwLock<HashMap<AccessCode, AdditionalData>>>,
    refresh_tokens: Arc<RwLock<HashMap<RefreshCode, AdditionalData>>>,
}

#[derive(Clone, Debug, Deref, Display, Eq, From, Hash, PartialEq)]
#[from(forward)]
struct ClientId(String);

#[derive(Clone, Debug, Deref, Display, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
struct AuthCode(Uuid);

#[derive(Clone, Debug, Deref, Display, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
struct AccessCode(Uuid);

#[derive(Clone, Debug, Deref, Display, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
struct RefreshCode(Uuid);

impl Default for AuthCode {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AccessCode {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RefreshCode {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

/// PKCE challenge (RFC 7636) attached to an authorization code,
/// checked against the `code_verifier` sent at the token endpoint.
///
/// Note: Only the `S256` method is supported;
///       `plain` is rejected at `/authorize` time since it provides
///       no real protection over a bare authorization code.
#[derive(Clone, Debug)]
struct PkceChallenge {
    challenge: String,
}

impl PkceChallenge {
    /// Returns `true` if `verifier` hashes to this challenge.
    fn verify(&self, verifier: &str) -> bool {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let digest = Sha256::digest(verifier.as_bytes());
        let computed = URL_SAFE_NO_PAD.encode(digest);
        computed == self.challenge
    }
}

#[derive(Clone, Debug)]
struct AdditionalData {
    created_at: DateTime<Utc>,
    client_id: ClientId,
    pkce: Option<PkceChallenge>,
}

impl AdditionalData {
    fn new(client_id: impl Into<ClientId>) -> Self {
        Self::with_pkce(client_id, None)
    }

    fn with_pkce(client_id: impl Into<ClientId>, pkce: Option<PkceChallenge>) -> Self {
        let created_at = Utc::now();
        Self {
            created_at,
            client_id: client_id.into(),
            pkce,
        }
    }

    /// Whether this entry has outlived `ttl_secs`, counted from issuance.
    fn is_older_than(&self, ttl_secs: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.created_at);
        age.num_seconds() >= ttl_secs as i64
    }

    /// Whether this entry's access token has outlived
    /// [`ACCESS_TOKEN_EXPIRES_IN`], counted from issuance.
    fn is_expired(&self) -> bool {
        self.is_older_than(ACCESS_TOKEN_EXPIRES_IN)
    }
}

/// Query parameters of /authorize API call
#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    #[allow(dead_code)]
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// Authorization HTML
#[derive(Template)]
#[template(path = "mcp_oauth_authorize.html")]
struct AuthorizeTemplate {
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String,
    scopes: String,
    code_challenge: String,
    code_challenge_method: String,
}

/// Query parameters of /approve API call
#[derive(Debug, Deserialize)]
pub struct ApprovalForm {
    client_id: String,
    redirect_uri: String,
    state: String,
    approved: String,
    #[serde(default)]
    code_challenge: String,
    // Round-tripped through the hidden form field for completeness,
    // but not read: `authorize` already rejected anything other than `S256`
    // (or no method at all) before this form could ever be rendered.
    #[allow(dead_code)]
    #[serde(default)]
    code_challenge_method: String,
}

const ACCESS_TOKEN_EXPIRES_IN: u64 = 3600;

/// How long an authorization code stays redeemable, per RFC 6749 §4.1.2
/// ("MUST expire shortly after it is issued"). Also the single-use cutoff:
/// a code is removed on first (successful or expired) use.
const AUTH_CODE_EXPIRES_IN: u64 = 600;

/// How long a refresh token stays usable. Unlike access tokens, refresh
/// tokens are meant to be long-lived, so this is measured in days rather
/// than hours (30 days).
const REFRESH_TOKEN_EXPIRES_IN: u64 = 30 * 24 * 60 * 60;

#[derive(Clone, Debug, Serialize)]
struct TokenAnswer {
    token_type: String,
    access_token: String,
    expires_in: u64,
    refresh_token: String,
}

impl TokenAnswer {
    fn new(access_token: AccessCode, refresh_token: RefreshCode) -> Self {
        let conv = |token: Uuid| token.as_simple().to_string();
        Self {
            token_type: "Bearer".to_owned(),
            access_token: conv(*access_token),
            expires_in: ACCESS_TOKEN_EXPIRES_IN,
            refresh_token: conv(*refresh_token),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    #[serde(default)]
    code: String,
    #[allow(dead_code)]
    #[serde(default)]
    client_id: String,
    #[allow(dead_code)]
    #[serde(default)]
    client_secret: String,
    #[allow(dead_code)]
    #[serde(default)]
    redirect_uri: String,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    refresh_token: String,
}

impl ClientId {
    fn from_config(config: OAuthClientConfig) -> (Self, OAuthClientConfig) {
        let client_id = config.client_id.clone().into();
        (client_id, config)
    }
}

impl McpOAuthStore {
    pub fn new(base_url: Url, clients: McpClientsConfig) -> Self {
        let clients = clients
            .clients
            .into_iter()
            .map(|client| {
                ClientId::from_config(
                    OAuthClientConfig::new(client.name, client.redirect_uri.to_string())
                        .with_client_secret(client.secret.expose_secret())
                        .with_scopes(vec!["MCP".to_string()]),
                )
            })
            .collect();

        Self {
            base_url,
            clients: Arc::new(RwLock::new(clients)),
            auth_sessions: Arc::new(RwLock::new(HashMap::new())),
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// This server's own base URL without a trailing slash,
    /// e.g. `https://mcp.example.com`.
    fn base_url(&self) -> String {
        self.base_url.origin().ascii_serialization()
    }

    /// `Host`/`issuer` value derived from the base URL,
    /// e.g. `mcp.example.com` or `127.0.0.1:9100`.
    fn issuer_host(&self) -> String {
        match self.base_url.port() {
            Some(port) => format!(
                "{host}:{port}",
                host = self.base_url.host_str().unwrap_or_default()
            ),
            None => self.base_url.host_str().unwrap_or_default().to_owned(),
        }
    }

    async fn client_registered<'a>(
        &'a self,
        client_id: impl Into<ClientId>,
    ) -> Option<OAuthClientConfig> {
        let clients = self.clients.read().await;
        clients.get(&client_id.into()).map(|c| c.to_owned())
    }

    async fn gen_auth_code(
        &self,
        client_id: impl Into<ClientId>,
        pkce: Option<PkceChallenge>,
    ) -> AuthCode {
        let data = AdditionalData::with_pkce(client_id, pkce);
        let code = AuthCode::default();

        let mut auth_sessions = self.auth_sessions.write().await;
        auth_sessions.insert(code.clone(), data);

        code
    }

    /// Validates and consumes an authorization code: it's removed from the
    /// store either way, since a code is single-use per RFC 6749 §4.1.2 and
    /// must not be redeemable again after an expired or failed attempt.
    async fn validate_auth_code(&self, code: &str) -> Option<AdditionalData> {
        let Ok(code) = code.parse::<AuthCode>() else {
            tracing::warn!("invalid authorization code: {}", code);
            return None;
        };

        let mut auth_sessions = self.auth_sessions.write().await;
        let data = auth_sessions.remove(&code)?;

        if data.is_older_than(AUTH_CODE_EXPIRES_IN) {
            tracing::debug!("authorization code expired: {}", *code);
            return None;
        }

        Some(data)
    }

    async fn gen_access_token(&self, client_id: impl Into<ClientId>) -> TokenAnswer {
        let data = AdditionalData::new(client_id);
        let access_token = AccessCode::default();
        let refresh_token = RefreshCode::default();

        let (mut access_tokens, mut refresh_tokens) =
            tokio::join!(self.access_tokens.write(), self.refresh_tokens.write());
        access_tokens.insert(access_token.clone(), data.clone());
        refresh_tokens.insert(refresh_token.clone(), data);

        TokenAnswer::new(access_token, refresh_token)
    }

    async fn validate_access_token(&self, token: &str) -> Option<AdditionalData> {
        let Ok(token) = token.parse::<AccessCode>() else {
            tracing::warn!("invalid access token: {}", token);
            return None;
        };

        let mut access_tokens = self.access_tokens.write().await;
        let data = access_tokens.get(&token)?;

        if data.is_expired() {
            tracing::debug!("access token expired: {}", *token);
            access_tokens.remove(&token);
            return None;
        }

        Some(data.to_owned())
    }

    /// Removes access tokens older than [`ACCESS_TOKEN_EXPIRES_IN`].
    ///
    /// Refresh tokens are unaffected — they're long-lived by design.
    async fn cleanup_expired_access_tokens(&self) {
        let mut access_tokens = self.access_tokens.write().await;
        let before = access_tokens.len();
        access_tokens.retain(|_, data| !data.is_expired());
        let removed = before - access_tokens.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired access token(s)");
        }
    }

    /// Removes authorization codes older than [`AUTH_CODE_EXPIRES_IN`].
    ///
    /// Codes that *are* redeemed are already removed by
    /// [`Self::validate_auth_code`] on use; this only catches codes a client
    /// never came back for, so the map doesn't grow unbounded.
    async fn cleanup_expired_auth_codes(&self) {
        let mut auth_sessions = self.auth_sessions.write().await;
        let before = auth_sessions.len();
        auth_sessions.retain(|_, data| !data.is_older_than(AUTH_CODE_EXPIRES_IN));
        let removed = before - auth_sessions.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired authorization code(s)");
        }
    }

    /// Removes refresh tokens older than [`REFRESH_TOKEN_EXPIRES_IN`].
    ///
    /// Expired refresh tokens are already rejected on use by
    /// [`Self::validate_refresh_token`]; this only catches tokens whose
    /// client never came back to use them, so the map doesn't grow
    /// unbounded.
    async fn cleanup_expired_refresh_tokens(&self) {
        let mut refresh_tokens = self.refresh_tokens.write().await;
        let before = refresh_tokens.len();
        refresh_tokens.retain(|_, data| !data.is_older_than(REFRESH_TOKEN_EXPIRES_IN));
        let removed = before - refresh_tokens.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired refresh token(s)");
        }
    }

    /// Periodically sweeps expired access tokens, authorization codes, and
    /// refresh tokens until `shutdown` fires.
    ///
    /// This is a housekeeping task, not a security boundary: expired
    /// entries are already rejected on use by [`Self::validate_access_token`],
    /// [`Self::validate_auth_code`], and [`Self::validate_refresh_token`]. It
    /// just keeps the maps from growing unbounded between requests.
    pub async fn background_cleanup(&self, shutdown: CancellationToken) {
        let mut interval = time::interval(time::Duration::from_secs(AUTH_CODE_EXPIRES_IN));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.cleanup_expired_access_tokens().await;
                    self.cleanup_expired_auth_codes().await;
                    self.cleanup_expired_refresh_tokens().await;
                }
                () = shutdown.cancelled() => {
                    tracing::debug!("stopping MCP OAuth cleanup task");
                    break;
                }
            }
        }
    }

    /// Validates a refresh token, rejecting (and removing) it if expired.
    ///
    /// Unlike authorization codes, a refresh token is *not* consumed here —
    /// it stays valid for repeated use until it expires, per the existing
    /// `refresh_token` grant behavior.
    async fn validate_refresh_token(&self, token: &str) -> Option<AdditionalData> {
        let Ok(token) = token.parse::<RefreshCode>() else {
            tracing::warn!("invalid refresh token: {}", token);
            return None;
        };

        let mut refresh_tokens = self.refresh_tokens.write().await;
        let data = refresh_tokens.get(&token)?;

        if data.is_older_than(REFRESH_TOKEN_EXPIRES_IN) {
            tracing::debug!("refresh token expired: {}", *token);
            refresh_tokens.remove(&token);
            return None;
        }

        Some(data.to_owned())
    }
}

pub async fn auth_server(State(state): State<Arc<McpOAuthStore>>) -> impl IntoResponse {
    tracing::debug!("client fetches metadata of authentication server");

    let base_url = state.base_url();
    let additional_fields = HashMap::from([("response_types_supported".into(), json!(["code"]))]);

    let mut metadata = AuthorizationMetadata::default();
    metadata.registration_endpoint = Some(format!("{base_url}/register"));
    metadata.authorization_endpoint = format!("{base_url}/authorize");
    metadata.token_endpoint = format!("{base_url}/token");
    metadata.scopes_supported = Some(vec!["MCP".to_owned()]);
    metadata.response_types_supported = Some(vec!["code".to_owned()]);
    metadata.code_challenge_methods_supported = Some(vec!["S256".to_owned()]);
    metadata.issuer = Some(state.issuer_host());
    metadata.additional_fields = additional_fields;

    tracing::debug!("metadata: {:?}", metadata);
    (StatusCode::OK, Json(metadata))
}

/// OAuth 2.0 Protected Resource Metadata (RFC 9728),
/// served under `/.well-known/oauth-protected-resource*`
/// so MCP clients can discover which authorization server(s)
/// to use for this resource.
#[derive(Debug, Serialize)]
struct ProtectedResourceMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

pub async fn protected_resource(State(state): State<Arc<McpOAuthStore>>) -> impl IntoResponse {
    tracing::debug!("client fetches protected-resource metadata");

    let base_url = state.base_url();
    let metadata = ProtectedResourceMetadata {
        resource: format!("{base_url}/mcp"),
        authorization_servers: vec![base_url],
        bearer_methods_supported: vec!["header".to_owned()],
        scopes_supported: vec!["MCP".to_owned()],
    };

    (StatusCode::OK, Json(metadata))
}

pub async fn authorize(
    Query(params): Query<AuthorizeQuery>,
    State(state): State<Arc<McpOAuthStore>>,
) -> impl IntoResponse {
    tracing::debug!("client asks for user authorization");

    // check if client is registered
    let client = &params.client_id;
    let Some(client) = state.client_registered(client).await else {
        tracing::warn!("client {client} not registered, skipping authorize rendering");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unregistered client id"
            })),
        )
            .into_response();
    };

    // compare redirect uris
    if client.redirect_uri != params.redirect_uri {
        tracing::warn!(
            "client {client} registered with different redirect uri, skipping authorize rendering",
            client = client.client_id
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unregistered redirect uri"
            })),
        )
            .into_response();
    }

    // check response type
    if params.response_type != "code" {
        tracing::warn!(
            "client {client} wants to use unsupported response type {response}, skipping authorize rendering",
            client = client.client_id,
            response = params.response_type
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unsupported response type"
            })),
        )
            .into_response();
    }

    // reject `plain` PKCE (and the implicit "plain" default when no method is
    // given at all): it offers no protection over a bare authorization code,
    // so only S256 is accepted.
    if params.code_challenge.is_some() && params.code_challenge_method.as_deref() != Some("S256") {
        tracing::warn!(
            "client {client} used unsupported code_challenge_method {method:?}, rejecting",
            client = client.client_id,
            method = params.code_challenge_method
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "only S256 code_challenge_method is supported"
            })),
        )
            .into_response();
    }

    // render HTML
    let template = AuthorizeTemplate {
        client_id: params.client_id,
        redirect_uri: params.redirect_uri,
        scope: params.scope.clone().unwrap_or_default(),
        state: params.state.clone().unwrap_or_default(),
        scopes: params
            .scope
            .clone()
            .unwrap_or_else(|| "(no scope)".to_string()),
        code_challenge: params.code_challenge.clone().unwrap_or_default(),
        code_challenge_method: params.code_challenge_method.clone().unwrap_or_default(),
    };
    Html(template.render().unwrap()).into_response()
}

pub async fn approve(
    State(state): State<Arc<McpOAuthStore>>,
    Form(form): Form<ApprovalForm>,
) -> impl IntoResponse {
    let mut redirect_url = if form.approved == "true" {
        tracing::info!("user approved the authorization request");
        // `code_challenge_method` was already validated as `S256` (or absent)
        // in `authorize`; the hidden form field just carries it through the
        // redirect to here, so it's not re-checked.
        let pkce = if form.code_challenge.is_empty() {
            None
        } else {
            Some(PkceChallenge {
                challenge: form.code_challenge.clone(),
            })
        };
        let auth_code = state.gen_auth_code(form.client_id, pkce).await;
        format!(
            "{uri}?code={code}",
            uri = form.redirect_uri,
            code = auth_code.as_simple()
        )
    } else {
        tracing::warn!("user rejected the authorization request");
        format!("{uri}?error=access_denied", uri = form.redirect_uri)
    };
    if !form.state.is_empty() {
        redirect_url.push_str("&state=");
        redirect_url.push_str(&form.state);
    }
    tracing::debug!("redirecting to: {}", redirect_url);
    Redirect::to(&redirect_url).into_response()
}

pub async fn gen_access_token(
    State(state): State<Arc<McpOAuthStore>>,
    request: Request<Body>,
) -> impl IntoResponse {
    tracing::debug!("client requests an access token");
    let request = match body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => {
            tracing::debug!("request body: {}", String::from_utf8_lossy(&bytes));
            bytes
        }
        Err(e) => {
            tracing::error!("cannot read request body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": "can't read request body"
                })),
            )
                .into_response();
        }
    };

    let request = match serde_urlencoded::from_bytes::<TokenRequest>(&request) {
        Ok(form) => {
            tracing::debug!("successfully parsed form data: {:?}", form);
            form
        }
        Err(e) => {
            tracing::error!("cannot parse form data: {}", e);
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": format!("can't parse form data: {}", e)
                })),
            )
                .into_response();
        }
    };

    if request.client_id.is_empty() {
        tracing::error!("empty client id detected");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_client",
                "error_description": "invalid client id"
            })),
        )
            .into_response();
    }

    let Some(client) = state.client_registered(&request.client_id).await else {
        tracing::warn!("invalid client id: {}", request.client_id);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_client",
                "error_description": "unregistered client id"
            })),
        )
            .into_response();
    };

    if let Some(client_secret) = client.client_secret {
        if request.client_secret != client_secret {
            tracing::warn!("invalid secret for client {}", client.client_id);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client",
                    "error_description": "invalid secret"
                })),
            )
                .into_response();
        }
    } else {
        tracing::warn!("skipping secret comparison for client {}", client.client_id);
    }

    match request.grant_type.as_str() {
        "authorization_code" => {
            let auth_code = &request.code;
            let Some(data) = state.validate_auth_code(auth_code).await else {
                tracing::warn!("invalid authorization code: {}", auth_code);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "invalid authorization code"
                    })),
                )
                    .into_response();
            };

            if let Some(pkce) = &data.pkce {
                let verified = request
                    .code_verifier
                    .as_deref()
                    .is_some_and(|verifier| pkce.verify(verifier));
                if !verified {
                    tracing::warn!("PKCE verification failed for client {}", data.client_id);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "invalid_grant",
                            "error_description": "invalid code_verifier"
                        })),
                    )
                        .into_response();
                }
            }

            let token = state.gen_access_token(data.client_id).await;
            match serde_json::to_value(token) {
                Ok(token) => {
                    tracing::info!("successfully created access token");
                    (StatusCode::OK, Json(token)).into_response()
                }
                Err(e) => {
                    tracing::error!("failed to create access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to create access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        "refresh_token" => {
            let refresh_token = &request.refresh_token;
            let Some(data) = state.validate_refresh_token(refresh_token).await else {
                tracing::warn!("invalid refresh token: {}", refresh_token);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "invalid refresh token"
                    })),
                )
                    .into_response();
            };

            let token = state.gen_access_token(data.client_id).await;
            match serde_json::to_value(token) {
                Ok(token) => {
                    tracing::info!("successfully recreated access token");
                    (StatusCode::OK, Json(token)).into_response()
                }
                Err(e) => {
                    tracing::error!("failed to recreate access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to recreate access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        _ => {
            tracing::warn!("unsupported grant type: {}", request.grant_type);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "unsupported_grant_type",
                    "error_description": "only authorization_code is supported"
                })),
            )
                .into_response()
        }
    }
}

/// Builds a 401 response carrying a `WWW-Authenticate` header that points
/// clients at this resource's protected-resource metadata (RFC 9728 §5.1),
/// so they know where to discover the authorization server to use.
fn unauthorized(state: &McpOAuthStore) -> Response {
    let metadata_url = format!(
        "{}/.well-known/oauth-protected-resource/mcp",
        state.base_url()
    );
    (
        StatusCode::UNAUTHORIZED,
        [(
            axum::http::header::WWW_AUTHENTICATE,
            format!(r#"Bearer resource_metadata="{metadata_url}""#),
        )],
    )
        .into_response()
}

pub async fn validate_access_token(
    State(token_store): State<Arc<McpOAuthStore>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    tracing::debug!("validate_token_middleware");
    let auth_header = request.headers().get("Authorization");
    let token = match auth_header {
        Some(header) => {
            let header_str = header.to_str().unwrap_or("");
            if let Some(stripped) = header_str.strip_prefix("Bearer ") {
                stripped.to_string()
            } else {
                tracing::warn!("incomplete auth header");
                return unauthorized(&token_store);
            }
        }
        None => {
            tracing::warn!("missing auth header");
            return unauthorized(&token_store);
        }
    };
    tracing::debug!("token: {token}");

    match token_store.validate_access_token(&token).await {
        Some(data) => {
            tracing::info!("valid access token (client {})", data.client_id);
            next.run(request).await
        }
        None => {
            tracing::warn!("invalid access token");
            unauthorized(&token_store)
        }
    }
}
