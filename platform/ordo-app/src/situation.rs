//! The situation map (MAP-1, MAP-2, ADR-0006): the command workspace's view *Lage* with the
//! map, the list of everything on it (A11Y-5), placing objects at the map's centre, and the
//! offline copy of the map region, objects shown with the profile's tactical symbols (MAP-3).
//! The map itself is MapLibre GL JS behind ORDO's glue
//! `map/ordo-map.js`, imported only when the view opens.

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
use crate::text;
use crate::ui::{Action, Actions};

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
            let mut properties =
                serde_json::json!({ "label": o.placed.label, "kind": o.placed.kind });
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

// [impl->dsn~situation-map~1]
/// The map with ORDO's objects, the list of them, placing and offline.
#[component]
pub fn SituationView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let editable = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
    let situation = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| SituationMap::from_events(&c.events(ordo_map::COMPARTMENT)))
    });
    let symbols = Memo::new(move |_| {
        revision.track();
        situation.with(|s| symbols_of(core, s))
    });
    let features = move || situation.with(|s| symbols.with(|y| geojson(s, y)));
    let state = RwSignal::new(MapState::Loading);
    // The glue module and the map handle, once created (JS values stay on this thread).
    let handle = StoredValue::new_local(None::<(JsValue, JsValue)>);
    let node = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        let Some(element) = node.get() else { return };
        if handle.with_value(Option::is_some) {
            return;
        }
        let centre = situation.with_untracked(SituationMap::centre);
        leptos::task::spawn_local(async move {
            let created = async {
                let module = glue().await?;
                let centre = centre.map_or(JsValue::NULL, |[lon, lat]| {
                    js_sys::Array::of2(&JsValue::from_f64(lon), &JsValue::from_f64(lat)).into()
                });
                let map = call_async(
                    &module,
                    "create",
                    &[element.into(), centre, JsValue::from_str(text::MAP_LABEL)],
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
                    let _ = call(
                        &module,
                        "setObjects",
                        &[map.clone(), JsValue::from_str(&untrack(features))],
                    );
                    handle.set_value(Some((module, map)));
                    state.set(MapState::Ready(match from.as_str() {
                        "device" => text::MAP_FROM_DEVICE,
                        "server" => text::MAP_FROM_SERVER,
                        _ => text::MAP_NO_REGION,
                    }));
                }
                Err(_) => state.set(MapState::Unavailable),
            }
        });
    });
    // Every change of the objects goes to the map.
    Effect::new(move || {
        let json = features();
        handle.with_value(|h| {
            if let Some((module, map)) = h {
                let _ = call(
                    module,
                    "setObjects",
                    &[map.clone(), JsValue::from_str(&json)],
                );
            }
        });
    });
    let centre = move || -> Option<[f64; 2]> {
        handle.with_value(|h| {
            let (module, map) = h.as_ref()?;
            let c: js_sys::Array = call(module, "centre", std::slice::from_ref(map))
                .ok()?
                .dyn_into()
                .ok()?;
            Some([c.get(0).as_f64()?, c.get(1).as_f64()?])
        })
    };
    let set_draft = move |json: String| {
        handle.with_value(|h| {
            if let Some((module, map)) = h {
                let _ = call(module, "setDraft", &[map.clone(), JsValue::from_str(&json)]);
            }
        });
    };
    let show = move |at: [f64; 2]| {
        handle.with_value(|h| {
            if let Some((module, map)) = h {
                let _ = call(
                    module,
                    "flyTo",
                    &[
                        map.clone(),
                        JsValue::from_f64(at[0]),
                        JsValue::from_f64(at[1]),
                    ],
                );
            }
        });
    };
    view! {
        <section class="card situation" aria-labelledby="situation-title">
            <h2 id="situation-title">{text::MAP_TITLE}</h2>
            <p class="situation__state" role="status">
                {move || match state.get() {
                    MapState::Loading => text::MAP_LOADING,
                    MapState::Ready(from) => from,
                    MapState::Unavailable => text::MAP_UNAVAILABLE,
                }}
            </p>
            <div class="situation__frame">
                <div class="situation__map" node_ref=node data-map=move || match state.get() {
                    MapState::Ready(_) => "ready",
                    MapState::Loading => "loading",
                    MapState::Unavailable => "unavailable",
                }></div>
                // The map's centre, where objects are placed and corners set.
                <div class="situation__crosshair" aria-hidden="true"></div>
            </div>
            <h3 id="objects-title">{text::MAP_OBJECTS}</h3>
            {move || {
                let list: Vec<_> = situation.with(|s| s.list().into_iter().cloned().collect());
                let symbols = symbols.get();
                if list.is_empty() {
                    return view! { <p class="hint">{text::MAP_OBJECTS_NONE}</p> }.into_any();
                }
                let editable = editable.get();
                view! {
                    <ul class="map-objects" aria-labelledby="objects-title" data-keys="list">
                        {list
                            .into_iter()
                            .map(|o| {
                                let at = o.placed.geometry.anchor();
                                let mut actions = Vec::new();
                                if let Some(at) = at {
                                    actions.push(Action {
                                        text: text::MAP_SHOW,
                                        label: text::map_show_label(&o.placed.label),
                                        quiet: false,
                                        run: Callback::new(move |()| show(at)),
                                    });
                                }
                                if editable {
                                    let object = o.placed.object.clone();
                                    actions.push(Action {
                                        text: text::MAP_REMOVE,
                                        label: text::map_remove_label(&o.placed.label),
                                        quiet: true,
                                        run: Callback::new(move |()| {
                                            let removed = ObjectRemoved { object: object.clone() };
                                            let _ = core::map_event(core, revision, ordo_map::REMOVED, &removed.encode().unwrap_or_default());
                                        }),
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
                                        {match (o.placed.kind, o.placed.reference.as_deref().and_then(|r| DRAWINGS.get(r))) {
                                            (ObjectKind::Drawing, Some(c)) => c.label.clone(),
                                            (kind, _) => text::map_kind(kind).to_owned(),
                                        }}
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
            <Show when=move || editable.get()>
                <PlaceForm core revision centre=Callback::new(move |()| centre()) />
                <DrawForm core revision centre=Callback::new(move |()| centre()) draft=Callback::new(set_draft) />
            </Show>
            <OfflineRegion state handle />
        </section>
    }
}

/// Placing the incident location, a unit, a UHS or a position at the map's centre: the
/// keyboard moves the map (arrow keys), so placing never needs a pointer.
#[component]
fn PlaceForm(
    core: Handle,
    revision: RwSignal<u64>,
    centre: Callback<(), Option<[f64; 2]>>,
) -> impl IntoView {
    let kind = RwSignal::new("incident".to_owned());
    let reference = RwSignal::new(String::new());
    let label = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let choices = Memo::new(move |_| {
        revision.track();
        match kind.get().as_str() {
            "unit" => core.with_value(|c| {
                ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                    .present()
                    .map(|u| (u.unit.clone(), u.unit.clone()))
                    .collect::<Vec<_>>()
            }),
            "uhs" => crate::views::uhs_list(core)
                .into_iter()
                .map(|u| (u.id, u.name))
                .collect(),
            "position" => crate::command::position_choices(&crate::command::structure(core)),
            _ => Vec::new(),
        }
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(at) = centre.run(()) else {
            error.set(Some(text::MAP_UNAVAILABLE));
            return;
        };
        let kind_value = kind.get_untracked();
        let object_kind = match kind_value.as_str() {
            "unit" => ObjectKind::Unit,
            "uhs" => ObjectKind::Uhs,
            "position" => ObjectKind::Position,
            _ => ObjectKind::Incident,
        };
        let reference_value = reference.get_untracked();
        let shown = if object_kind == ObjectKind::Incident {
            label.get_untracked().trim().to_owned()
        } else {
            choices
                .with_untracked(|c| {
                    c.iter()
                        .find(|(id, _)| *id == reference_value)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_default()
        };
        if object_kind != ObjectKind::Incident && reference_value.is_empty() {
            error.set(Some(text::MAP_PLACE_MISSING));
            return;
        }
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
        if placed.check().is_err() {
            error.set(Some(text::MAP_PLACE_MISSING));
            return;
        }
        if core::map_event(
            core,
            revision,
            ordo_map::PLACED,
            &placed.encode().unwrap_or_default(),
        )
        .is_ok()
        {
            error.set(None);
        }
    };
    view! {
        <form class="form place-form" novalidate on:submit=submit>
            <h3>{text::MAP_PLACE}</h3>
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
                            .map(|(id, name)| view! { <option value=id>{name}</option> })
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
                            {move || choices.get().into_iter().map(|(id, name)| {
                                let chosen = id.clone();
                                view! { <option value=id selected=move || reference.get() == chosen>{name}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                </Show>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row"><button type="submit" class="button">{text::MAP_PLACE_SUBMIT}</button></div>
        </form>
    }
}

// [impl->dsn~situation-map~1]
/// Keeping the map region on the device for use without network (MAP-2).
#[component]
fn OfflineRegion(
    state: RwSignal<MapState>,
    handle: StoredValue<Option<(JsValue, JsValue)>, LocalStorage>,
) -> impl IntoView {
    let status = RwSignal::new(None::<(bool, f64)>);
    let message = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let module = move || handle.with_value(|h| h.as_ref().map(|(m, _)| m.clone()));
    let refresh = move || {
        if let Some(m) = module() {
            leptos::task::spawn_local(async move {
                if let Ok(s) = call_async(&m, "offlineState", &[]).await {
                    let server = js_sys::Reflect::get(&s, &"server".into())
                        .ok()
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let device = js_sys::Reflect::get(&s, &"device".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    status.set(Some((server, device)));
                }
            });
        }
    };
    Effect::new(move || {
        if matches!(state.get(), MapState::Ready(_)) {
            refresh();
        }
    });
    let save = move |_| {
        let Some(m) = module() else { return };
        busy.set(true);
        message.set(Some(text::MAP_SAVING.to_owned()));
        leptos::task::spawn_local(async move {
            match call_async(&m, "saveOffline", &[]).await {
                Ok(size) => message.set(Some(text::map_saved(size.as_f64().unwrap_or(0.0)))),
                Err(_) => message.set(Some(text::MAP_SAVE_FAILED.to_owned())),
            }
            busy.set(false);
            refresh();
        });
    };
    let remove = move |_| {
        let Some(m) = module() else { return };
        leptos::task::spawn_local(async move {
            let _ = call_async(&m, "removeOffline", &[]).await;
            message.set(Some(text::MAP_REMOVED.to_owned()));
            refresh();
        });
    };
    view! {
        <div class="offline-region">
            <h3>{text::MAP_OFFLINE}</h3>
            {move || match status.get() {
                None => view! { <p class="hint">{text::MAP_LOADING}</p> }.into_any(),
                Some((false, device)) if device <= 0.0 => view! { <p class="hint">{text::MAP_NO_REGION}</p> }.into_any(),
                Some((server, device)) => view! {
                    <p class="offline-region__state">
                        {if device > 0.0 { text::map_on_device(device) } else { text::MAP_NOT_ON_DEVICE.to_owned() }}
                    </p>
                    <div class="row">
                        {server.then(|| view! {
                            <button type="button" class="button button--small" disabled=move || busy.get() on:click=save>{text::MAP_SAVE}</button>
                        })}
                        {(device > 0.0).then(|| view! {
                            <button type="button" class="button button--small button--quiet" on:click=remove>{text::MAP_REMOVE_OFFLINE}</button>
                        })}
                    </div>
                }.into_any(),
            }}
            <p role="status">{move || message.get()}</p>
        </div>
    }
}

// [impl->dsn~map-drawing~1]
/// Drawing an area or a line (MAP-4, ADR-0058): corner by corner at the map's centre, which
/// the keyboard, touch or a mouse moves; the map previews the shape as it grows.
#[component]
fn DrawForm(
    core: Handle,
    revision: RwSignal<u64>,
    centre: Callback<(), Option<[f64; 2]>>,
    draft: Callback<String>,
) -> impl IntoView {
    let category = RwSignal::new(
        DRAWINGS
            .categories
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_default(),
    );
    let label = RwSignal::new(String::new());
    let corners = RwSignal::new(Vec::<[f64; 2]>::new());
    let error = RwSignal::new(None::<&'static str>);
    let shape =
        move || category.with(|c| DRAWINGS.get(c).map_or(ordo_map::Shape::Area, |c| c.shape));
    // The preview: the corners and the shape so far.
    Effect::new(move || {
        let points = corners.get();
        let area = shape() == ordo_map::Shape::Area;
        let mut features: Vec<serde_json::Value> = points
            .iter()
            .map(|p| serde_json::json!({ "type": "Feature", "properties": {}, "geometry": Geometry::Point(*p) }))
            .collect();
        if points.len() >= 2 {
            let line = if area && points.len() >= 3 {
                ordo_map::drawings::closed_ring(&points)
            } else {
                points.clone()
            };
            features.push(serde_json::json!({ "type": "Feature", "properties": {}, "geometry": Geometry::LineString(line) }));
        }
        draft.run(
            serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string(),
        );
    });
    let add = move |_| match centre.run(()) {
        Some(at) => {
            corners.update(|c| c.push(at));
            error.set(None);
        }
        None => error.set(Some(text::MAP_UNAVAILABLE)),
    };
    let undo = move |_| {
        corners.update(|c| {
            c.pop();
        })
    };
    let cancel = move |_| {
        corners.set(Vec::new());
        label.set(String::new());
        error.set(None);
    };
    let finish = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(chosen) = DRAWINGS.get(&category.get_untracked()).cloned() else {
            return;
        };
        let points = corners.get_untracked();
        if points.len() < chosen.shape.min_points() {
            error.set(Some(match chosen.shape {
                ordo_map::Shape::Area => text::DRAW_TOO_FEW_AREA,
                ordo_map::Shape::Line => text::DRAW_TOO_FEW_LINE,
            }));
            return;
        }
        let Some(id) = core::new_ulid() else { return };
        let name = label.get_untracked().trim().to_owned();
        let placed = ObjectPlaced {
            object: ObjectKind::Drawing.object_id(&id.to_string()),
            kind: ObjectKind::Drawing,
            reference: Some(chosen.id.clone()),
            label: if name.is_empty() {
                chosen.label.clone()
            } else {
                name
            },
            geometry: match chosen.shape {
                ordo_map::Shape::Area => {
                    Geometry::Polygon(vec![ordo_map::drawings::closed_ring(&points)])
                }
                ordo_map::Shape::Line => Geometry::LineString(points),
            },
        };
        if placed.check().is_err() {
            error.set(Some(text::MAP_PLACE_MISSING));
            return;
        }
        if core::map_event(
            core,
            revision,
            ordo_map::PLACED,
            &placed.encode().unwrap_or_default(),
        )
        .is_ok()
        {
            corners.set(Vec::new());
            label.set(String::new());
            error.set(None);
        }
    };
    view! {
        <form class="form draw-form" novalidate on:submit=finish>
            <h3>{text::DRAW_TITLE}</h3>
            <p class="hint">{text::DRAW_HINT}</p>
            <div class="form--inline">
                <div class="field">
                    <label for="draw-category">{text::DRAW_CATEGORY}</label>
                    <select id="draw-category" on:change=move |ev| category.set(event_target_value(&ev))>
                        {DRAWINGS.categories.iter().map(|c| {
                            let id = c.id.clone();
                            view! { <option value=c.id.clone() selected=move || category.get() == id>{c.label.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="draw-label">{text::MAP_PLACE_LABEL}</label>
                    <input id="draw-label" type="text" autocomplete="off" bind:value=label />
                </div>
            </div>
            <p class="draw-form__count" role="status">{move || text::draw_corners(corners.with(Vec::len))}</p>
            <div class="row">
                <button type="button" class="button" on:click=add>{text::DRAW_ADD_CORNER}</button>
                <button type="button" class="button button--quiet" on:click=undo disabled=move || corners.with(Vec::is_empty)>{text::DRAW_UNDO}</button>
                <button type="button" class="button button--quiet" on:click=cancel disabled=move || corners.with(Vec::is_empty)>{text::DRAW_CANCEL}</button>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row"><button type="submit" class="button button--primary">{text::DRAW_FINISH}</button></div>
        </form>
    }
}
