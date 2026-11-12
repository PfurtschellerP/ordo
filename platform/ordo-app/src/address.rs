//! Address search for the incident's map (ADR-0066): the person types an address and
//! presses "Suchen"; the server asks its geocoder and answers with up to five places. There
//! is no search as you type (the public geocoder's policy, and less traffic in the field).

use leptos::prelude::*;
use ordo_sync::message::{GEOCODE_QUERY_MAX, GeocodeStatus, GeocodedPlace};

use crate::core::{self, Handle};
use crate::text;

/// A chosen place: longitude and latitude in degrees, and its name.
pub type Chosen = ([f64; 2], String);

// [impl->dsn~address-search~1]
/// A search form and its places; `on_choose` gets the place the person picks. `id`
/// prefixes the element IDs, so the form can appear more than once in the app.
#[component]
pub fn AddressSearch(
    core: Handle,
    revision: RwSignal<u64>,
    #[prop(into)] id: String,
    on_choose: Callback<Chosen>,
) -> impl IntoView {
    let input_id = format!("{id}-address");
    let query = RwSignal::new(String::new());
    // The query sent, whose answer is awaited or shown.
    let asked = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let chosen = RwSignal::new(None::<String>);
    let answer = Memo::new(move |_| {
        revision.track();
        let asked = asked.get()?;
        core.with_value(|c| c.geocoded.clone())
            .filter(|a| a.query == asked)
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let q = query.get_untracked().trim().to_owned();
        chosen.set(None);
        if q.is_empty() {
            message.set(Some(text::ADDRESS_EMPTY.to_owned()));
            return;
        }
        if q.chars().count() > GEOCODE_QUERY_MAX {
            message.set(Some(text::ADDRESS_TOO_LONG.to_owned()));
            return;
        }
        if core::geocode(core, &q) {
            asked.set(Some(q));
            message.set(Some(text::ADDRESS_SEARCHING.to_owned()));
        } else {
            asked.set(None);
            message.set(Some(text::ADDRESS_OFFLINE.to_owned()));
        }
    };
    let status = move || {
        if let Some(label) = chosen.get() {
            return Some(text::address_chosen(&label));
        }
        let Some(a) = answer.get() else {
            return message.get();
        };
        Some(match a.status {
            GeocodeStatus::Found if a.places.is_empty() => text::ADDRESS_NOT_FOUND.to_owned(),
            GeocodeStatus::Found => text::address_found(a.places.len()),
            GeocodeStatus::Off => text::ADDRESS_OFF.to_owned(),
            GeocodeStatus::Refused => text::ADDRESS_REFUSED.to_owned(),
            GeocodeStatus::Failed => text::ADDRESS_FAILED.to_owned(),
        })
    };
    let places = move || answer.get().map(|a| a.places).unwrap_or_default();
    let pick = move |place: GeocodedPlace| {
        let at = [f64::from(place.lon_e7) / 1e7, f64::from(place.lat_e7) / 1e7];
        chosen.set(Some(place.label.clone()));
        on_choose.run((at, place.label));
    };
    view! {
        <div class="address-search">
            <form class="address-search__form" role="search" on:submit=submit>
                <div class="field">
                    <label for=input_id.clone()>{text::ADDRESS_LABEL}</label>
                    <input id=input_id type="search" autocomplete="off" enterkeyhint="search"
                        maxlength=GEOCODE_QUERY_MAX.to_string() bind:value=query />
                </div>
                <button type="submit" class="button">{text::ADDRESS_SUBMIT}</button>
            </form>
            <p role="status" class="address-search__status">{status}</p>
            <Show when=move || !places().is_empty()>
                <ul class="address-search__places" aria-label=text::ADDRESS_PLACES>
                    <For each=places key=|p| (p.label.clone(), p.lat_e7, p.lon_e7) let:place>
                        {
                            let label = place.label.clone();
                            view! {
                                <li>
                                    <button type="button" class="button button--small address-search__place"
                                        on:click=move |_| pick(place.clone())>{label}</button>
                                </li>
                            }
                        }
                    </For>
                </ul>
                <p class="hint">{text::ADDRESS_ATTRIBUTION}</p>
            </Show>
        </div>
    }
}
