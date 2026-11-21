//! *Sanität – Besatzung* (UNIT-9, ADR-0079): the crew's own workspace. A unit's crew
//! sets its status, moves its tasks and transports on and takes its patient cases over; it
//! sees only the units whose crew names the person's account.

use leptos::prelude::*;
use ordo_med_patients::transport::{TransportBoard, TransportStatus};
use ordo_units::{Unit, UnitRegister};

use crate::core::{self, Handle};
use crate::text;

/// The units present whose crew names this person (by their account).
pub(crate) fn my_units(core: Handle) -> Vec<Unit> {
    core.with_value(|c| {
        let me = c.replica.user().to_string();
        UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
            .crewed_by(&me)
            .cloned()
            .collect()
    })
}

/// Says that this person is on no crew, and who puts them on one.
fn no_crew() -> AnyView {
    view! { <section class="card"><p>{text::CREW_NONE_HERE}</p></section> }.into_any()
}

// [impl->dsn~crew-workspace~3]
/// View *Aufträge*: first the open Aufträge for this person (UHS-15), then per unit its
/// status, set here, and its open Aufträge, one card each with place, map and, on a patient
/// Auftrag, "Patient aufnehmen" (REP-3, REP-4).
#[component]
pub fn CrewTasks(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let state = Memo::new(move |_| {
        revision.track();
        let now = core::now_ms();
        let tasks = crate::shared::tasks::task_list(core);
        let me = core.with_value(|c| c.replica.user().to_string());
        let mine: Vec<ordo_tasks::Task> = tasks.of_person(&me).into_iter().cloned().collect();
        let units = my_units(core)
            .into_iter()
            .map(|u| {
                let open: Vec<ordo_tasks::Task> =
                    tasks.of_unit(&u.unit).into_iter().cloned().collect();
                (u, open)
            })
            .collect::<Vec<_>>();
        (mine, units, now)
    });
    view! {
        {move || {
            let (mine, units, now) = state.get();
            if mine.is_empty() && units.is_empty() {
                return no_crew();
            }
            let mine = (!mine.is_empty()).then(|| view! {
                <section class="card crew-mine" aria-labelledby="crew-mine-title">
                    <h2 id="crew-mine-title">{text::MY_TASKS}</h2>
                    <ul class="task-list" data-keys="list">
                        {mine.into_iter().map(|task| view! {
                            <crate::shared::tasks::TaskItem core revision task editable=true now crew=true />
                        }).collect_view()}
                    </ul>
                </section>
            });
            let units = units.into_iter()
                .map(|(u, open)| {
                    let name = u.unit.clone();
                    let current = u.status.as_ref().map(|s| s.status.clone()).unwrap_or_default();
                    let title = format!("crew-{}", core::slug(&name));
                    let labelled = title.clone();
                    view! {
                        <section class="card crew-unit" aria-labelledby=labelled data-unit=name.clone()>
                            <h2 id=title>{name.clone()}</h2>
                            <p class="unit-card__details">{crate::shared::units::details(&u)}</p>
                            <p class="unit-status__current">{text::COL_STATUS} ": " <strong>{crate::shared::units::status_shown(&u)}</strong></p>
                            <crate::shared::units::StatusSelect core revision unit=name.clone() current />
                            <h3>{text::tasks_title(open.len())}</h3>
                            {if open.is_empty() {
                                view! { <p>{text::TASKS_EMPTY}</p> }.into_any()
                            } else {
                                view! {
                                    <ul class="task-list" data-keys="list">
                                        {open.into_iter().map(|task| view! {
                                            <crate::shared::tasks::TaskItem core revision task editable=true now crew=true unit=name.clone() />
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </section>
                    }
                })
                .collect_view();
            view! { {mine} {units} }.into_any()
        }}
    }
}

// [impl->dsn~crew-workspace~3]
/// View *Transporte*: the open transports assigned to the person's units, with their next
/// steps (TRANS-2).
#[component]
pub fn CrewTransports(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let transports = Memo::new(move |_| {
        revision.track();
        let units: Vec<String> = my_units(core).into_iter().map(|u| u.unit).collect();
        if units.is_empty() {
            return None;
        }
        let board = core.with_value(|c| {
            TransportBoard::from_events(&c.events(ordo_med_patients::transport::COMPARTMENT))
        });
        let mut list: Vec<_> = board
            .transports
            .into_values()
            .filter(|t| {
                t.unit.as_ref().is_some_and(|u| units.contains(u))
                    && !matches!(
                        t.status,
                        TransportStatus::HandedOver | TransportStatus::Cancelled
                    )
            })
            .collect();
        list.sort_by_key(|t| t.requested_at());
        Some(list)
    });
    view! {
        {move || match transports.get() {
            None => no_crew(),
            Some(list) => view! {
                <section class="card crew-transports" aria-labelledby="crew-transports-title">
                    <h2 id="crew-transports-title">{text::CREW_TRANSPORTS}</h2>
                    {if list.is_empty() {
                        view! { <p>{text::TRANSPORTS_EMPTY}</p> }.into_any()
                    } else {
                        view! {
                            <ul class="transport-list" data-keys="list">
                                {list.into_iter().map(|t| {
                                    let request = t.request.request.clone();
                                    let shown = t.clone();
                                    view! {
                                        <li class="transport-card" data-request=request>
                                            <crate::med::transports::TransportDetails core transport=shown show_uhs=true />
                                            <crate::med::transports::StepButtons core revision transport=t />
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>
            }
            .into_any(),
        }}
    }
}
