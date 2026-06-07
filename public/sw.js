// Stub service worker — exists solely to satisfy PWA install criteria.
// No fetch interception, no caching. Offline support is deferred.

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
