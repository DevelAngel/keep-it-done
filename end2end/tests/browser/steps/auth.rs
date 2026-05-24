use anyhow::Result;
use cucumber::{then, when};
use thirtyfour::prelude::*;

use crate::world::AppWorld;

/// Inject a fetch mock that returns a Tinyauth-style 401 for
/// every POST request (= Leptos server-function calls).
/// GET requests pass through so static assets keep working.
///
/// web-sys calls `window.fetch(Request)` with a `Request` object
/// as first argument, while plain JS uses `fetch(url, {method})`.
/// The mock must handle both calling conventions.
#[when("the auth proxy rejects server requests")]
async fn mock_auth_proxy_401(world: &mut AppWorld) -> Result<()> {
    world
        .http
        .execute(
            r#"
            const origFetch = window.fetch.bind(window);
            window.fetch = function(input, init) {
                const method = (input instanceof Request)
                    ? input.method
                    : (init && init.method);
                if (method === 'POST') {
                    return Promise.resolve(new Response(
                        '{"message":"Unauthorized","status":401}',
                        { status: 401, headers: { 'Content-Type': 'application/json' } }
                    ));
                }
                return origFetch.apply(this, arguments);
            };
            "#,
            vec![],
        )
        .await?;
    Ok(())
}

#[then("I see the session expired message")]
async fn see_session_expired(world: &mut AppWorld) -> Result<()> {
    let el = world
        .http
        .query(By::Testid("session-expired"))
        .first()
        .await?;
    let text = el.text().await?;
    assert!(
        text.contains("Session expired"),
        "expected session-expired hint, got: {text}",
    );
    Ok(())
}
