//! The helper register on *Vorbereitung – Personal* (HELP-1, HELP-2, ADR-0071): the
//! incident's helpers with their personal data, which only the incident command reads.

use std::collections::BTreeSet;

use leptos::prelude::*;
use ordo_helpers::{FieldError, Fields, Helper, HelperChange, HelperRegister};
use ordo_units::UnitCatalogue;

use crate::core::{self, Handle};
use crate::text;
use crate::ui::{Action, Actions, CheckChips, Suggestions};
use crate::units::CATALOGUE;

/// The register, if this device may read it: granted by the server, or kept from before.
pub(crate) fn register(core: Handle) -> Option<HelperRegister> {
    let events = core.with_value(|c| c.events(ordo_helpers::COMPARTMENT));
    (core::granted(core, ordo_helpers::COMPARTMENT) || !events.is_empty())
        .then(|| HelperRegister::from_events(&events))
}

/// "im Dienst seit 10:12", "Dienst 10:12 – 18:00" or "Dienst ab 08:00".
fn duty(h: &Helper, now: u64) -> Option<String> {
    let (start, end) = (h.fields.duty_start, h.fields.duty_end);
    let at = |ms: u64| core::date_time(ms);
    match (start, end) {
        _ if h.on_duty(now) => Some(text::on_duty_since(&at(start.unwrap_or_default()))),
        (Some(s), Some(e)) => Some(text::duty_from_to(&at(s), &at(e))),
        (Some(s), None) => Some(text::duty_from(&at(s))),
        (None, Some(e)) => Some(text::duty_to(&at(e))),
        (None, None) => None,
    }
}

/// Organisation, qualifications and duty of a helper, as one line.
fn details(h: &Helper, now: u64) -> String {
    let quals = h.fields.qualifications.as_ref().map(|q| {
        q.iter()
            .map(|id| UnitCatalogue::label(&CATALOGUE.qualifications, id).to_owned())
            .collect::<Vec<_>>()
            .join(", ")
    });
    [h.fields.organisation.clone(), quals, duty(h, now)]
        .into_iter()
        .flatten()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

// [impl->dsn~helper-register~1]
/// The incident's helpers by name and the form to register or change one. Shown only to
/// those who may read the register.
#[component]
pub fn HelpersView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let helpers = Memo::new(move |_| {
        revision.track();
        register(core)
    });
    let editing = RwSignal::new(None::<Helper>);
    view! {
        <Show when=move || helpers.with(Option::is_some)>
            <section aria-labelledby="helpers-title" class="card helpers">
                <h2 id="helpers-title" tabindex="-1">{text::HELPERS_TITLE}</h2>
                <p class="hint">{text::HELPERS_INTRO}</p>
                {move || {
                    let now = core::now_ms();
                    let list: Vec<Helper> = helpers
                        .get()
                        .map(|r| r.by_name().into_iter().cloned().collect())
                        .unwrap_or_default();
                    if list.is_empty() {
                        return view! { <p>{text::HELPERS_EMPTY}</p> }.into_any();
                    }
                    view! {
                        <ul class="roster helper-list" data-keys="list">
                            {list
                                .into_iter()
                                .map(|h| {
                                    let name = h.name().to_owned();
                                    let line = details(&h, now);
                                    let account = h.fields.account.clone();
                                    let on_duty = h.on_duty(now);
                                    let edit = h.clone();
                                    let id = h.id.clone();
                                    let duty_action = Action {
                                        text: if on_duty { text::DUTY_END } else { text::DUTY_START },
                                        label: if on_duty { text::duty_end_label(&name) } else { text::duty_start_label(&name) },
                                        quiet: true,
                                        run: Callback::new({
                                            let name = name.clone();
                                            move |()| {
                                                let now = core::now_ms();
                                                let fields = if on_duty {
                                                    Fields { duty_end: Some(now), ..Fields::default() }
                                                } else {
                                                    // A new duty: started now, an earlier end cleared.
                                                    Fields { duty_start: Some(now), duty_end: Some(0), ..Fields::default() }
                                                };
                                                let change = HelperChange { helper: id.clone(), fields };
                                                let _ = core::helper_event(core, revision, false, &change, &name);
                                            }
                                        }),
                                    };
                                    view! {
                                        <li class="roster__entry" data-helper=h.id.clone()>
                                            <span class="roster__person">{name.clone()}</span>
                                            {(!line.is_empty()).then(|| view! { <span class="roster__id">{line}</span> })}
                                            {account.map(|a| view! { <span class="roster__id">{text::helper_account(&a)}</span> })}
                                            <span class="row helper-list__actions">
                                                <button type="button" class="button button--small" aria-label=text::helper_edit_label(&name)
                                                    on:click=move |_| {
                                                        editing.set(Some(edit.clone()));
                                                        crate::views::focus("helper-name");
                                                    }>
                                                    {text::EDIT}
                                                </button>
                                                <Actions actions=vec![duty_action] />
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                    .into_any()
                }}
                {move || {
                    // A new form for each helper edited, filled with them.
                    let helper = editing.get();
                    view! { <HelperForm core revision editing helper /> }
                }}
            </section>
        </Show>
    }
}

/// The form's text for a field: its current value or empty.
fn shown(value: Option<&String>) -> String {
    value.cloned().unwrap_or_default()
}

/// What changed: `Some` for a field typed differently from `before` (an empty text clears
/// it), `None` where it is the same.
fn changed(typed: String, before: Option<&String>) -> Option<String> {
    let typed = typed.trim().to_owned();
    (typed != before.map(|b| b.trim()).unwrap_or_default()).then_some(typed)
}

/// Registers a helper, or changes the one in `helper`.
#[component]
fn HelperForm(
    core: Handle,
    revision: RwSignal<u64>,
    editing: RwSignal<Option<Helper>>,
    helper: Option<Helper>,
) -> impl IntoView {
    let before = helper
        .as_ref()
        .map(|h| h.fields.clone())
        .unwrap_or_default();
    let name = RwSignal::new(shown(before.name.as_ref()));
    let organisation = RwSignal::new(shown(before.organisation.as_ref()));
    let phone = RwSignal::new(shown(before.phone.as_ref()));
    let address = RwSignal::new(shown(before.address.as_ref()));
    let birth_date = RwSignal::new(shown(before.birth_date.as_ref()));
    let account = RwSignal::new(shown(before.account.as_ref()));
    let time = |ms: Option<u64>| RwSignal::new(ms.map(core::local_input_time).unwrap_or_default());
    let (start, end) = (time(before.duty_start), time(before.duty_end));
    let qualifications = RwSignal::new(
        before
            .qualifications
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>(),
    );
    let error = RwSignal::new(None::<&'static str>);
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    let id = helper.as_ref().map(|h| h.id.clone());
    let title = match &helper {
        Some(h) => text::helper_edit_title(h.name()),
        None => text::HELPER_ADD_TITLE.to_owned(),
    };
    let editing_one = id.is_some();
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let ms = |s: RwSignal<String>| {
            let typed = s.get_untracked();
            if typed.trim().is_empty() {
                Ok(None)
            } else {
                core::local_time_ms(typed.trim()).map(Some).ok_or(())
            }
        };
        let (Ok(duty_start), Ok(duty_end)) = (ms(start), ms(end)) else {
            error.set(Some(text::HELPER_TIME));
            return;
        };
        let quals: Vec<String> = CATALOGUE
            .qualifications
            .iter()
            .filter(|q| qualifications.with_untracked(|s| s.contains(&q.id)))
            .map(|q| q.id.clone())
            .collect();
        let registration = id.is_none();
        let fields = if registration {
            let given = |s: RwSignal<String>| {
                Some(s.get_untracked().trim().to_owned()).filter(|t| !t.is_empty())
            };
            Fields {
                name: given(name),
                organisation: given(organisation),
                phone: given(phone),
                address: given(address),
                birth_date: given(birth_date),
                account: given(account),
                qualifications: (!quals.is_empty()).then_some(quals),
                duty_start,
                duty_end,
            }
        } else {
            let was_quals = before.qualifications.clone().unwrap_or_default();
            // A time cleared is 0 (see `Fields`).
            let time_changed =
                |now: Option<u64>, was: Option<u64>| (now != was).then_some(now.unwrap_or(0));
            Fields {
                name: changed(name.get_untracked(), before.name.as_ref()),
                organisation: changed(organisation.get_untracked(), before.organisation.as_ref()),
                phone: changed(phone.get_untracked(), before.phone.as_ref()),
                address: changed(address.get_untracked(), before.address.as_ref()),
                birth_date: changed(birth_date.get_untracked(), before.birth_date.as_ref()),
                account: changed(account.get_untracked(), before.account.as_ref()),
                qualifications: (quals != was_quals).then_some(quals),
                duty_start: time_changed(duty_start, before.duty_start),
                duty_end: time_changed(duty_end, before.duty_end),
            }
        };
        let helper = match &id {
            Some(id) => id.clone(),
            None => match core::new_ulid() {
                Some(u) => u.to_string(),
                None => {
                    error.set(Some(text::HELPER_NOT_SAVED));
                    return;
                }
            },
        };
        let change = HelperChange { helper, fields };
        if let Err(e) = change.check(registration) {
            error.set(Some(match e {
                FieldError::NameMissing => text::HELPER_NAME_MISSING,
                FieldError::TooLong => text::HELPER_TOO_LONG,
                FieldError::NotADate => text::HELPER_BIRTH_DATE,
                FieldError::NotAnAccount => text::HELPER_ACCOUNT,
            }));
            crate::views::focus(match e {
                FieldError::NameMissing | FieldError::TooLong => "helper-name",
                FieldError::NotADate => "helper-birth-date",
                FieldError::NotAnAccount => "helper-account",
            });
            return;
        }
        let shown_name = name.get_untracked().trim().to_owned();
        if core::helper_event(core, revision, registration, &change, &shown_name).is_err() {
            error.set(Some(text::HELPER_NOT_SAVED));
            return;
        }
        error.set(None);
        if editing_one {
            editing.set(None); // a fresh form for the next one
        } else {
            for s in [
                name,
                organisation,
                phone,
                address,
                birth_date,
                account,
                start,
                end,
            ] {
                s.set(String::new());
            }
            qualifications.set(BTreeSet::new());
            crate::views::focus("helper-name");
        }
    };
    let options: Vec<(String, String)> = CATALOGUE
        .qualifications
        .iter()
        .map(|q| (q.id.clone(), q.label.clone()))
        .collect();
    let text_field = |id: &'static str,
                      label: &'static str,
                      value: RwSignal<String>,
                      autocomplete: &'static str| {
        view! {
            <div class="field">
                <label for=id>{label}</label>
                <input id=id type="text" autocomplete=autocomplete maxlength=ordo_helpers::TEXT_MAX.to_string() bind:value=value />
            </div>
        }
    };
    view! {
        <form class="form helper-form" novalidate on:submit=submit aria-labelledby="helper-form-title">
            <h3 id="helper-form-title">{title}</h3>
            <div class="form--inline">
                {text_field("helper-name", text::HELPER_NAME, name, "off")}
                {text_field("helper-organisation", text::UNIT_ORGANISATION, organisation, "off")}
                {text_field("helper-phone", text::HELPER_PHONE, phone, "off")}
                <div class="field">
                    <label for="helper-birth-date">{text::HELPER_BIRTH_DATE_LABEL}</label>
                    <input id="helper-birth-date" type="date" bind:value=birth_date />
                </div>
            </div>
            {text_field("helper-address", text::HELPER_ADDRESS, address, "off")}
            <CheckChips legend=text::CREW_QUALIFICATIONS options value=qualifications />
            <div class="form--inline">
                <div class="field">
                    <label for="helper-duty-start">{text::HELPER_DUTY_START}</label>
                    <input id="helper-duty-start" type="datetime-local" bind:value=start />
                </div>
                <div class="field">
                    <label for="helper-duty-end">{text::HELPER_DUTY_END}</label>
                    <input id="helper-duty-end" type="datetime-local" bind:value=end />
                </div>
                <div class="field">
                    <label for="helper-account">{text::HELPER_ACCOUNT_LABEL}</label>
                    <input id="helper-account" type="text" autocomplete="off" list="helper-account-people" bind:value=account />
                    <Suggestions id="helper-account-people" values=people core />
                </div>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">
                    {if editing_one { text::SAVE } else { text::HELPER_ADD }}
                </button>
                {editing_one.then(|| view! {
                    <button type="button" class="button" on:click=move |_| editing.set(None)>{text::CANCEL}</button>
                })}
            </div>
        </form>
    }
}
