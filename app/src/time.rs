//! Central time accessor.
//!
//! All view-rendering code must use [`now()`] and [`today()`] instead
//! of calling `Utc::now()` directly.  See ADR: central-time-accessor.

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        /// Read the time offset from Leptos context (server-side).
        ///
        /// Returns `None` when no offset is active or when called
        /// outside a Leptos reactive scope.
        fn get_offset() -> Option<TimeDelta> {
            use crate::server::ssr::SharedTimeOffset;
            leptos::context::use_context::<SharedTimeOffset>()
                .and_then(|o| o.get())
                .map(TimeDelta::seconds)
        }

        /// Return the active offset in seconds, if any.
        ///
        /// Used by [`App`](crate::App) to inject a `<meta>` tag that
        /// bridges the offset from SSR to WASM.
        pub fn active_offset_seconds() -> Option<i64> {
            use crate::server::ssr::SharedTimeOffset;
            leptos::context::use_context::<SharedTimeOffset>()
                .and_then(|o| o.get())
        }
    } else {
        /// Read the time offset from a `<meta>` tag (browser-side).
        ///
        /// The value is read once from the DOM and cached for the
        /// lifetime of the WASM module.
        fn get_offset() -> Option<TimeDelta> {
            static OFFSET: std::sync::OnceLock<Option<TimeDelta>> = std::sync::OnceLock::new();
            *OFFSET.get_or_init(|| {
                read_meta_offset().map(TimeDelta::seconds)
            })
        }

        fn read_meta_offset() -> Option<i64> {
            let window = web_sys::window()?;
            let document = window.document()?;
            let el = document
                .query_selector("meta[name='kid-time-offset-seconds']")
                .ok()??;
            el.get_attribute("content")?.parse().ok()
        }

        /// No-op on the client side — the offset is read via the
        /// `<meta>` tag, not via Leptos context.
        pub fn active_offset_seconds() -> Option<i64> {
            None
        }
    }
}

/// Current UTC time, shifted by the active offset.
///
/// When no offset is active, this returns plain `Utc::now()`.
pub fn now() -> DateTime<Utc> {
    match get_offset() {
        Some(offset) => Utc::now() + offset,
        None => Utc::now(),
    }
}

/// Current UTC date, shifted by the active offset.
pub fn today() -> NaiveDate {
    now().date_naive()
}

/// Like [`now()`], but takes an explicit offset (in seconds) instead of
/// reading it from Leptos context/the DOM.
///
/// For callers outside the Leptos reactive scope - e.g. the MCP server,
/// which reads the same `SharedTimeOffset` directly rather than through a
/// context - that don't have `now()`/`today()`'s implicit context access
/// available.
pub fn now_at_offset(offset_seconds: Option<i64>) -> DateTime<Utc> {
    match offset_seconds.map(TimeDelta::seconds) {
        Some(offset) => Utc::now() + offset,
        None => Utc::now(),
    }
}

/// Like [`today()`], but takes an explicit offset; see [`now_at_offset()`].
pub fn today_at_offset(offset_seconds: Option<i64>) -> NaiveDate {
    now_at_offset(offset_seconds).date_naive()
}
