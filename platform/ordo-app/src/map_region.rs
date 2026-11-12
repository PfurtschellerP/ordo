//! The incident's map region (MAP-8, ADR-0057, ADR-0062): the incident command chooses a
//! centre and a radius, the server cuts the region from the open world map, the page shows
//! the progress. On the page *Vorbereitung – Einsatz*.

use leptos::prelude::*;
use ordo_sync::message::{MapRegion, MapRegionRequest, MapRegionState};

use crate::core::{self, Handle};
use crate::text;

/// Reads a coordinate as typed: a decimal number, with a point or a comma.
fn degrees(value: &str) -> Option<f64> {
    value
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

/// The incident location on the situation map, if an incident is open and has one.
fn incident_location(core: Handle) -> Option<[f64; 2]> {
    core.with_value(|c| {
        let events = c.events(ordo_map::COMPARTMENT);
        ordo_map::SituationMap::from_events(&events)
            .objects
            .get(ordo_map::INCIDENT_OBJECT)
            .and_then(|o| o.placed.geometry.anchor())
    })
}

fn state_line(region: &MapRegion) -> String {
    let area = region.area.map(|a| {
        (
            f64::from(a.lat_e7) / 1e7,
            f64::from(a.lon_e7) / 1e7,
            f64::from(a.radius_m) / 1000.0,
        )
    });
    match region.state {
        MapRegionState::None => text::MAP_REGION_NONE.to_owned(),
        MapRegionState::Running => {
            text::map_region_running(region.done.unwrap_or(0), region.total.unwrap_or(0))
        }
        MapRegionState::Ready => text::map_region_ready(
            area,
            region.bytes.unwrap_or(0) as f64,
            region.at.map(core::date_time),
        ),
        MapRegionState::Failed => {
            text::map_region_failed(region.error.as_deref().unwrap_or_default())
        }
    }
}

// [impl->dsn~map-region~2]
/// The section: the incident's region, and a new one cut around a centre.
#[component]
pub fn MapRegionView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let region = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.map_region.clone())
    });
    // Ask now, and again while the server cuts.
    core::get_map_region(core);
    let alive = StoredValue::new(true);
    on_cleanup(move || alive.set_value(false));
    leptos::task::spawn_local(async move {
        loop {
            core::sleep_ms(1_500).await;
            if !alive.try_get_value().unwrap_or(false) {
                return;
            }
            if region
                .get_untracked()
                .is_some_and(|r| r.state == MapRegionState::Running)
            {
                core::get_map_region(core);
            }
        }
    });
    let start = incident_location(core);
    let lat = RwSignal::new(
        start
            .map(|[_, lat]| format!("{lat:.5}"))
            .unwrap_or_default(),
    );
    let lon = RwSignal::new(
        start
            .map(|[lon, _]| format!("{lon:.5}"))
            .unwrap_or_default(),
    );
    let radius = RwSignal::new("10".to_owned());
    let error = RwSignal::new(None::<&'static str>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (Some(la), Some(lo), Some(r)) = (
            degrees(&lat.get_untracked()),
            degrees(&lon.get_untracked()),
            degrees(&radius.get_untracked()),
        ) else {
            error.set(Some(text::MAP_REGION_INVALID));
            return;
        };
        if !(-85.0..=85.0).contains(&la)
            || !(-180.0..=180.0).contains(&lo)
            || !(1.0..=50.0).contains(&r)
        {
            error.set(Some(text::MAP_REGION_INVALID));
            return;
        }
        error.set(None);
        core::cut_map_region(
            core,
            MapRegionRequest {
                lat_e7: (la * 1e7).round() as i32,
                lon_e7: (lo * 1e7).round() as i32,
                radius_m: (r * 1000.0).round() as u32,
            },
        );
    };
    let running = move || {
        region
            .get()
            .is_some_and(|r| r.state == MapRegionState::Running)
    };
    view! {
        <section class="card map-region" aria-labelledby="map-region-title">
            <h2 id="map-region-title">{text::MAP_REGION_TITLE}</h2>
            <p class="hint">{text::MAP_REGION_HINT}</p>
            <p class="map-region__state" role="status">
                {move || region.get().map_or_else(|| text::MAP_LOADING.to_owned(), |r| state_line(&r))}
            </p>
            {move || {
                region.get().filter(|r| r.state == MapRegionState::Running).map(|r| {
                    let (done, total) = (r.done.unwrap_or(0), r.total.unwrap_or(0).max(1));
                    view! {
                        <progress class="map-region__progress" max=total value=done aria-label=text::MAP_REGION_PROGRESS></progress>
                    }
                })
            }}
            <form class="form" novalidate on:submit=submit>
                <h3>{text::MAP_REGION_NEW}</h3>
                <div class="form--inline">
                    <div class="field">
                        <label for="region-lat">{text::MAP_REGION_LAT}</label>
                        <input id="region-lat" type="text" inputmode="decimal" autocomplete="off" bind:value=lat />
                    </div>
                    <div class="field">
                        <label for="region-lon">{text::MAP_REGION_LON}</label>
                        <input id="region-lon" type="text" inputmode="decimal" autocomplete="off" bind:value=lon />
                    </div>
                    <div class="field">
                        <label for="region-radius">{text::MAP_REGION_RADIUS}</label>
                        <input id="region-radius" type="number" min="1" max="50" step="1" bind:value=radius />
                    </div>
                </div>
                <p class="hint">{if start.is_some() { text::MAP_REGION_FROM_INCIDENT } else { text::MAP_REGION_COORDINATES }}</p>
                <p role="alert" class="error">{move || error.get()}</p>
                <div class="row">
                    <button type="submit" class="button button--primary" disabled=running>{text::MAP_REGION_CUT}</button>
                </div>
            </form>
        </section>
    }
}
