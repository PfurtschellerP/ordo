//! Organisation workspace: the user's paired devices (all devices with the
//! `manage_devices` permission), and revoking a lost or stolen one (SEC-4).
//!
//! Revoking asks for confirmation in place, on the device's card, not in a modal dialog
//! (ADR-0029). The server cuts the device off at once; the device deletes its key and data
//! as soon as it connects again.

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
pub fn DevicesView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    // Ask for the list whenever the connection is (again) ready.
    Effect::new(move |_| {
        if ready.get() {
            core::list_devices(core);
        }
    });
    let all = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.has_permission(MANAGE_DEVICES))
    });
    let devices = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.devices.clone())
    });
    let own = core.with_value(|c| c.replica.device());
    let confirming = RwSignal::new(None::<DeviceId>);
    view! {
        <section aria-labelledby="devices-title" class="card">
            <h2 id="devices-title" tabindex="-1">{text::DEVICES}</h2>
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
                            <ul class="device-list">
                                {list
                                    .into_iter()
                                    .map(|info| {
                                        let own = info.device == own;
                                        view! { <DeviceCard core info own show_person confirming /> }
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

/// One device, with its facts and the revoke action.
#[component]
fn DeviceCard(
    core: Handle,
    info: DeviceInfo,
    own: bool,
    show_person: bool,
    confirming: RwSignal<Option<DeviceId>>,
) -> impl IntoView {
    let device = info.device;
    let ulid = device.ulid().to_string();
    let short = core::short_device(&device);
    let revoke_id = format!("revoke-{ulid}");
    let confirm_id = format!("revoke-confirm-{ulid}");
    let question_id = format!("revoke-question-{ulid}");
    let asking = move || confirming.get() == Some(device);
    let when = |ms: Option<u64>| ms.map_or_else(|| text::UNKNOWN.to_owned(), core::date_time);
    let mut facts = Vec::new();
    if show_person {
        facts.push((text::DEVICE_PERSON, info.user.to_string()));
    }
    facts.push((text::DEVICE_PAIRED, when(info.paired)));
    facts.push((text::DEVICE_VALID_UNTIL, when(info.valid_until)));

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
        core::revoke_device(core, device);
        // The card disappears with the server's new list; keep focus in the view.
        after_render(|| focus("devices-title"));
    };
    let label = text::revoke_label(&short);
    let question = text::revoke_question(&short, own);
    view! {
        <li class="device-card" data-device=ulid>
            <h3>
                {text::device_name(&short)}
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
