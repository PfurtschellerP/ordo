//! The UHS module's view *Behandlungsplätze* (UHS-8, ADR-0086): every on-site care unit
//! the person may read with its free treatment places per category, so the staff see which
//! unit is full and where else a patient could go. Computed from the numbers the device
//! already holds; nothing is ranked. The units are set up by the incident command.

use leptos::prelude::*;
use ordo_med_patients::{self as med, overview};

use crate::core::Handle;
use crate::med::places::{Level, PlaceRow, TreatmentPlaces};
use crate::med::text;
use crate::med::triage::CATEGORIES;
use crate::med::uhs::uhs_list;

// [impl->dsn~crew-places~1]
/// *Freie Behandlungsplätze* for crews (UHS-19): every UHS's strips, from the shared
/// compartment only (places and patient counts), without orders and transports.
#[component]
pub fn CrewPlaces(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let (counts, mut occupancies) = core.with_value(|c| {
            let incident = c.events(med::UHS_COMPARTMENT);
            (
                med::latest_counts(&incident),
                overview::occupancies(&incident),
            )
        });
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let c = counts.get(&u.id).cloned().unwrap_or_default();
                PlaceRow {
                    occupancy: occupancies.remove(&u.id),
                    load: (CATEGORIES.iter().map(|k| c.get(*k)).sum(), u.capacity),
                    more: None,
                    id: u.id,
                    name: u.name,
                }
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section aria-labelledby="crew-places-title" class="card crew-places">
            <h2 id="crew-places-title">{text::CREW_PLACES}</h2>
            <TreatmentPlaces rows level=Level::H3 />
        </section>
    }
}

// [impl->dsn~uhs-overview~3]
/// Every unit: its places per category, its patients without one, its open orders and
/// its open and planned transports.
#[component]
pub fn OverviewView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let summaries = core.with_value(|c| {
            overview::summaries(&c.events(med::COMPARTMENT), &c.events(med::UHS_COMPARTMENT))
        });
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let s = summaries.get(&u.id).cloned().unwrap_or_default();
                let patients = CATEGORIES.iter().map(|k| s.counts.get(*k)).sum();
                PlaceRow {
                    more: Some(format!(
                        "{} · {}",
                        text::open_orders(s.open_orders, s.unconfirmed_orders),
                        text::transports_count(s.transports_open, s.transports_planned)
                    )),
                    occupancy: s.occupancy,
                    load: (patients, u.capacity),
                    id: u.id,
                    name: u.name,
                }
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section aria-labelledby="overview-title" class="card">
            <h2 id="overview-title">{text::PLACES}</h2>
            <TreatmentPlaces rows level=Level::H3 />
        </section>
    }
}
