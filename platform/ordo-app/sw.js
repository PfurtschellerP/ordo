// [impl->dsn~offline-app-shell~1]
// ORDO service worker: keeps the app shell on the device so ORDO starts without a
// network connection (offline-first, OFF-1, UX-5). Data never passes through here:
// sync uses the WebSocket at /sync, which service workers do not intercept.
const CACHE = "ordo-app-shell";
const NO_CACHE = ["/sync", "/healthz", "/readyz", "/metrics"];
// The device storage worker (ADR-0034) is started by the app, not linked from the page.
const WORKER_FILES = ["/ordo-store_loader.js", "/ordo-store.js", "/ordo-store_bg.wasm"];

// Fetches index.html and every same-origin file it references (Trunk's hashed
// assets), and stores them. Files no longer referenced are removed.
async function cacheShell() {
  const cache = await caches.open(CACHE);
  const response = await fetch("/", { cache: "no-store" });
  if (!response.ok) return;
  const html = await response.clone().text();
  const assets = [...html.matchAll(/(?:href|src)="([^"]+)"/g)]
    .map((m) => new URL(m[1], self.location.origin))
    .filter((url) => url.origin === self.location.origin)
    .map((url) => url.pathname)
    .concat(WORKER_FILES);
  await cache.put("/", response);
  await cache.addAll(assets);
  const keep = new Set(["/", ...assets]);
  for (const request of await cache.keys()) {
    if (!keep.has(new URL(request.url).pathname)) await cache.delete(request);
  }
}

self.addEventListener("install", (event) => {
  event.waitUntil(cacheShell().then(() => self.skipWaiting()));
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (
    request.method !== "GET" ||
    url.origin !== self.location.origin ||
    NO_CACHE.includes(url.pathname) ||
    url.pathname.startsWith("/blobs/") // attachments: with a ticket, never cached
  ) {
    return;
  }
  if (request.mode === "navigate") {
    // Network first, so a new version is picked up; the cached shell when offline.
    event.respondWith(
      fetch(request)
        .then((response) => {
          if (response.ok) event.waitUntil(cacheShell().catch(() => {}));
          return response;
        })
        .catch(() => caches.match("/")),
    );
    return;
  }
  // Hashed assets never change: cache first.
  event.respondWith(caches.match(request).then((cached) => cached || fetch(request)));
});
