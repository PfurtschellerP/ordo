//! The Med workspace's overview for the medical lead (UHS-8): one card per on-site care
//! unit with the numbers the device already holds. Numbers only; nothing is ranked.

use leptos::prelude::*;
use ordo_med_patients::{self as med, overview};

use crate::core::Handle;
use crate::text;
use crate::views::{CountCard, uhs_list};

// [impl->dsn~uhs-overview~2]
/// Every unit: patients by category against capacity, discharged, free places, open
/// orders, open and planned transports.
#[component]
pub fn OverviewView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let cards = Memo::new(move |_| {
        revision.track();
        let (summaries, details) = core.with_value(|c| {
            let incident = c.events(med::UHS_COMPARTMENT);
            (
                overview::summaries(&c.events(med::COMPARTMENT), &incident),
                med::uhs_details(&incident),
            )
        });
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let mut s = summaries.get(&u.id).cloned().unwrap_or_default();
                s.counts.uhs.clone_from(&u.id);
                let location = details.get(&u.id).and_then(|d| d.location.clone());
                (u.name, u.capacity, s, location)
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section aria-labelledby="overview-title" class="card">
            <h2 id="overview-title">{text::OVERVIEW_TITLE}</h2>
            {move || {
                let list = cards.get();
                if list.is_empty() {
                    return view! { <p>{text::UHS_EMPTY}</p> }.into_any();
                }
                // The whole incident first (MANV).
                let all = overview::total(list.iter().map(|(_, _, s, _)| s));
                let capacity: u32 = list.iter().map(|(_, c, _, _)| c).sum();
                let list = std::iter::once((text::TOTAL_TITLE.to_owned(), capacity, all, None)).chain(list);
                view! {
                    <ul class="count-list">
                        {list
                            .into_iter()
                            .map(|(name, capacity, s, location)| {
                                let places = s.places.map(|(free, total)| text::places_free(free, total));
                                view! {
                                    <CountCard name capacity counts=s.counts.clone()>
                                        // [impl->dsn~uhs-details~2]
                                        {location.map(|l| view! { <p class="count-card__location">{text::UHS_LOCATION} ": " {l}</p> })}
                                        <p class="count-card__more">
                                            {places.map(|p| format!("{}: {p} · ", text::PLACES))}
                                            {text::open_orders(s.open_orders, s.unconfirmed_orders)} " · "
                                            {text::transports_count(s.transports_open, s.transports_planned)}
                                        </p>
                                    </CountCard>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}
