// ORDO's glue to MapLibre GL JS and PMTiles (ADR-0006, MAP-1, MAP-2).
//
// The Rust app imports this module only when the situation map opens. It loads the
// vendored libraries (map/vendor), shows the map region from the device's offline copy or
// from the server, and draws ORDO's objects, which the app passes as GeoJSON. Everything
// the map shows is also in the app's accessible list (A11Y-5).
//
// [impl->dsn~situation-map~1]

const REGION_URL = "/tiles/region.pmtiles";
const OFFLINE_NAME = "ordo-region.pmtiles";
const OBJECTS = "ordo-objects";
// Germany, until the incident is placed.
const START = { center: [10.45, 51.16], zoom: 5 };

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
  // MapLibre's styles go into the cascade layer `vendor` (app.css), below ORDO's own, so
  // the design system (tokens, touch targets) wins over them.
  if (!document.getElementById("maplibre-css")) {
    const css = document.createElement("style");
    css.id = "maplibre-css";
    css.textContent = '@import url("/map/vendor/maplibre-gl.css") layer(vendor);';
    document.head.append(css);
  }
  const maplibre = await import("/map/vendor/maplibre-gl.mjs");
  await script("/map/vendor/pmtiles.js");
  await script("/map/vendor/basemaps.js");
  const protocol = new globalThis.pmtiles.Protocol();
  maplibre.addProtocol("pmtiles", protocol.tile);
  libs = { maplibre, pmtiles: globalThis.pmtiles, basemaps: globalThis.basemaps, protocol };
  return libs;
}

async function offlineFile() {
  try {
    const dir = await navigator.storage.getDirectory();
    const handle = await dir.getFileHandle(OFFLINE_NAME);
    return await handle.getFile();
  } catch {
    return null;
  }
}

// Where the map region comes from: the device's copy, the server, or nowhere.
async function region({ pmtiles, protocol }) {
  const file = await offlineFile();
  if (file && file.size > 0) {
    protocol.add(new pmtiles.PMTiles(new pmtiles.FileSource(file)));
    return { url: `pmtiles://${file.name}`, from: "device" };
  }
  try {
    const head = await fetch(REGION_URL, { method: "HEAD" });
    if (head.ok) return { url: `pmtiles://${location.origin}${REGION_URL}`, from: "server" };
  } catch {
    // offline and no copy on the device
  }
  return { url: null, from: "none" };
}

function token(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/** Creates the map in `element`; returns a handle, or throws (e.g. no WebGL). */
export async function create(element, centre, label) {
  const l = await load();
  const source = await region(l);
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
  const accent = token("--ordo-color-action-primary-background");
  const text = token("--ordo-color-text-primary");
  const halo = token("--ordo-color-surface-default");
  map.addSource(OBJECTS, { type: "geojson", data: { type: "FeatureCollection", features: [] } });
  map.addLayer({ id: "ordo-areas", type: "fill", source: OBJECTS, filter: ["==", ["geometry-type"], "Polygon"], paint: { "fill-color": accent, "fill-opacity": 0.2 } });
  map.addLayer({ id: "ordo-lines", type: "line", source: OBJECTS, filter: ["!=", ["geometry-type"], "Point"], paint: { "line-color": accent, "line-width": 3 } });
  map.addLayer({ id: "ordo-points", type: "circle", source: OBJECTS, filter: ["==", ["geometry-type"], "Point"], paint: { "circle-radius": 8, "circle-color": accent, "circle-stroke-width": 2, "circle-stroke-color": halo } });
  map.addLayer({
    id: "ordo-labels",
    type: "symbol",
    source: OBJECTS,
    layout: { "text-field": ["get", "label"], "text-font": ["Noto Sans Medium"], "text-size": 14, "text-offset": [0, 1.2], "text-anchor": "top" },
    paint: { "text-color": text, "text-halo-color": halo, "text-halo-width": 2 },
  });
  return { map, from: source.from };
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

/** Whether the server has a map region, and the size of the device's copy (0: none). */
export async function offlineState() {
  const file = await offlineFile();
  let server = false;
  try {
    server = (await fetch(REGION_URL, { method: "HEAD" })).ok;
  } catch {
    server = false;
  }
  return { server, device: file ? file.size : 0 };
}

/** Keeps the whole map region on the device (origin-private file system). */
export async function saveOffline() {
  const response = await fetch(REGION_URL);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const dir = await navigator.storage.getDirectory();
  const handle = await dir.getFileHandle(OFFLINE_NAME, { create: true });
  const writable = await handle.createWritable();
  await response.body.pipeTo(writable);
  return (await handle.getFile()).size;
}

/** Deletes the device's copy of the map region. */
export async function removeOffline() {
  const dir = await navigator.storage.getDirectory();
  await dir.removeEntry(OFFLINE_NAME).catch(() => {});
}
