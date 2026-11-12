// [impl->dsn~offline-app-shell~2]
// ORDO service worker: keeps the app shell on the device so ORDO starts without a
// network connection (offline-first, OFF-1, UX-5). Data never passes through here:
// sync uses the WebSocket at /sync, which service workers do not intercept.
const NO_CACHE = ["/sync", "/healthz", "/readyz", "/metrics"];
// The device storage worker (ADR-0034) is started by the app, not linked from the page.
const WORKER_FILES = ["/ordo-store_loader.js", "/ordo-store.js", "/ordo-store_bg.wasm"];
// The situation map (ADR-0006): glue, libraries and label fonts, loaded when the map opens.
// The map region itself (/tiles/) is read in parts and kept by the app, not cached here.
const MAP_FILES = [
  "/map/ordo-map.js",
  "/map/vendor/maplibre-gl.mjs",
  "/map/vendor/maplibre-gl-worker.mjs",
  "/map/vendor/maplibre-gl.css",
  "/map/vendor/pmtiles.js",
  "/map/vendor/basemaps.js",
  "/map/vendor/terra-draw.js",
  "/map/vendor/terra-draw-maplibre-gl-adapter.js",
  ...["Noto%20Sans%20Regular", "Noto%20Sans%20Medium", "Noto%20Sans%20Italic"].flatMap((font) =>
    ["0-255", "256-511", "8192-8447"].map((range) => `/map/glyphs/${font}/${range}.pbf`),
  ),
];

// The typefaces (fonts/fonts.css, vendor-fonts.sh): every one, so the app reads the same
// offline, whatever text it shows first.
const FONT_FILES = ["next", "mono"].flatMap((family) =>
  ["400", "700"].flatMap((weight) =>
    ["latin", "latin-ext"].map((subset) => `/fonts/atkinson-hyperlegible-${family}-${subset}-${weight}-normal.woff2`),
  ),
);

// One cache per version of the app, named after its index.html, so a new version never
// mixes with the files of the last one. A cache counts as complete once it holds "/",
// which is stored last; until then the last complete one keeps serving.
const PREFIX = "ordo-app-shell";
// Trunk's content-hashed build files (`name-<16 hex>.ext`, `name-<16 hex>_bg.wasm`, and the
// snippets under `snippets/name-<16 hex>/`) never change under their name.
const HASHED = /[-_][0-9a-f]{16}(?:_bg\.|\.|\/)/;
// Files without a content hash are asked from the network first, but a device on a bad
// connection must not wait long for them: after this, the stored copy is used.
const NETWORK_WAIT_MS = 4000;

async function versionName(html) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(html));
  const hex = [...new Uint8Array(digest).slice(0, 8)].map((b) => b.toString(16).padStart(2, "0")).join("");
  return `${PREFIX}-${hex}`;
}

// Fetches index.html and every same-origin file it references (Trunk's hashed assets),
// with the worker, map and font files, into the cache of this version; then removes the
// caches of other versions.
async function cacheShell() {
  const response = await fetch("/", { cache: "no-store" });
  if (!response.ok) return;
  const html = await response.clone().text();
  const name = await versionName(html);
  const cache = await caches.open(name);
  if (!(await cache.match("/"))) {
    const assets = [...html.matchAll(/(?:href|src)="([^"]+)"/g)]
      .map((m) => new URL(m[1], self.location.origin))
      .filter((url) => url.origin === self.location.origin)
      .map((url) => url.pathname)
      .concat(WORKER_FILES, MAP_FILES, FONT_FILES);
    // All or nothing (Cache.addAll); the unhashed files are revalidated, not taken from
    // the browser's HTTP cache unchecked.
    await cache.addAll(assets.map((path) => new Request(path, { cache: HASHED.test(path) ? "default" : "no-cache" })));
    await cache.put("/", response);
  }
  for (const other of await caches.keys()) {
    if (other !== name && other.startsWith(PREFIX)) await caches.delete(other);
  }
}

// The stored copy from a complete version cache (there is one after cleanup).
async function stored(request) {
  for (const name of await caches.keys()) {
    if (!name.startsWith(PREFIX)) continue;
    const cache = await caches.open(name);
    if (!(await cache.match("/"))) continue;
    const hit = await cache.match(request);
    if (hit) return hit;
  }
  return undefined;
}

// The network's answer, or the stored copy when the network fails or takes too long.
async function networkFirst(request) {
  const network = fetch(request);
  network.catch(() => {}); // a late failure after the timeout is no error
  const timeout = new Promise((resolve) => setTimeout(resolve, NETWORK_WAIT_MS, undefined));
  try {
    const answer = await Promise.race([network, timeout]);
    if (answer) return answer;
  } catch {
    // offline: use the stored copy
  }
  return (await stored(request)) ?? network;
}

self.addEventListener("install", (event) => {
  event.waitUntil(cacheShell().catch(() => {}).then(() => self.skipWaiting()));
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
    url.pathname.startsWith("/.well-known/") || // versions: always the server's own answer
    url.pathname.startsWith("/blobs/") || // attachments: with a ticket, never cached
    url.pathname.startsWith("/tiles/") // the map region: range requests, kept by the app
  ) {
    return;
  }
  if (request.mode === "navigate") {
    // Network first, so a new version is picked up; the stored shell when offline.
    event.respondWith(
      fetch(request)
        .then((response) => {
          if (response.ok) event.waitUntil(cacheShell().catch(() => {}));
          return response;
        })
        .catch(() => stored("/")),
    );
    return;
  }
  // Hashed files never change: stored first. Every other file (the storage worker, the
  // map's glue and libraries, fonts) belongs to the version the page was loaded with, so
  // it comes from the network while there is one: an old stored copy must never run with
  // a new app.
  if (HASHED.test(url.pathname)) {
    event.respondWith(stored(request).then((hit) => hit ?? fetch(request)));
  } else {
    event.respondWith(networkFirst(request));
  }
});
