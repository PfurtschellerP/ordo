//! The situation map (MAP-1, MAP-2, ADR-0006): the view *Lagekarte* with the map, the list
//! of everything on it (A11Y-5) and the offline copy of the map region, objects shown with
//! the profile's tactical symbols (MAP-3); and the editor page *Karte bearbeiten*
//! (ADR-0064), where the incident command places, moves, draws and reshapes. The map itself
//! is MapLibre GL JS behind ORDO's glue `map/ordo-map.js`, imported only when a map opens.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use base64::Engine as _;
use leptos::prelude::*;
use ordo_map::{
    Geometry, ObjectKind, ObjectPlaced, ObjectRemoved, SituationMap, Symbol, SymbolCatalogue,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::core::{self, Handle};
use crate::nav::Route;
use crate::text;
use crate::ui::{Action, Actions, Icon};

/// Where the glue is served (copied by Trunk, cached by the service worker).
const GLUE: &str = "/map/ordo-map.js";

// A module snippet, not `new Function`: the Content Security Policy forbids eval.
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function import_module(path) { return import(path); }"
)]
extern "C" {
    fn import_module(path: &str) -> js_sys::Promise;
}

/// Imports the glue module (a dynamic `import()`, so the map code loads only here).
async fn glue() -> Result<JsValue, JsValue> {
    JsFuture::from(import_module(GLUE)).await
}

/// Calls an exported function of the glue.
fn call(module: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let f: js_sys::Function = js_sys::Reflect::get(module, &JsValue::from_str(name))?.dyn_into()?;
    f.apply(&JsValue::NULL, &args.iter().collect::<js_sys::Array>())
}

/// Calls an async export of the glue and waits for it.
async fn call_async(module: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let promise: js_sys::Promise = call(module, name, args)?.dyn_into()?;
    JsFuture::from(promise).await
}

/// The profile's tactical symbols (`symbols/symbols.json`, checked at build time).
static SYMBOLS: LazyLock<SymbolCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/symbols.json")))
        .expect("symbols were checked at build time")
});

/// What can be drawn (`map/drawings.toml`, checked at build time; MAP-4).
static DRAWINGS: LazyLock<ordo_map::DrawingCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/drawings.json")))
        .expect("drawing categories were checked at build time")
});

/// The symbols' SVG files by symbol ID, as JSON for the glue.
const SYMBOL_IMAGES: &str = include_str!(concat!(env!("OUT_DIR"), "/symbol-images.json"));

/// A symbol as a `data:` URL for an `<img>`.
fn symbol_url(id: &str) -> Option<String> {
    static IMAGES: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
        serde_json::from_str(SYMBOL_IMAGES).expect("symbols were checked at build time")
    });
    IMAGES.get(id).map(|svg| {
        format!(
            "data:image/svg+xml;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(svg)
        )
    })
}

// [impl->dsn~tactical-symbols~1]
/// Which symbol each object gets (MAP-3), by object ID: a unit by its type or service, a
/// UHS by its kind, a position by its kind, the incident location its own. Objects this
/// device cannot look up (e.g. a UHS without access to medical data) get none.
fn symbols_of(core: Handle, map: &SituationMap) -> BTreeMap<String, Symbol> {
    let units = core
        .with_value(|c| ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)));
    let uhs = crate::views::uhs_list(core);
    let structure = crate::command::structure(core);
    map.objects
        .values()
        .filter_map(|o| {
            let reference = o.placed.reference.as_deref().unwrap_or_default();
            let symbol = match o.placed.kind {
                ObjectKind::Incident => SYMBOLS.for_incident(),
                ObjectKind::Unit => units
                    .units
                    .get(reference)
                    .and_then(|u| SYMBOLS.for_unit(&u.unit_type, u.service.as_deref())),
                ObjectKind::Uhs => uhs
                    .iter()
                    .find(|u| u.id == reference)
                    .and_then(|u| serde_json::to_value(u.kind).ok())
                    .and_then(|kind| kind.as_str().and_then(|k| SYMBOLS.for_uhs(k))),
                ObjectKind::Position => structure
                    .positions
                    .get(reference)
                    .and_then(|p| SYMBOLS.for_position(&p.created.kind)),
                ObjectKind::Drawing => None,
            }?;
            Some((o.placed.object.clone(), symbol.clone()))
        })
        .collect()
}

/// The objects as a GeoJSON FeatureCollection (RFC 7946) for the glue; `symbol` names the
/// image the glue shows for a point.
fn geojson(map: &SituationMap, symbols: &BTreeMap<String, Symbol>) -> String {
    let features: Vec<serde_json::Value> = map
        .objects
        .values()
        .map(|o| {
            // `object` lets a tap find the object (MapLibre keeps only numeric feature IDs).
            let mut properties = serde_json::json!({
                "label": o.placed.label,
                "kind": o.placed.kind,
                "object": o.placed.object,
            });
            if let Some(symbol) = symbols.get(&o.placed.object) {
                properties["symbol"] = serde_json::Value::from(symbol.id.clone());
            }
            // A drawing's line pattern, from its category (MAP-4).
            if let Some(category) = o.placed.reference.as_deref().and_then(|r| DRAWINGS.get(r))
                && o.placed.kind == ObjectKind::Drawing
            {
                properties["line"] = serde_json::to_value(category.line).unwrap_or_default();
            }
            serde_json::json!({
                "type": "Feature",
                "id": o.placed.object,
                "properties": properties,
                "geometry": o.placed.geometry,
            })
        })
        .collect();
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// "48.37012, 10.89123" (latitude first, as people read coordinates).
fn coordinates(g: &Geometry) -> String {
    g.anchor()
        .map_or_else(String::new, |[lon, lat]| format!("{lat:.5}, {lon:.5}"))
}

/// What the map shows and where it comes from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MapState {
    Loading,
    Ready(&'static str),
    Unavailable,
}

/// The glue module and the map handle, once created (JS values stay on this thread).
type MapHandle = StoredValue<Option<(JsValue, JsValue)>, LocalStorage>;

/// Calls the glue's `name` with the map handle and `args`, if the map is there.
fn with_map(handle: MapHandle, name: &str, args: &[JsValue]) -> Option<JsValue> {
    handle
        .try_with_value(|h| {
            let (module, map) = h.as_ref()?;
            let mut all = vec![map.clone()];
            all.extend_from_slice(args);
            call(module, name, &all).ok()
        })
        .flatten()
}

/// The map's centre: longitude, latitude.
fn map_centre(handle: MapHandle) -> Option<[f64; 2]> {
    let c: js_sys::Array = with_map(handle, "centre", &[])?.dyn_into().ok()?;
    Some([c.get(0).as_f64()?, c.get(1).as_f64()?])
}

/// Moves the map to a point.
fn show_at(handle: MapHandle, at: [f64; 2]) {
    let _ = with_map(
        handle,
        "flyTo",
        &[JsValue::from_f64(at[0]), JsValue::from_f64(at[1])],
    );
}

/// The incident's map objects, their symbols, and whether this person may edit them.
#[derive(Clone, Copy)]
struct Situation {
    map: Memo<SituationMap>,
    symbols: Memo<BTreeMap<String, Symbol>>,
    editable: Memo<bool>,
}

fn situation(core: Handle, revision: RwSignal<u64>) -> Situation {
    let editable = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
    let map = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| SituationMap::from_events(&c.events(ordo_map::COMPARTMENT)))
    });
    let symbols = Memo::new(move |_| {
        revision.track();
        map.with(|s| symbols_of(core, s))
    });
    Situation {
        map,
        symbols,
        editable,
    }
}

// [impl->dsn~situation-map~4]
/// Creates the map in `node` once it is in the page, keeps it showing the objects, and
/// removes it with the view.
fn mount_map(
    core: Handle,
    node: NodeRef<leptos::html::Div>,
    s: Situation,
    interactive: bool,
) -> (MapHandle, RwSignal<MapState>) {
    let features = move || s.map.with(|m| s.symbols.with(|y| geojson(m, y)));
    let state = RwSignal::new(MapState::Loading);
    let handle: MapHandle = StoredValue::new_local(None);
    Effect::new(move || {
        let Some(element) = node.get() else { return };
        if handle.with_value(Option::is_some) {
            return;
        }
        let centre = s.map.with_untracked(SituationMap::centre);
        leptos::task::spawn_local(async move {
            let created = async {
                let module = glue().await?;
                let centre = centre.map_or(JsValue::NULL, |[lon, lat]| {
                    js_sys::Array::of2(&JsValue::from_f64(lon), &JsValue::from_f64(lat)).into()
                });
                // [impl->dsn~map-region~2]
                // The incident's region, read with this connection's ticket (ADR-0062),
                // which follows the welcome: wait for it briefly while connecting.
                for _ in 0..30 {
                    let offline = core.with_value(|c| c.connection == core::Connection::Offline);
                    if offline || core::map_ticket(core).is_some() {
                        break;
                    }
                    core::sleep_ms(100).await;
                }
                let (incident, ticket) = region_args(core);
                let map = call_async(
                    &module,
                    "create",
                    &[
                        element.into(),
                        centre,
                        JsValue::from_str(text::MAP_LABEL),
                        incident,
                        ticket,
                        JsValue::from_bool(interactive),
                    ],
                )
                .await?;
                // The symbols first, so the objects find their images.
                call_async(
                    &module,
                    "setSymbols",
                    &[map.clone(), JsValue::from_str(SYMBOL_IMAGES)],
                )
                .await?;
                Ok::<_, JsValue>((module, map))
            };
            match created.await {
                Ok((module, map)) => {
                    let from = call(&module, "regionSource", std::slice::from_ref(&map))
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    // The view may have closed meanwhile: then the map goes too.
                    let Some(json) = s.map.try_with_untracked(|_| untrack(features)) else {
                        let _ = call(&module, "destroy", &[map]);
                        return;
                    };
                    let _ = call(
                        &module,
                        "setObjects",
                        &[map.clone(), JsValue::from_str(&json)],
                    );
                    if let Some((module, map)) = handle.try_set_value(Some((module, map))).flatten()
                    {
                        let _ = call(&module, "destroy", &[map]);
                        return;
                    }
                    state.set(MapState::Ready(match from.as_str() {
                        "device" => text::MAP_FROM_DEVICE,
                        "server" => text::MAP_FROM_SERVER,
                        _ => text::MAP_NO_REGION,
                    }));
                }
                Err(_) => {
                    let _ = state.try_set(MapState::Unavailable);
                }
            }
        });
    });
    // Every change of the objects goes to the map.
    Effect::new(move || {
        let json = features();
        let _ = with_map(handle, "setObjects", &[JsValue::from_str(&json)]);
    });
    on_cleanup(move || {
        let _ = with_map(handle, "destroy", &[]);
    });
    (handle, state)
}

/// The line under the title: loading, where the map region comes from, or no map.
#[component]
fn MapStateLine(state: RwSignal<MapState>) -> impl IntoView {
    view! {
        <p class="situation__state" role="status">
            {move || match state.get() {
                MapState::Loading => text::MAP_LOADING,
                MapState::Ready(from) => from,
                MapState::Unavailable => text::MAP_UNAVAILABLE,
            }}
        </p>
    }
}

/// The map's frame; the crosshair marks the centre where the keyboard places and draws.
#[component]
fn MapFrame(
    node: NodeRef<leptos::html::Div>,
    state: RwSignal<MapState>,
    #[prop(default = false)] crosshair: bool,
) -> impl IntoView {
    view! {
        <div class="situation__frame">
            <div class="situation__map" node_ref=node data-map=move || match state.get() {
                MapState::Ready(_) => "ready",
                MapState::Loading => "loading",
                MapState::Unavailable => "unavailable",
            }></div>
            {crosshair.then(|| view! { <div class="situation__crosshair" aria-hidden="true"></div> })}
        </div>
    }
}

// [impl->dsn~situation-map~4]
/// The device's own position, only when asked: the map moves there.
#[component]
fn LocateButton(handle: MapHandle, state: RwSignal<MapState>) -> impl IntoView {
    let located = RwSignal::new(None::<&'static str>);
    let locate = move |_| {
        let Some((module, map)) = handle.with_value(Clone::clone) else {
            return;
        };
        located.set(Some(text::MAP_LOCATING));
        leptos::task::spawn_local(async move {
            let result = call_async(&module, "locate", &[map]).await;
            let _ = located.try_set(Some(match result {
                Ok(_) => text::MAP_LOCATED,
                Err(e) if e.as_string().as_deref() == Some("denied") => text::MAP_LOCATE_DENIED,
                Err(_) => text::MAP_LOCATE_FAILED,
            }));
        });
    };
    view! {
        <button id="map-locate" type="button" class="button button--small"
            disabled=move || !matches!(state.get(), MapState::Ready(_))
            on:click=locate>{text::MAP_LOCATE}</button>
        <p role="status" class="hint map-locate__state">{move || located.get()}</p>
    }
}

/// What an object is, as the list says it: a drawing by its category.
fn kind_label(placed: &ObjectPlaced) -> String {
    match (
        placed.kind,
        placed.reference.as_deref().and_then(|r| DRAWINGS.get(r)),
    ) {
        (ObjectKind::Drawing, Some(c)) => c.label.clone(),
        (kind, _) => text::map_kind(kind).to_owned(),
    }
}

/// The list of everything on the map (A11Y-5). On the editor page each object can be
/// chosen (`edit`) and removed.
#[component]
fn ObjectList(
    core: Handle,
    revision: RwSignal<u64>,
    s: Situation,
    handle: MapHandle,
    #[prop(default = None)] edit: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <h3 id="objects-title">{text::MAP_OBJECTS}</h3>
        {move || {
            let list: Vec<_> = s.map.with(|m| m.list().into_iter().cloned().collect());
            let symbols = s.symbols.get();
            if list.is_empty() {
                return view! { <p class="hint">{text::MAP_OBJECTS_NONE}</p> }.into_any();
            }
            let editing = edit.is_some() && s.editable.get();
            view! {
                <ul class="map-objects" aria-labelledby="objects-title" data-keys="list">
                    {list
                        .into_iter()
                        .map(|o| {
                            let mut actions = Vec::new();
                            if let Some(at) = o.placed.geometry.anchor() {
                                actions.push(Action {
                                    text: text::MAP_SHOW,
                                    label: text::map_show_label(&o.placed.label),
                                    quiet: false,
                                    run: Callback::new(move |()| show_at(handle, at)),
                                });
                            }
                            if let (true, Some(edit)) = (editing, edit) {
                                let object = o.placed.object.clone();
                                actions.push(Action {
                                    text: text::MAP_EDIT_OBJECT,
                                    label: text::map_edit_label(&o.placed.label),
                                    quiet: false,
                                    run: Callback::new(move |()| edit.run(object.clone())),
                                });
                                let object = o.placed.object.clone();
                                actions.push(Action {
                                    text: text::MAP_REMOVE,
                                    label: text::map_remove_label(&o.placed.label),
                                    quiet: true,
                                    run: Callback::new(move |()| remove(core, revision, &object)),
                                });
                            }
                            // The symbol's name as its text, e.g. "Rettungswagen".
                            let symbol = symbols.get(&o.placed.object).and_then(|s| {
                                symbol_url(&s.id).map(|url| view! {
                                    <img class="map-object__symbol" src=url alt=s.label.clone() data-symbol=s.id.clone() />
                                })
                            });
                            view! {
                                <li class="map-object" data-object=o.placed.object.clone()>
                                    {symbol}
                                    <strong>{o.placed.label.clone()}</strong> " · "
                                    {kind_label(&o.placed)}
                                    " · " <span class="map-object__at">{coordinates(&o.placed.geometry)}</span>
                                    <Actions actions />
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        }}
    }
}

/// Writes a placement; `false` if it is not valid or not stored.
fn place(core: Handle, revision: RwSignal<u64>, placed: &ObjectPlaced) -> bool {
    placed.check().is_ok()
        && core::map_event(
            core,
            revision,
            ordo_map::PLACED,
            &placed.encode().unwrap_or_default(),
        )
        .is_ok()
}

/// Takes an object off the map.
fn remove(core: Handle, revision: RwSignal<u64>, object: &str) {
    let removed = ObjectRemoved {
        object: object.to_owned(),
    };
    let _ = core::map_event(
        core,
        revision,
        ordo_map::REMOVED,
        &removed.encode().unwrap_or_default(),
    );
}

// [impl->dsn~situation-map~4]
/// *Führung – Lagekarte*: the map with ORDO's objects, the list of them and the offline
/// copy. Editing happens in *Vorbereitung* (ADR-0065).
#[component]
pub fn SituationView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let s = situation(core, revision);
    let node = NodeRef::<leptos::html::Div>::new();
    let (handle, state) = mount_map(core, node, s, true);
    view! {
        <section class="card situation" aria-labelledby="situation-title">
            <h2 id="situation-title">{text::MAP_TITLE}</h2>
            <MapStateLine state />
            <MapFrame node state />
            <div class="row">
                <LocateButton handle state />
            </div>
            <ObjectList core revision s handle />
            <OfflineRegion core state handle />
        </section>
    }
}

// [impl->dsn~situation-map~4]
/// The overview's map tile (ADR-0067): the map shows the objects but does not move, with
/// the number of objects and where the region comes from. It keeps the region on the
/// device like the Lagekarte.
#[component]
pub fn MapPreview(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let s = situation(core, revision);
    let node = NodeRef::<leptos::html::Div>::new();
    let (handle, state) = mount_map(core, node, s, false);
    view! {
        <div class="tile__map">
            <MapFrame node state />
        </div>
        <div class="tile__chips">
            <span id="overview-objects" class="tag">{move || text::summary_map(s.map.with(|m| m.objects.len()))}</span>
            <span class="tag" data-from=move || match state.get() {
                MapState::Ready(from) if from == text::MAP_FROM_DEVICE => "device",
                MapState::Ready(_) => "server",
                MapState::Loading => "loading",
                MapState::Unavailable => "unavailable",
            }>{move || match state.get() {
                MapState::Ready(from) => from,
                MapState::Loading => text::MAP_LOADING,
                MapState::Unavailable => text::MAP_UNAVAILABLE,
            }}</span>
        </div>
        <OfflineRegion core state handle />
    }
}

// [impl->dsn~situation-map~4]
/// *Vorbereitung – Lagekarte* (ADR-0067): the incident's map region, for those who may
/// prepare it, then the map editor, on one page.
#[component]
pub fn MapPreparation(
    core: Handle,
    revision: RwSignal<u64>,
    /// Opened to place one object: the place panel's kind and reference.
    #[prop(default = None)]
    preset: Option<(String, String)>,
) -> impl IntoView {
    let manage_map = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, crate::incidents::MANAGE_MAP))
        })
    });
    view! {
        <Show when=move || manage_map.get()>
            <crate::map_region::MapRegionView core revision />
        </Show>
        <MapEditor core revision preset />
    }
}

/// The record that opens *Vorbereitung – Lagekarte* to place one object:
/// `place:<kind>:<ID>`.
const PLACE: &str = "place:";

/// The route of *Vorbereitung – Lagekarte*, with a [`place_record`] or none.
pub fn editor_route(record: Option<&str>) -> Route {
    match record {
        Some(r) => Route::record("command.prepare", "map", r),
        None => Route::view("command.prepare", "map"),
    }
}

/// The record that opens the editor to place `reference` of `kind`, e.g. a UHS.
pub fn place_record(kind: ObjectKind, reference: &str) -> String {
    format!("{PLACE}{}:{reference}", place_kind(kind))
}

/// What to place, from a record made by [`place_record`].
pub fn preset(record: &str) -> Option<(String, String)> {
    let (kind, reference) = record.strip_prefix(PLACE)?.split_once(':')?;
    (["unit", "uhs", "position"].contains(&kind) && !reference.is_empty())
        .then(|| (kind.to_owned(), reference.to_owned()))
}

/// The kind as the place panel's choice names it.
fn place_kind(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Unit => "unit",
        ObjectKind::Uhs => "uhs",
        ObjectKind::Position => "position",
        ObjectKind::Incident | ObjectKind::Drawing => "incident",
    }
}

/// The editor page's tools (ADR-0064).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tool {
    Select,
    Place,
    Area,
    Line,
}

impl Tool {
    const ALL: [Self; 4] = [Self::Select, Self::Place, Self::Area, Self::Line];

    fn label(self) -> &'static str {
        match self {
            Self::Select => text::TOOL_SELECT,
            Self::Place => text::TOOL_PLACE,
            Self::Area => text::TOOL_AREA,
            Self::Line => text::TOOL_LINE,
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Select => "mouse-pointer-2",
            Self::Place => "map-pin",
            Self::Area => "pentagon",
            Self::Line => "spline",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Select => "tool-select",
            Self::Place => "tool-place",
            Self::Area => "tool-area",
            Self::Line => "tool-line",
        }
    }

    /// The shape a drawing tool draws.
    fn shape(self) -> Option<ordo_map::Shape> {
        match self {
            Self::Area => Some(ordo_map::Shape::Area),
            Self::Line => Some(ordo_map::Shape::Line),
            Self::Select | Self::Place => None,
        }
    }
}

/// The first category the profile has for `shape`.
fn first_category(shape: ordo_map::Shape) -> String {
    DRAWINGS
        .categories
        .iter()
        .find(|c| c.shape == shape)
        .map(|c| c.id.clone())
        .unwrap_or_default()
}

/// The preview of a shape being drawn: its corners, and the line or ring so far.
fn draft(corners: &[[f64; 2]], shape: ordo_map::Shape) -> Vec<serde_json::Value> {
    let mut features: Vec<serde_json::Value> = corners
        .iter()
        .map(|p| serde_json::json!({ "type": "Feature", "properties": {}, "geometry": Geometry::Point(*p) }))
        .collect();
    if corners.len() >= 2 {
        let line = if shape == ordo_map::Shape::Area && corners.len() >= 3 {
            ordo_map::drawings::closed_ring(corners)
        } else {
            corners.to_vec()
        };
        features.push(serde_json::json!({ "type": "Feature", "properties": {}, "geometry": Geometry::LineString(line) }));
    }
    features
}

// [impl->dsn~situation-map~4]
// [impl->dsn~map-drawing~2]
/// *Karte bearbeiten* (ADR-0064): the map as large as the page allows, a toolbar above it,
/// the tool's panel and the list below. Every tap sets what the tool sets; the button
/// "… in Kartenmitte …" does the same at the crosshair, which the keyboard moves.
#[component]
pub fn MapEditor(
    core: Handle,
    revision: RwSignal<u64>,
    /// Opened to place one object: the place panel's kind and reference.
    #[prop(default = None)]
    preset: Option<(String, String)>,
) -> impl IntoView {
    let s = situation(core, revision);
    let node = NodeRef::<leptos::html::Div>::new();
    let (handle, state) = mount_map(core, node, s, true);
    let tool = RwSignal::new(if preset.is_some() {
        Tool::Place
    } else {
        Tool::Select
    });
    // Select: the chosen object, and whether Terra Draw is reshaping it.
    let chosen = RwSignal::new(None::<String>);
    let reshaping = RwSignal::new(false);
    // Area and line: the corners so far, the category, the name, and the drawing being
    // drawn again, if any.
    let corners = RwSignal::new(Vec::<[f64; 2]>::new());
    let category = RwSignal::new(first_category(ordo_map::Shape::Area));
    let name = RwSignal::new(String::new());
    let redraw = RwSignal::new(None::<String>);
    // Place: what.
    let (preset_kind, preset_reference) =
        preset.unwrap_or_else(|| ("incident".to_owned(), String::new()));
    let kind = RwSignal::new(preset_kind);
    let reference = RwSignal::new(preset_reference);
    let label = RwSignal::new(String::new());
    let status = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<&'static str>);

    let object = move |id: &str| s.map.with_untracked(|m| m.objects.get(id).cloned());
    let end_reshape = move || {
        if reshaping.get_untracked() {
            let _ = with_map(handle, "stopReshaping", &[]);
            reshaping.set(false);
        }
    };
    let reset = move || {
        end_reshape();
        chosen.set(None);
        corners.set(Vec::new());
        redraw.set(None);
        error.set(None);
    };
    let set_tool = move |t: Tool| {
        reset();
        status.set(None);
        if let Some(shape) = t.shape()
            && DRAWINGS.get(&category.get_untracked()).map(|c| c.shape) != Some(shape)
        {
            category.set(first_category(shape));
        }
        tool.set(t);
    };
    // Choosing an object; a drawing goes to Terra Draw to be reshaped by pointer.
    let choose = Callback::new(move |id: String| {
        reset();
        tool.set(Tool::Select);
        let Some(o) = object(&id) else { return };
        chosen.set(Some(id.clone()));
        status.set(Some(text::map_chosen(&o.placed.label)));
        if o.placed.kind != ObjectKind::Drawing {
            return;
        }
        let line = o
            .placed
            .reference
            .as_deref()
            .and_then(|r| DRAWINGS.get(r))
            .and_then(|c| serde_json::to_value(c.line).ok())
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let geometry = serde_json::to_string(&o.placed.geometry).unwrap_or_default();
        let Some((module, map)) = handle.with_value(Clone::clone) else {
            return;
        };
        leptos::task::spawn_local(async move {
            let args = [
                map,
                JsValue::from_str(&id),
                JsValue::from_str(&geometry),
                JsValue::from_str(&line),
            ];
            match call_async(&module, "reshape", &args).await {
                Ok(_) if chosen.get_untracked().as_deref() == Some(&id) => reshaping.set(true),
                Ok(_) => {
                    let _ = call(&module, "stopReshaping", &args[..1]);
                }
                Err(_) => error.set(Some(text::MAP_RESHAPE_FAILED)),
            }
        });
    });
    // Placing what the panel says at `at`.
    let place_at = move |at: [f64; 2]| {
        let object_kind = object_kind(&kind.get_untracked());
        let reference_value = reference.get_untracked();
        if object_kind != ObjectKind::Incident && reference_value.is_empty() {
            error.set(Some(text::MAP_PLACE_MISSING));
            return;
        }
        let shown = if object_kind == ObjectKind::Incident {
            label.get_untracked().trim().to_owned()
        } else {
            place_choices(core, object_kind)
                .into_iter()
                .find(|(id, _)| *id == reference_value)
                .map(|(_, name)| name)
                .unwrap_or_default()
        };
        let placed = ObjectPlaced {
            object: object_kind.object_id(&reference_value),
            kind: object_kind,
            reference: (object_kind != ObjectKind::Incident).then_some(reference_value),
            label: if shown.is_empty() {
                text::INCIDENT_LOCATION.to_owned()
            } else {
                shown
            },
            geometry: Geometry::Point(at),
        };
        if place(core, revision, &placed) {
            error.set(None);
            status.set(Some(text::map_placed(&placed.label)));
        } else {
            error.set(Some(text::MAP_PLACE_MISSING));
        }
    };
    // Moving the chosen point object to `at`.
    let move_to = move |at: [f64; 2]| {
        let Some(o) = chosen.get_untracked().and_then(|id| object(&id)) else {
            status.set(Some(text::MAP_TAP_TO_CHOOSE.to_owned()));
            return;
        };
        if !matches!(o.placed.geometry, Geometry::Point(_)) {
            return;
        }
        let moved = ObjectPlaced {
            geometry: Geometry::Point(at),
            ..o.placed.clone()
        };
        if place(core, revision, &moved) {
            status.set(Some(text::map_moved(&o.placed.label)));
        }
    };
    // What a tap at `at` does, with the object hit (and whether it is a point).
    let at_point = move |at: [f64; 2], hit: Option<(String, bool)>| match tool.get_untracked() {
        Tool::Select => {
            if reshaping.get_untracked() {
                return;
            }
            let current = chosen.get_untracked();
            let current_is_point = current
                .as_deref()
                .and_then(object)
                .is_some_and(|o| matches!(o.placed.geometry, Geometry::Point(_)));
            match hit {
                Some((id, _)) if current.as_ref() == Some(&id) => {}
                // With a symbol chosen, a tap into an area moves it there; another symbol
                // is chosen instead.
                Some((id, point)) if !current_is_point || point => choose.run(id),
                _ if current_is_point => move_to(at),
                _ => status.set(Some(text::MAP_TAP_TO_CHOOSE.to_owned())),
            }
        }
        Tool::Place => place_at(at),
        Tool::Area | Tool::Line => {
            corners.update(|c| c.push(at));
            error.set(None);
        }
    };
    let at_centre = move || match map_centre(handle) {
        Some(at) => at_point(at, None),
        None => error.set(Some(text::MAP_UNAVAILABLE)),
    };
    // Taps on the map, once it is there.
    let tapping = StoredValue::new(false);
    Effect::new(move || {
        if !matches!(state.get(), MapState::Ready(_)) || tapping.get_value() {
            return;
        }
        let on_tap = wasm_bindgen::closure::Closure::<dyn FnMut(f64, f64, JsValue, bool)>::new(
            move |lng: f64, lat: f64, hit: JsValue, point: bool| {
                at_point([lng, lat], hit.as_string().map(|h| (h, point)));
            },
        );
        // The map owns the handler; it goes with the map.
        let _ = with_map(handle, "onTap", &[on_tap.into_js_value()]);
        tapping.set_value(true);
    });
    // The preview: corners and shape so far, and a ring around a chosen symbol.
    Effect::new(move || {
        let shape = tool.get().shape().unwrap_or(ordo_map::Shape::Area);
        let mut features = corners.with(|c| draft(c, shape));
        if !reshaping.get()
            && let Some(o) = chosen
                .get()
                .and_then(|id| s.map.with(|m| m.objects.get(&id).cloned()))
            && let Geometry::Point(p) = o.placed.geometry
        {
            features.push(serde_json::json!({ "type": "Feature", "properties": { "chosen": true }, "geometry": Geometry::Point(p) }));
        }
        let json = serde_json::json!({ "type": "FeatureCollection", "features": features });
        let _ = with_map(handle, "setDraft", &[JsValue::from_str(&json.to_string())]);
    });
    // Saving a reshaped drawing as one event.
    let apply = move |_| {
        let Some(o) = chosen.get_untracked().and_then(|id| object(&id)) else {
            return;
        };
        let geometry = with_map(handle, "reshaped", &[])
            .and_then(|g| g.as_string())
            .and_then(|g| serde_json::from_str::<Geometry>(&g).ok());
        let Some(geometry) = geometry else {
            error.set(Some(text::MAP_RESHAPE_FAILED));
            return;
        };
        let reshaped = ObjectPlaced {
            geometry,
            ..o.placed.clone()
        };
        if place(core, revision, &reshaped) {
            reset();
            status.set(Some(text::map_reshaped(&o.placed.label)));
        } else {
            error.set(Some(text::MAP_RESHAPE_FAILED));
        }
    };
    let discard = move |_| {
        reset();
        status.set(None);
    };
    // Drawing the chosen drawing again, corner by corner, under the same ID.
    let draw_again = move |_| {
        let Some(o) = chosen.get_untracked().and_then(|id| object(&id)) else {
            return;
        };
        let category_of = o.placed.reference.as_deref().and_then(|r| DRAWINGS.get(r));
        let Some(c) = category_of else { return };
        reset();
        tool.set(match c.shape {
            ordo_map::Shape::Area => Tool::Area,
            ordo_map::Shape::Line => Tool::Line,
        });
        category.set(c.id.clone());
        name.set(o.placed.label.clone());
        redraw.set(Some(o.placed.object.clone()));
        status.set(Some(text::draw_redraw(&o.placed.label)));
    };
    let remove_chosen = move |_| {
        let Some(o) = chosen.get_untracked().and_then(|id| object(&id)) else {
            return;
        };
        reset();
        remove(core, revision, &o.placed.object);
        status.set(Some(text::map_removed(&o.placed.label)));
    };
    let finish = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(c) = DRAWINGS.get(&category.get_untracked()).cloned() else {
            return;
        };
        let points = corners.get_untracked();
        if points.len() < c.shape.min_points() {
            error.set(Some(match c.shape {
                ordo_map::Shape::Area => text::DRAW_TOO_FEW_AREA,
                ordo_map::Shape::Line => text::DRAW_TOO_FEW_LINE,
            }));
            return;
        }
        let object = match redraw.get_untracked() {
            Some(id) => id,
            None => match core::new_ulid() {
                Some(id) => ObjectKind::Drawing.object_id(&id.to_string()),
                None => return,
            },
        };
        let given = name.get_untracked().trim().to_owned();
        let placed = ObjectPlaced {
            object,
            kind: ObjectKind::Drawing,
            reference: Some(c.id.clone()),
            label: if given.is_empty() {
                c.label.clone()
            } else {
                given
            },
            geometry: match c.shape {
                ordo_map::Shape::Area => {
                    Geometry::Polygon(vec![ordo_map::drawings::closed_ring(&points)])
                }
                ordo_map::Shape::Line => Geometry::LineString(points),
            },
        };
        if place(core, revision, &placed) {
            corners.set(Vec::new());
            redraw.set(None);
            name.set(String::new());
            error.set(None);
            status.set(Some(text::map_drawn(&placed.label)));
        } else {
            error.set(Some(text::MAP_PLACE_MISSING));
        }
    };

    let toolbar = move || {
        Tool::ALL
            .into_iter()
            .map(|t| {
                view! {
                    <button id=t.id() type="button" class="map-tool"
                        aria-pressed=move || (tool.get() == t).to_string()
                        on:click=move |_| set_tool(t)>
                        <Icon name=t.icon() />{t.label()}
                    </button>
                }
            })
            .collect_view()
    };
    let select_panel = move || {
        let Some(o) = chosen
            .get()
            .and_then(|id| s.map.with(|m| m.objects.get(&id).cloned()))
        else {
            return view! { <p class="hint">{text::MAP_TAP_TO_CHOOSE}</p> }.into_any();
        };
        let title = format!("{} · {}", o.placed.label, kind_label(&o.placed));
        if o.placed.kind == ObjectKind::Drawing {
            return view! {
                <p class="map-panel__chosen"><strong>{title}</strong></p>
                <p class="hint">{text::RESHAPE_HINT}</p>
                <div class="row">
                    <button type="button" class="button button--primary" disabled=move || !reshaping.get() on:click=apply>{text::RESHAPE_APPLY}</button>
                    <button type="button" class="button button--quiet" on:click=discard>{text::RESHAPE_DISCARD}</button>
                    <button type="button" class="button" on:click=draw_again>{text::RESHAPE_REDRAW}</button>
                    <button type="button" class="button button--quiet" on:click=remove_chosen>{text::MAP_REMOVE}</button>
                </div>
            }
            .into_any();
        }
        view! {
            <p class="map-panel__chosen"><strong>{title}</strong></p>
            <p class="hint">{text::MAP_MOVE_HINT}</p>
            <div class="row">
                <button type="button" class="button button--primary" on:click=move |_| at_centre()>{text::MAP_MOVE_CENTRE}</button>
                <button type="button" class="button button--quiet" on:click=discard>{text::MAP_DESELECT}</button>
                <button type="button" class="button button--quiet" on:click=remove_chosen>{text::MAP_REMOVE}</button>
            </div>
        }
        .into_any()
    };
    let place_panel = move || {
        view! {
            <p class="hint">{text::MAP_PLACE_HINT}</p>
            <div class="form--inline">
                <div class="field">
                    <label for="place-kind">{text::MAP_PLACE_KIND}</label>
                    <select id="place-kind" prop:value=move || kind.get() on:change=move |ev| {
                        kind.set(event_target_value(&ev));
                        reference.set(String::new());
                    }>
                        {[("incident", text::INCIDENT_LOCATION), ("unit", text::TAB_RESOURCES), ("uhs", text::UHS_TITLE), ("position", text::POSITION)]
                            .into_iter()
                            .map(|(id, name)| view! { <option value=id selected=move || kind.get() == id>{name}</option> })
                            .collect_view()}
                    </select>
                </div>
                <Show
                    when=move || kind.get() != "incident"
                    fallback=move || view! {
                        <div class="field">
                            <label for="place-label">{text::MAP_PLACE_LABEL}</label>
                            <input id="place-label" type="text" autocomplete="off" bind:value=label />
                        </div>
                    }
                >
                    <div class="field">
                        <label for="place-reference">{text::MAP_PLACE_WHICH}</label>
                        <select id="place-reference" prop:value=move || reference.get() on:change=move |ev| reference.set(event_target_value(&ev))>
                            <option value="" selected=move || reference.get().is_empty()>{text::MAP_PLACE_CHOOSE}</option>
                            // Each option says whether it is chosen: when the list is drawn
                            // anew (a sync), the browser would otherwise show the first one.
                            {move || {
                                revision.track();
                                place_choices(core, object_kind(&kind.get())).into_iter().map(|(id, name)| {
                                    let mine = id.clone();
                                    view! { <option value=id selected=move || reference.get() == mine>{name}</option> }
                                }).collect_view()
                            }}
                        </select>
                    </div>
                </Show>
            </div>
            <div class="row">
                <button type="button" class="button button--primary" on:click=move |_| at_centre()>{text::MAP_PLACE_SUBMIT}</button>
            </div>
        }
    };
    let draw_panel = move || {
        let shape = tool.get().shape().unwrap_or(ordo_map::Shape::Area);
        view! {
            {move || redraw.get().map(|_| view! { <p class="map-panel__chosen"><strong>{move || text::draw_redraw(&name.get())}</strong></p> })}
            <p class="hint">{text::DRAW_HINT}</p>
            <form class="form draw-form" novalidate on:submit=finish>
                <div class="form--inline">
                    <div class="field">
                        <label for="draw-category">{text::DRAW_CATEGORY}</label>
                        <select id="draw-category" on:change=move |ev| category.set(event_target_value(&ev))>
                            {DRAWINGS.categories.iter().filter(|c| c.shape == shape).map(|c| {
                                let id = c.id.clone();
                                view! { <option value=c.id.clone() selected=move || category.get() == id>{c.label.clone()}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                    <div class="field">
                        <label for="draw-label">{text::MAP_PLACE_LABEL}</label>
                        <input id="draw-label" type="text" autocomplete="off" bind:value=name />
                    </div>
                </div>
                <p class="draw-form__count" role="status">{move || text::draw_corners(corners.with(Vec::len))}</p>
                <div class="row">
                    <button type="button" class="button" on:click=move |_| at_centre()><Icon name="crosshair" />{text::DRAW_ADD_CORNER}</button>
                    <button type="button" class="button button--quiet" on:click=move |_| corners.update(|c| { c.pop(); }) disabled=move || corners.with(Vec::is_empty)>{text::DRAW_UNDO}</button>
                    <button type="button" class="button button--quiet" on:click=move |_| { reset(); name.set(String::new()); } disabled=move || corners.with(Vec::is_empty) && redraw.with(Option::is_none)>{text::DRAW_CANCEL}</button>
                    <button type="submit" class="button button--primary">{text::DRAW_FINISH}</button>
                </div>
            </form>
        }
    };
    view! {
        <section class="card situation map-editor" aria-labelledby="map-editor-title">
            <h2 id="map-editor-title">{text::MAP_EDIT}</h2>
            <p class="hint">{text::MAP_EDIT_HINT}</p>
            <Show
                when=move || s.editable.get()
                fallback=|| view! { <p class="notice">{text::MAP_EDIT_NOT_ALLOWED}</p> }
            >
                <div class="map-tools" role="group" aria-label=text::MAP_TOOLS>{toolbar}</div>
            </Show>
            <MapStateLine state />
            <MapFrame node state crosshair=true />
            <Show when=move || s.editable.get()>
                <div class="map-panel" data-tool=move || format!("{:?}", tool.get()).to_lowercase()>
                    <h3 id="map-panel-title">{move || tool.get().label()}</h3>
                    {move || match tool.get() {
                        Tool::Select => select_panel().into_any(),
                        Tool::Place => place_panel().into_any(),
                        Tool::Area | Tool::Line => draw_panel().into_any(),
                    }}
                    <p role="alert" class="error">{move || error.get()}</p>
                    <p role="status" class="map-panel__status">{move || status.get()}</p>
                </div>
            </Show>
            <div class="row"><LocateButton handle state /></div>
            // [impl->dsn~address-search~1]
            <Show when=move || s.editable.get()>
                <crate::address::AddressSearch core revision id="editor"
                    on_choose=Callback::new(move |(at, _): crate::address::Chosen| show_at(handle, at)) />
            </Show>
            <ObjectList core revision s handle edit=Some(choose) />
        </section>
    }
}

/// The object kind of the place panel's choice.
fn object_kind(choice: &str) -> ObjectKind {
    match choice {
        "unit" => ObjectKind::Unit,
        "uhs" => ObjectKind::Uhs,
        "position" => ObjectKind::Position,
        _ => ObjectKind::Incident,
    }
}

/// What can be placed of a kind: (reference, name).
fn place_choices(core: Handle, kind: ObjectKind) -> Vec<(String, String)> {
    match kind {
        ObjectKind::Unit => core.with_value(|c| {
            ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .present()
                .map(|u| (u.unit.clone(), u.unit.clone()))
                .collect()
        }),
        ObjectKind::Uhs => crate::views::uhs_list(core)
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect(),
        ObjectKind::Position => crate::command::position_choices(&crate::command::structure(core)),
        ObjectKind::Incident | ObjectKind::Drawing => Vec::new(),
    }
}

// [impl->dsn~situation-map~4]
/// Keeping the map region on the device for use without network (MAP-2).
/// The incident's ULID and the map ticket as arguments for the glue module (`null` when
/// missing).
fn region_args(core: Handle) -> (JsValue, JsValue) {
    let incident = core::incident_ulid(core).map_or(JsValue::NULL, |i| JsValue::from_str(&i));
    let ticket = core::map_ticket(core).map_or(JsValue::NULL, |(_, t)| JsValue::from_str(&t));
    (incident, ticket)
}

/// Asks the glue whether the server has the region and whether the device keeps it; saves
/// it with `auto_save` when the server has it and the device does not.
fn refresh_state(core: Handle, module: JsValue, auto_save: Option<Callback<()>>) {
    leptos::task::spawn_local(async move {
        let (incident, ticket) = region_args(core);
        if let Ok(s) = call_async(&module, "offlineState", &[incident, ticket]).await {
            let server = js_sys::Reflect::get(&s, &"server".into())
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let device = js_sys::Reflect::get(&s, &"device".into())
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if server
                && device <= 0.0
                && let Some(f) = auto_save
            {
                f.run(());
            }
        }
    });
}

// [impl->dsn~situation-map~4]
/// The offline copy of the incident's map region (MAP-2, ADR-0065): saved by itself, once
/// per opening, when the server has a region the device does not keep yet. Nothing to
/// press; a line says when it is being saved or failed. Whether the map reads from the
/// device shows in the map's state line.
#[component]
fn OfflineRegion(core: Handle, state: RwSignal<MapState>, handle: MapHandle) -> impl IntoView {
    let message = RwSignal::new(None::<String>);
    let tried = StoredValue::new(false);
    let module = move || handle.with_value(|h| h.as_ref().map(|(m, _)| m.clone()));
    let auto_save = Callback::new(move |()| {
        let Some(m) = module() else { return };
        if tried.get_value() {
            return;
        }
        tried.set_value(true);
        message.set(Some(text::MAP_SAVING.to_owned()));
        leptos::task::spawn_local(async move {
            let (incident, ticket) = region_args(core);
            let done = match call_async(&m, "saveOffline", &[incident, ticket]).await {
                Ok(size) => text::map_saved(size.as_f64().unwrap_or(0.0)),
                Err(_) => text::MAP_SAVE_FAILED.to_owned(),
            };
            let _ = message.try_set(Some(done));
        });
    });
    Effect::new(move || {
        if matches!(state.get(), MapState::Ready(_))
            && let Some(m) = module()
        {
            refresh_state(core, m, Some(auto_save));
        }
    });
    view! { <p class="offline-region hint" role="status">{move || message.get()}</p> }
}
