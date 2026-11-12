//! Organisation workspace: the user's paired devices (all devices with the
//! `manage_devices` permission), and revoking a lost or stolen one (SEC-4).
//!
//! Revoking asks for confirmation in place, on the device's card, not in a modal dialog
//! (ADR-0029). The server cuts the device off at once; the device deletes its key and data
//! as soon as it connects again.
//!
//! The incident command sees the devices used in its incident with the same cards and
//! blocks them there, without the organisation's list (ORG-12).

use leptos::prelude::*;
use ordo_core::DeviceId;
use ordo_sync::message::DeviceInfo;

use crate::core::{self, Handle};
use crate::text;
use crate::views::{after_render, focus};

/// Permission to see and revoke all devices of the organisation.
const MANAGE_DEVICES: &str = "manage_devices";

// [impl->dsn~device-revocation~1]
/// The device list: own devices, or all with `manage_devices`.
#[component]
pub fn DevicesView(
    core: Handle,
    revision: RwSignal<u64>,
    /// Only the person's own devices (*Meine Geräte*), also for administrators.
    #[prop(default = false)]
    mine_only: bool,
) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    // Ask for the list whenever the connection is (again) ready.
    // Ask again after every new connection (an answer may be lost when it is cut).
    Effect::new(move |_| {
        if ready.get() && welcomes.get() > 0 {
            core::list_devices(core);
        }
    });
    let all = Memo::new(move |_| {
        revision.track();
        !mine_only && core.with_value(|c| c.replica.has_permission(MANAGE_DEVICES))
    });
    let me = core.with_value(|c| c.replica.user().clone());
    let devices = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.devices.clone()).map(|list| {
            list.into_iter()
                .filter(|d| !mine_only || d.user == me)
                .collect::<Vec<_>>()
        })
    });
    let own = core.with_value(|c| c.replica.device());
    let confirming = RwSignal::new(None::<DeviceId>);
    view! {
        <section aria-labelledby="devices-title" class="card">
            <h2 id="devices-title" tabindex="-1">{if mine_only { text::MY_DEVICES } else { text::DEVICES }}</h2>
            <p>{move || if all.get() { text::DEVICES_INTRO_ALL } else { text::DEVICES_INTRO_OWN }}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::DEVICES_OFFLINE}</p> }.into_any();
                }
                match devices.get() {
                    None => view! { <p>{text::DEVICES_LOADING}</p> }.into_any(),
                    Some(list) if list.is_empty() => view! { <p>{text::DEVICES_EMPTY}</p> }.into_any(),
                    Some(list) => {
                        let show_person = all.get();
                        view! {
                            <ul class="device-list" data-keys="list">
                                {list
                                    .into_iter()
                                    .map(|info| {
                                        let own = info.device == own;
                                        view! { <DeviceCard core info own show_person confirming block=false /> }
                                    })
                                    .collect_view()}
                            </ul>
                        }
                        .into_any()
                    }
                }
            }}
        </section>
    }
}

/// Incident permission to see and block the devices used in the incident.
const BLOCK_DEVICES: &str = "block_devices";

// [impl->dsn~incident-devices~1]
/// The devices used in the open incident, for those who may block them (*Vorbereitung –
/// Personal*); nothing for everyone else.
#[component]
pub fn IncidentDevicesView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let allowed = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, BLOCK_DEVICES))
        })
    });
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    // The people of the incident: a new role brings their devices into the list.
    let people = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.incident_info().map(|i| i.roster.len()))
    });
    Effect::new(move |_| {
        people.track();
        if allowed.get() && ready.get() && welcomes.get() > 0 {
            core::list_incident_devices(core);
        }
    });
    // Whether each device is connected changes without events: ask again now and then.
    if let Ok(tick) = set_interval_with_handle(
        move || {
            if allowed.get_untracked() && ready.get_untracked() {
                core::list_incident_devices(core);
            }
        },
        std::time::Duration::from_secs(30),
    ) {
        on_cleanup(move || tick.clear());
    }
    let devices = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident_devices
                .as_ref()
                .filter(|list| c.incident.as_ref() == Some(&list.incident))
                .map(|list| list.devices.clone())
        })
    });
    let own = core.with_value(|c| c.replica.device());
    let confirming = RwSignal::new(None::<DeviceId>);
    move || {
        allowed.get().then(|| {
            view! {
                <section aria-labelledby="incident-devices-title" class="card">
                    <h2 id="incident-devices-title" tabindex="-1">{text::INCIDENT_DEVICES}</h2>
                    <p>{text::INCIDENT_DEVICES_INTRO}</p>
                    {move || {
                        if !ready.get() {
                            return view! { <p class="notice">{text::DEVICES_OFFLINE}</p> }.into_any();
                        }
                        match devices.get() {
                            None => view! { <p>{text::DEVICES_LOADING}</p> }.into_any(),
                            Some(list) if list.is_empty() => {
                                view! { <p>{text::INCIDENT_DEVICES_EMPTY}</p> }.into_any()
                            }
                            Some(list) => view! {
                                <ul class="device-list" data-keys="list">
                                    {list
                                        .into_iter()
                                        .map(|info| {
                                            let own = info.device == own;
                                            view! { <DeviceCard core info own show_person=true confirming block=true /> }
                                        })
                                        .collect_view()}
                                </ul>
                            }
                            .into_any(),
                        }
                    }}
                </section>
            }
        })
    }
}

/// One device, with its facts and the revoke action (`block`: in the open incident).
#[component]
fn DeviceCard(
    core: Handle,
    info: DeviceInfo,
    own: bool,
    show_person: bool,
    confirming: RwSignal<Option<DeviceId>>,
    block: bool,
) -> impl IntoView {
    let device = info.device;
    let ulid = device.ulid().to_string();
    let short = core::short_device(&device);
    let revoke_id = format!("revoke-{ulid}");
    let confirm_id = format!("revoke-confirm-{ulid}");
    let question_id = format!("revoke-question-{ulid}");
    let asking = move || confirming.get() == Some(device);
    let when = |ms: Option<u64>| ms.map_or_else(|| text::UNKNOWN.to_owned(), core::date_time);
    let mut facts: Vec<(&str, Signal<String>)> = Vec::new();
    if show_person {
        let user = info.user.clone();
        facts.push((
            text::DEVICE_PERSON,
            crate::profile::naming(move || crate::profile::display_name(core, &user)),
        ));
    }
    facts.push((text::DEVICE_PAIRED, when(info.paired).into()));
    facts.push((text::DEVICE_VALID_UNTIL, when(info.valid_until).into()));
    facts.push((
        text::DEVICE_CONNECTION,
        if info.connected {
            text::DEVICE_CONNECTED
        } else {
            text::DEVICE_NOT_CONNECTED
        }
        .to_owned()
        .into(),
    ));

    let ask = {
        let confirm_id = confirm_id.clone();
        move |_| {
            confirming.set(Some(device));
            let confirm_id = confirm_id.clone();
            after_render(move || focus(&confirm_id));
        }
    };
    let cancel = {
        let revoke_id = revoke_id.clone();
        move |_| {
            confirming.set(None);
            let revoke_id = revoke_id.clone();
            after_render(move || focus(&revoke_id));
        }
    };
    let revoke = move |_| {
        confirming.set(None);
        if block {
            core::block_device(core, device);
        } else {
            core::revoke_device(core, device);
        }
        // The card disappears with the server's new list; keep focus in the view.
        after_render(move || {
            focus(if block {
                "incident-devices-title"
            } else {
                "devices-title"
            })
        });
    };
    let label = text::revoke_label(&short);
    let question = text::revoke_question(&short, own);
    view! {
        <li class="device-card" data-device=ulid>
            <h3>
                {info.name.clone().map_or_else(|| text::device_name(&short), |name| text::named_device(&name, &short))}
                {own.then(|| format!(" ({})", text::THIS_DEVICE))}
            </h3>
            <dl class="facts">
                {facts
                    .into_iter()
                    .map(|(label, value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> })
                    .collect_view()}
            </dl>
            <Show
                when=move || asking()
                fallback=move || {
                    view! {
                        <button
                            id=revoke_id.clone()
                            type="button"
                            class="button device-card__action"
                            aria-label=label.clone()
                            on:click=ask.clone()
                        >
                            {text::REVOKE}
                        </button>
                    }
                }
            >
                <div class="device-card__confirm" role="group" aria-labelledby=question_id.clone()>
                    <p id=question_id.clone() class="notice notice--warning">{question.clone()}</p>
                    <div class="device-card__buttons">
                        <button id=confirm_id.clone() type="button" class="button button--danger" on:click=revoke>
                            {text::REVOKE_CONFIRM}
                        </button>
                        <button type="button" class="button" on:click=cancel.clone()>{text::CANCEL}</button>
                    </div>
                </div>
            </Show>
        </li>
    }
}
