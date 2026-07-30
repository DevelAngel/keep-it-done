//! Minimal OAuth 2.0 authorization-code + PKCE client for the e2e harness,
//! just enough to get a bearer token for the MCP server (see
//! `docs/adr/rmcp-mcp-server.md`). No browser is involved: the redirect_uri
//! in `mcp-clients.test.toml` is never actually served — this harness
//! captures the authorization code straight from the `Location` header of
//! the `/oauth/approve` response instead of following a real redirect.

use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use std::net::SocketAddr;

use crate::helpers::{OAUTH_CLIENT_ID, OAUTH_CLIENT_SECRET, OAUTH_REDIRECT_URI};

/// Runs the full authorization-code + PKCE flow against `mcp_addr` and
/// returns a bearer access token, ready to use as the MCP transport's
/// `auth_header`.
pub async fn fetch_access_token(mcp_addr: SocketAddr) -> Result<String> {
    // PKCE verifier: two concatenated UUIDv4s (64 chars), well within the
    // 43-128 char range required by RFC 7636.
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    // Approve directly instead of going through `/authorize`'s HTML consent
    // page first: `/oauth/approve` doesn't re-validate client registration
    // itself, and there's no real user to click "approve" here anyway.
    let no_redirect = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let response = no_redirect
        .post(format!("http://{mcp_addr}/oauth/approve"))
        .form(&[
            ("client_id", OAUTH_CLIENT_ID),
            ("redirect_uri", OAUTH_REDIRECT_URI),
            ("state", "e2e"),
            ("approved", "true"),
            ("code_challenge", &challenge),
        ])
        .send()
        .await
        .context("POST /oauth/approve")?;

    let location = response
        .headers()
        .get("location")
        .context("/oauth/approve did not redirect")?
        .to_str()
        .context("non-ASCII Location header")?
        .to_owned();

    let code = url::Url::parse(&location)
        .or_else(|_| url::Url::parse(&format!("http://placeholder{location}")))
        .context("parsing /oauth/approve redirect Location")?
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .context("no `code` in /oauth/approve redirect")?;

    let token: serde_json::Value = reqwest::Client::new()
        .post(format!("http://{mcp_addr}/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("redirect_uri", OAUTH_REDIRECT_URI),
            ("code_verifier", &verifier),
        ])
        .send()
        .await
        .context("POST /token")?
        .error_for_status()
        .context("/token returned an error")?
        .json()
        .await
        .context("parsing /token response")?;

    match token.get("access_token").and_then(|v| v.as_str()) {
        Some(access_token) => Ok(access_token.to_owned()),
        None => bail!("/token response had no access_token: {token}"),
    }
}
