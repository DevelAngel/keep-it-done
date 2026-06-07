---
status: proposed
date: 2026-06-07
---

# Progressive Web App (PWA) for Installable Mobile Experience

## Context and Problem Statement

The Leptos web UI is already mobile-first and responsive, but
on Android and iOS it runs as a browser tab. Users must navigate
to the URL each time, there is no home-screen icon, and the
browser chrome (address bar, tabs) wastes vertical space on
small screens. The app should feel like a native app when
launched from the home screen.

Full offline support is explicitly **not** a goal for this
iteration. The app requires a network connection to the server.

## Considered Options

- Web App Manifest + minimal Service Worker (PWA)
- Capacitor / Tauri Mobile wrapper
- No change (stay browser-only)

## Decision Outcome

Chosen option: "Web App Manifest + minimal Service Worker",
because it requires no native toolchain, no app store, and no
additional build artifact. A manifest and a stub service worker
are sufficient for the "Add to Home Screen" prompt on both
Android and iOS.

### Consequences

- Good, because users can install with one tap — no app store
  needed
- Good, because the browser chrome disappears in standalone
  mode, reclaiming screen space
- Good, because no additional build tooling or CI changes are
  required
- Bad, because iOS has PWA limitations (no push notifications
  without explicit permission flow, limited background sync)
- Neutral, because a service worker is registered but only
  serves as an install gate — caching can be added later

## More Information

The PWA consists of three static artifacts — a web app
manifest, a stub service worker, and icon files — all served
from the existing `public/` directory. No Rust code changes
are required; the Axum static file serving already covers
the new files.

Full offline support, push notifications, and app store
listings (TWA / PWABuilder) are explicitly deferred.
