//! A UHS asks for a physician or a paramedic (UHS-15): its staff ask in *UHS – Patienten*;
//! the incident command sees the request in *Disposition* and in the bell, puts a person on
//! the UHS's staff list (which gives them access to its patients, UHS-10) and closes it.

use leptos::prelude::*;
use ordo_med_patients::staffing::{self, Closed, Need, Requested, StaffRequests};
use ordo_med_patients::{self as med, UhsUpdated};

use crate::core::{self, Handle};
use crate::text;

/// The incident's staff requests, computed from the shared events.
pub fn list(core: Handle) -> StaffRequests {
    core.with_value(|c| StaffRequests::from_events(&c.events(staffing::COMPARTMENT)))
}

/// Records a request of `uhs` for `need`.
fn request(core: Handle, revision: RwSignal<u64>, uhs: &str, need: Need) {
    let Some(id) = core::new_ulid() else { return };
    let payload = Requested {
        request: id.to_string(),
        uhs: uhs.to_owned(),
        need,
    }
    .encode();
    if let Ok(payload) = payload {
        let _ = core::record_shared(
            core,
            revision,
            staffing::MODULE,
            staffing::REQUESTED,
            &payload,
            text::staff_requested(&core::uhs_name(core, uhs), need),
        );
    }
}

/// Closes a request: met by `person` (put on the staff list first), or withdrawn.
fn close(core: Handle, revision: RwSignal<u64>, request: &str, person: Option<String>) {
    if let Ok(payload) = (Closed {
        request: request.to_owned(),
        person,
    })
    .encode()
    {
        let _ = core::record_shared(
            core,
            revision,
            staffing::MODULE,
            staffing::CLOSED,
            &payload,
            text::STAFF_REQUEST_CLOSED.to_owned(),
        );
    }
}

// [impl->dsn~uhs-staff-request~1]
/// In *UHS – Patienten*, for the UHS the person works at: ask for a physician or a
/// paramedic, and see the requests and who comes.
#[component]
pub fn StaffRequestPanel(
    core: Handle,
    revision: RwSignal<u64>,
    uhs: Signal<String>,
) -> impl IntoView {
    let requests = Memo::new(move |_| {
        revision.track();
        let uhs = uhs.get();
        list(core)
            .of_uhs(&uhs)
            .into_iter()
            .take(3)
            .map(|r| {
                let state = match &r.closed {
                    None => text::STAFF_REQUEST_OPEN.to_owned(),
                    Some((_, _, Some(p))) => {
                        text::staff_request_met(&crate::profile::display_text(core, p))
                    }
                    Some((_, _, None)) => text::STAFF_REQUEST_WITHDRAWN.to_owned(),
                };
                (
                    text::staff_need(r.requested.need),
                    core::clock(r.at.wall_ms),
                    state,
                )
            })
            .collect::<Vec<_>>()
    });
    view! {
        <Show when=move || !uhs.get().is_empty()>
            <section class="card staff-requests" aria-labelledby="staff-requests-title">
                <h2 id="staff-requests-title">{text::STAFF_REQUESTS_TITLE}</h2>
                <div class="row">
                    {[Need::Physician, Need::Paramedic].into_iter().map(|need| view! {
                        <button type="button" class="button" on:click=move |_| request(core, revision, &uhs.get_untracked(), need)>
                            {text::staff_request_button(need)}
                        </button>
                    }).collect_view()}
                </div>
                {move || {
                    let list = requests.get();
                    (!list.is_empty()).then(|| view! {
                        <ul class="staff-requests__list">
                            {list.into_iter().map(|(need, at, state)| view! { <li>{need} " · " {at} " · " {state}</li> }).collect_view()}
                        </ul>
                    })
                }}
            </section>
        </Show>
    }
}

// [impl->dsn~uhs-staff-request~1]
/// In *Führung – Disposition*: the open requests of the UHS; the command names a person,
/// who goes on the UHS's staff list, and the request is closed.
#[component]
pub fn StaffRequestsBoard(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let open = Memo::new(move |_| {
        revision.track();
        list(core).open().into_iter().cloned().collect::<Vec<_>>()
    });
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    view! {
        <Show when=move || open.with(|o| !o.is_empty())>
            <section class="card staff-board" aria-labelledby="staff-board-title">
                <h2 id="staff-board-title">{text::STAFF_BOARD_TITLE}</h2>
                <ul class="report-list">
                    {move || open.get().into_iter().map(|r| {
                        let id = r.requested.request.clone();
                        let uhs_id = r.requested.uhs.clone();
                        let uhs = core::uhs_name(core, &uhs_id);
                        let need = r.requested.need;
                        let field = format!("staff-{id}");
                        let person = RwSignal::new(String::new());
                        let label = text::staff_request_label(&uhs, need);
                        let (send_id, drop_id) = (id.clone(), id.clone());
                        let send = move |ev: leptos::ev::SubmitEvent| {
                            ev.prevent_default();
                            let who = person.get_untracked().trim().to_owned();
                            if who.is_empty() {
                                return;
                            }
                            // On the staff list, kept once; then the request is met.
                            let mut staff = core.with_value(|c| {
                                med::uhs_details(&c.events(med::UHS_COMPARTMENT))
                                    .remove(&uhs_id)
                                    .map(|d| d.staff)
                                    .unwrap_or_default()
                            });
                            staff.retain(|s| !ordo_units::same_person(s, &who));
                            staff.push(who.clone());
                            let update = UhsUpdated { id: uhs_id.clone(), staff: Some(staff), ..Default::default() };
                            if core::update_uhs(core, revision, &update).is_ok() {
                                close(core, revision, &send_id, Some(who));
                            }
                        };
                        view! {
                            <li class="report-card" data-request=id.clone()>
                                <p><strong>{label.clone()}</strong> " · " {core::clock(r.at.wall_ms)}</p>
                                <form class="form--inline" novalidate on:submit=send>
                                    <div class="field">
                                        <label for=field.clone()>{text::STAFF_SEND}</label>
                                        <input id=field.clone() type="text" autocomplete="off" list=format!("{field}-people") bind:value=person />
                                        <crate::ui::Suggestions id=format!("{field}-people") values=people core />
                                    </div>
                                    <button type="submit" class="button button--small">{text::STAFF_ASSIGN}</button>
                                    <button type="button" class="button button--small button--quiet"
                                        aria-label=text::staff_withdraw_label(&label)
                                        on:click=move |_| close(core, revision, &drop_id, None)>
                                        {text::STAFF_WITHDRAW}
                                    </button>
                                </form>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </section>
        </Show>
    }
}
