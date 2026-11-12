// ORDO's glue to MapLibre GL JS and PMTiles (ADR-0006, MAP-1, MAP-2).
//
// The Rust app imports this module only when the situation map opens. It loads the
// vendored libraries (map/vendor), shows the incident's map region from the device's
// offline copy or from the server (with a ticket, ADR-0062), and draws ORDO's objects, which the app passes as GeoJSON. Everything
// the map shows is also in the app's accessible list (A11Y-5). Points with a `symbol`
// property show that tactical symbol of the profile (MAP-3), the others a circle.
//
// [impl->dsn~situation-map~1]
// [impl->dsn~tactical-symbols~1]

// One region per incident, named by the incident's ULID; read with a ticket.
const regionPath = (incident) => `/tiles/${incident}.pmtiles`;
const authorised = (ticket) => new Headers({ Authorization: `Bearer ${ticket}` });
const OBJECTS = "ordo-objects";
// The shape being drawn (MAP-4), before it is saved.
const DRAFT = "ordo-draft";
// Line patterns of drawings (profile map/drawings.toml): categories differ by pattern and
// label, not by colour alone.
const DASHES = { solid: null, dashed: [3, 2], dotted: [0.5, 1.5] };
// Germany, until the incident is placed.
const START = { center: [10.45, 51.16], zoom: 5 };
// Tactical symbols: CSS pixels high on the map, drawn at twice that for sharp screens.
const SYMBOL_HEIGHT = 32;
const SYMBOL_SCALE = 2;

let libs;

function script(src) {
  return new Promise((resolve, reject) => {
    if (document.querySelector(`script[src="${src}"]`)) return resolve();
    const el = document.createElement("script");
    el.src = src;
    el.onload = resolve;
    el.onerror = () => reject(new Error(`cannot load ${src}`));
    document.head.append(el);
  });
}

async function load() {
  if (libs) return libs;
  // MapLibre's styles are imported by app.css into the cascade layer `vendor`.
  const maplibre = await import("/map/vendor/maplibre-gl.mjs");
  await script("/map/vendor/pmtiles.js");
  await script("/map/vendor/basemaps.js");
  const protocol = new globalThis.pmtiles.Protocol();
  maplibre.addProtocol("pmtiles", protocol.tile);
  libs = { maplibre, pmtiles: globalThis.pmtiles, basemaps: globalThis.basemaps, protocol };
  return libs;
}

// The device's copy lives in Cache Storage: private to ORDO's origin like OPFS, and
// writable from the page in every browser (Safari cannot write OPFS files outside workers).
const OFFLINE_CACHE = "ordo-map-region";

async function offlineFile(incident) {
  if (!incident) return null;
  try {
    const cache = await caches.open(OFFLINE_CACHE);
    const response = await cache.match(regionPath(incident));
    if (!response) return null;
    return new File([await response.blob()], `ordo-${incident}.pmtiles`);
  } catch {
    return null;
  }
}

// Whether the server has the incident's region and the ticket lets this device read it.
async function onServer(incident, ticket) {
  if (!incident || !ticket) return false;
  try {
    return (await fetch(regionPath(incident), { method: "HEAD", headers: authorised(ticket) })).ok;
  } catch {
    return false; // offline
  }
}

// Where the incident's map region comes from: the device's copy, the server, or nowhere.
async function region({ pmtiles, protocol }, incident, ticket) {
  const file = await offlineFile(incident);
  if (file && file.size > 0) {
    const archive = new pmtiles.PMTiles(new pmtiles.FileSource(file));
    protocol.add(archive);
    return { url: `pmtiles://${file.name}`, from: "device", archive };
  }
  if (await onServer(incident, ticket)) {
    const url = `${location.origin}${regionPath(incident)}`;
    const archive = new pmtiles.PMTiles(new pmtiles.FetchSource(url, authorised(ticket)));
    protocol.add(archive);
    return { url: `pmtiles://${url}`, from: "server", archive };
  }
  return { url: null, from: "none" };
}

function token(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/**
 * Creates the map in `element` with the region of `incident` (its ULID), read from the
 * server with `ticket` if the device has no copy; returns a handle, or throws (e.g. no
 * WebGL).
 */
export async function create(element, centre, label, incident, ticket) {
  const l = await load();
  const source = await region(l, incident, ticket);
  const style = {
    version: 8,
    glyphs: `${location.origin}/map/glyphs/{fontstack}/{range}.pbf`,
    sources: {},
    layers: [{ id: "background", type: "background", paint: { "background-color": token("--ordo-color-surface-sunken") } }],
  };
  if (source.url) {
    style.sources.protomaps = {
      type: "vector",
      url: source.url,
      attribution: "© <a href=\"https://openstreetmap.org/copyright\">OpenStreetMap</a> contributors",
    };
    style.layers = l.basemaps.layers("protomaps", l.basemaps.namedFlavor("light"), { lang: "de" });
  }
  const map = new l.maplibre.Map({
    container: element,
    style,
    center: centre ?? START.center,
    zoom: centre ? 15 : START.zoom,
    attributionControl: { compact: false },
  });
  map.getCanvas().setAttribute("aria-label", label);
  map.addControl(new l.maplibre.NavigationControl({ showCompass: false }));
  await new Promise((resolve) => map.once("load", resolve));
  // Nothing placed yet: open on the map region, the incident's area, not on all Germany.
  if (!centre && source.archive) {
    try {
      const h = await source.archive.getHeader();
      map.fitBounds([[h.minLon, h.minLat], [h.maxLon, h.maxLat]], { padding: 24, animate: false });
    } catch {
      // no bounds in the archive: keep the start view
    }
  }
  const accent = token("--ordo-color-action-primary-background");
  const text = token("--ordo-color-text-primary");
  const halo = token("--ordo-color-surface-default");
  map.addSource(OBJECTS, { type: "geojson", data: { type: "FeatureCollection", features: [] } });
  map.addLayer({ id: "ordo-areas", type: "fill", source: OBJECTS, filter: ["==", ["geometry-type"], "Polygon"], paint: { "fill-color": accent, "fill-opacity": 0.2 } });
  for (const [style, dash] of Object.entries(DASHES)) {
    const paint = { "line-color": accent, "line-width": 3, ...(dash ? { "line-dasharray": dash } : {}) };
    map.addLayer({
      id: `ordo-lines-${style}`,
      type: "line",
      source: OBJECTS,
      filter: ["all", ["!=", ["geometry-type"], "Point"], ["==", ["coalesce", ["get", "line"], "solid"], style]],
      layout: { "line-cap": style === "dotted" ? "round" : "butt" },
      paint,
    });
  }
  const point = ["==", ["geometry-type"], "Point"];
  map.addLayer({ id: "ordo-points", type: "circle", source: OBJECTS, filter: ["all", point, ["!", ["has", "symbol"]]], paint: { "circle-radius": 8, "circle-color": accent, "circle-stroke-width": 2, "circle-stroke-color": halo } });
  // Symbols never hide each other: on scene, every unit counts.
  map.addLayer({
    id: "ordo-symbols",
    type: "symbol",
    source: OBJECTS,
    filter: ["all", point, ["has", "symbol"]],
    layout: { "icon-image": ["get", "symbol"], "icon-allow-overlap": true, "icon-ignore-placement": true },
  });
  map.addLayer({
    id: "ordo-labels",
    type: "symbol",
    source: OBJECTS,
    layout: {
      "text-field": ["get", "label"],
      "text-font": ["Noto Sans Medium"],
      "text-size": 14,
      "text-offset": ["case", ["has", "symbol"], ["literal", [0, SYMBOL_HEIGHT / 2 / 14 + 0.3]], ["literal", [0, 1.2]]],
      "text-anchor": "top",
    },
    paint: { "text-color": text, "text-halo-color": halo, "text-halo-width": 2 },
  });
  map.addSource(DRAFT, { type: "geojson", data: { type: "FeatureCollection", features: [] } });
  map.addLayer({ id: "ordo-draft-line", type: "line", source: DRAFT, filter: ["!=", ["geometry-type"], "Point"], paint: { "line-color": text, "line-width": 2, "line-dasharray": [2, 1] } });
  map.addLayer({ id: "ordo-draft-corners", type: "circle", source: DRAFT, filter: ["==", ["geometry-type"], "Point"], paint: { "circle-radius": 5, "circle-color": halo, "circle-stroke-width": 2, "circle-stroke-color": text } });
  return { map, from: source.from };
}

/** Shows the shape being drawn (a GeoJSON FeatureCollection as text; empty to clear). */
export function setDraft(handle, geojson) {
  handle.map.getSource(DRAFT)?.setData(JSON.parse(geojson));
}

/** Gives the size an SVG's viewBox implies, at `SYMBOL_HEIGHT` CSS pixels high, so every
 * browser draws it the same (an SVG without width and height has no size of its own). */
function sized(svg, scale) {
  const box = /viewBox="[\d.-]+ [\d.-]+ ([\d.]+) ([\d.]+)"/.exec(svg);
  const [w, h] = box ? [Number(box[1]), Number(box[2])] : [1, 1];
  const height = SYMBOL_HEIGHT * scale;
  const width = Math.round((height * w) / h);
  return svg.replace("<svg ", `<svg width="${width}" height="${height}" `);
}

/** Adds the profile's tactical symbols (a JSON object: symbol ID → SVG text) as images. */
export async function setSymbols(handle, json) {
  const symbols = JSON.parse(json);
  await Promise.all(
    Object.entries(symbols).map(async ([id, svg]) => {
      if (handle.map.hasImage(id)) return;
      const image = new Image();
      image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(sized(svg, SYMBOL_SCALE))}`;
      try {
        await image.decode();
        handle.map.addImage(id, image, { pixelRatio: SYMBOL_SCALE });
      } catch {
        // without this image the object shows as a circle
      }
    }),
  );
  // How many symbols the map has, for support and tests.
  handle.map.getContainer().dataset.symbols = String(Object.keys(symbols).filter((id) => handle.map.hasImage(id)).length);
}

/** Moves the map to the device's position, asked once (browser geolocation), without an
 *  animation, so objects are placed there right away; fails with "denied" or "unavailable". */
export function locate(handle) {
  return new Promise((resolve, reject) => {
    if (!navigator.geolocation) return reject("unavailable");
    navigator.geolocation.getCurrentPosition(
      (p) => {
        const at = [p.coords.longitude, p.coords.latitude];
        handle.map.jumpTo({ center: at, zoom: Math.max(handle.map.getZoom(), 16) });
        resolve(at);
      },
      (e) => reject(e.code === e.PERMISSION_DENIED ? "denied" : "unavailable"),
      { enableHighAccuracy: true, timeout: 20000, maximumAge: 60000 },
    );
  });
}

/** Shows ORDO's objects (a GeoJSON FeatureCollection as text). */
export function setObjects(handle, geojson) {
  handle.map.getSource(OBJECTS)?.setData(JSON.parse(geojson));
}

/** The centre of the map: [longitude, latitude]. */
export function centre(handle) {
  const c = handle.map.getCenter();
  return [c.lng, c.lat];
}

/** Moves the map to a point. */
export function flyTo(handle, lng, lat) {
  handle.map.flyTo({ center: [lng, lat], zoom: Math.max(handle.map.getZoom(), 15) });
}

/** Where the map region comes from: "device", "server" or "none". */
export function regionSource(handle) {
  return handle.from;
}

/** Whether the server has the incident's region, and the size of the device's copy (0: none). */
export async function offlineState(incident, ticket) {
  const file = await offlineFile(incident);
  return { server: await onServer(incident, ticket), device: file ? file.size : 0 };
}

/** Keeps the incident's whole map region on the device (Cache Storage), one per incident. */
export async function saveOffline(incident, ticket) {
  const response = await fetch(regionPath(incident), { headers: authorised(ticket) });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const cache = await caches.open(OFFLINE_CACHE);
  await cache.put(regionPath(incident), response);
  const file = await offlineFile(incident);
  return file ? file.size : 0;
}

/** Deletes the device's copy of the incident's map region. */
export async function removeOffline(incident) {
  const cache = await caches.open(OFFLINE_CACHE);
  await cache.delete(regionPath(incident));
}
