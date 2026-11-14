//! The incident's people on *Vorbereitung – Personal* (HELP-1, HELP-2, HELP-6, ADR-0071,
//! ADR-0076): the helper register with its personal data, which only the incident command
//! reads, and the roles in the incident, in one list and one form.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_helpers::{FieldError, Fields, Helper, HelperChange, HelperRegister};
use ordo_units::UnitCatalogue;

use crate::core::{self, Handle};
use crate::text;
use crate::ui::{Action, Actions, CheckChips, Suggestions};
use crate::units::CATALOGUE;

/// Formats of registration cards from the profile (`helpers/registration-cards.toml`,
/// checked at build time; ADR-0072).
static CARDS: LazyLock<ordo_helpers::cards::CardCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/registration-cards.json"
    )))
    .expect("card formats were checked at build time")
});

/// Settings of the profile for the register (`helpers/register.toml`, checked at build
/// time): phone numbers, postcodes, diets.
static SETTINGS: LazyLock<ordo_helpers::register::RegisterSettings> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/register.json")))
        .expect("register settings were checked at build time")
});

thread_local! {
    /// Roles chosen for helpers without an account, by helper ID, until the server answers
    /// with their guest ID (ADR-0077).
    static PENDING_ROLES: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

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

/// The diets of the helpers whose duty has not ended, most frequent first, those without
/// one last; spelling as first seen, counted without case. Empty if nobody gave one.
fn diet_counts(helpers: &[Helper], now: u64) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    let mut none = 0;
    for h in helpers
        .iter()
        .filter(|h| h.fields.duty_end.is_none_or(|e| e > now))
    {
        match h.fields.diet.as_deref().map(str::trim) {
            Some(d) if !d.is_empty() => {
                match counts
                    .iter_mut()
                    .find(|(k, _)| k.to_lowercase() == d.to_lowercase())
                {
                    Some((_, n)) => *n += 1,
                    None => counts.push((d.to_owned(), 1)),
                }
            }
            _ => none += 1,
        }
    }
    if counts.is_empty() {
        return counts;
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if none > 0 {
        counts.push((text::DIET_NONE.to_owned(), none));
    }
    counts
}

/// One person of the incident, as the list shows them: a helper of the register, a holder
/// of roles in the roster, or both, joined by the ORDO account (ADR-0076).
#[derive(Clone, PartialEq)]
struct Person {
    /// The register's entry, if this device may read it and there is one.
    helper: Option<Helper>,
    /// The ORDO account.
    account: Option<String>,
    /// Roles in the incident (by the account).
    roles: Vec<String>,
}

/// The people of the incident: every helper of the register (if readable) with the roles
/// of their account, then everyone with a role who is not in the register.
fn people(core: Handle, register: Option<&HelperRegister>) -> Vec<Person> {
    let roster: Vec<(String, Vec<String>)> = core.with_value(|c| {
        c.incident_info()
            .map(|i| {
                i.roster
                    .iter()
                    .filter(|(_, roles)| !roles.is_empty())
                    .map(|(u, roles)| (u.to_string(), roles.iter().cloned().collect()))
                    .collect()
            })
            .unwrap_or_default()
    });
    let roles_of = |account: &str| {
        roster
            .iter()
            .find(|(u, _)| u.eq_ignore_ascii_case(account))
            .map(|(_, r)| r.clone())
            .unwrap_or_default()
    };
    let mut list: Vec<Person> = register
        .map(|r| r.by_name())
        .unwrap_or_default()
        .into_iter()
        .map(|h| Person {
            account: h.fields.account.clone(),
            roles: h
                .fields
                .account
                .as_deref()
                .map(roles_of)
                .unwrap_or_default(),
            helper: Some(h.clone()),
        })
        .collect();
    for (user, roles) in &roster {
        let known = list.iter().any(|p| {
            p.account
                .as_deref()
                .is_some_and(|a| a.eq_ignore_ascii_case(user))
        });
        if !known {
            list.push(Person {
                helper: None,
                account: Some(user.clone()),
                roles: roles.clone(),
            });
        }
    }
    list
}

/// The name a person is shown by: the register's, unless it is only the account; then
/// the account's display name (ORG-7).
fn shown_name(core: Handle, p: &Person) -> String {
    match (&p.helper, &p.account) {
        (Some(h), Some(a)) if !h.name().eq_ignore_ascii_case(a) => h.name().to_owned(),
        (_, Some(a)) => crate::profile::display_text(core, a),
        (Some(h), None) => h.name().to_owned(),
        (None, None) => String::new(),
    }
}

// [impl->dsn~helper-register~3]
// [impl->dsn~incident-roles~1]
/// *Vorbereitung – Personal* (ADR-0076): the incident's people in one list, each with their
/// roles in the incident, and one form that registers a helper and grants their role. The
/// register's personal data shows only where this device may read it; without it (e.g. a
/// planner of another incident) the list holds the roles and the form grants them.
#[component]
pub fn HelpersView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let helpers = Memo::new(move |_| {
        revision.track();
        register(core)
    });
    let readable = Memo::new(move |_| helpers.with(Option::is_some));
    let manage = Memo::new(move |_| {
        revision.track();
        crate::incidents::may_manage(core)
    });
    // [impl->dsn~person-suggestions~1]
    // Those who grant roles get the people the server knows as suggestions.
    Effect::new(move || {
        if manage.get() {
            core::list_people(core);
        }
    });
    let list = Memo::new(move |_| {
        revision.track();
        helpers.with(|h| people(core, h.as_ref()))
    });
    let editing = RwSignal::new(None::<Person>);
    let error = RwSignal::new(None::<String>);
    // [impl->dsn~guest-access~1]
    // The server's answer to a guest code asked on this device (ADR-0077): the role chosen
    // goes to the guest's ID, the helper is linked to it, and the code is shown.
    let shown_pass = RwSignal::new(None::<(ordo_sync::message::GuestPass, String)>);
    Effect::new(move || {
        revision.track();
        // Taken once: a view drawn anew does not show it again.
        let Some(pass) = core.try_update_value(|c| c.guest_pass.take()).flatten() else {
            return;
        };
        let Some(guest) = pass.guest.clone() else {
            PENDING_ROLES.with_borrow_mut(|p| p.remove(&pass.tag));
            error.set(Some(text::GUEST_CODE_REFUSED.to_owned()));
            return;
        };
        let name = helpers
            .with_untracked(|r| {
                r.as_ref()
                    .and_then(|r| r.helpers.get(&pass.tag).map(|h| h.name().to_owned()))
            })
            .unwrap_or_default();
        if let Some(role) = PENDING_ROLES.with_borrow_mut(|p| p.remove(&pass.tag)) {
            let _ = core::change_role(core, revision, guest.as_str(), &role, true);
            let link = HelperChange {
                helper: pass.tag.clone(),
                fields: Fields {
                    account: Some(guest.to_string()),
                    ..Fields::default()
                },
            };
            let _ = core::helper_event(core, revision, false, &link, &name);
        }
        shown_pass.set(Some((pass, name)));
        request_animation_frame(|| crate::views::focus("guest-pass-title"));
    });
    view! {
        <section aria-labelledby="helpers-title" class="card helpers">
            <h2 id="helpers-title" tabindex="-1">{text::HELPERS_TITLE}</h2>
            <p class="hint">{move || if readable.get() { text::HELPERS_INTRO } else { text::HELPERS_INTRO_ROLES }}</p>
            {move || {
                let now = core::now_ms();
                let people = list.get();
                if people.is_empty() {
                    return view! { <p>{text::HELPERS_EMPTY}</p> }.into_any();
                }
                let registered: Vec<Helper> = people.iter().filter_map(|p| p.helper.clone()).collect();
                let diets = diet_counts(&registered, now);
                view! {
                    {(!diets.is_empty()).then(|| view! { <p class="helper-diets">{text::diet_counts(&diets)}</p> })}
                    <ul class="roster helper-list" data-keys="list">
                        {people
                            .into_iter()
                            .map(|p| view! { <PersonEntry core revision person=p editing error manage readable now /> })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <p role="alert" class="error">{move || error.get()}</p>
            {move || shown_pass.get().map(|(pass, name)| view! { <GuestPassView pass name shown=shown_pass /> })}
            <Show when=move || manage.get() || readable.get()>
                {move || {
                    // A new form for each person edited, filled with them.
                    let person = editing.get();
                    view! { <HelperForm core revision editing person readable=readable.get() manage=manage.get() /> }
                }}
            </Show>
        </section>
    }
}

/// One person of the list: name, account, details, roles (withdrawn here) and actions.
#[component]
fn PersonEntry(
    core: Handle,
    revision: RwSignal<u64>,
    person: Person,
    editing: RwSignal<Option<Person>>,
    error: RwSignal<Option<String>>,
    manage: Memo<bool>,
    readable: Memo<bool>,
    now: u64,
) -> impl IntoView {
    let name = shown_name(core, &person);
    let line = person
        .helper
        .as_ref()
        .map(|h| details(h, now))
        .unwrap_or_default();
    let account = person.account.clone();
    let guest = person
        .account
        .as_deref()
        .and_then(|a| a.parse::<ordo_core::UserId>().ok())
        .filter(ordo_core::UserId::is_guest);
    let duty_action = person.helper.as_ref().map(|h| {
        let on_duty = h.on_duty(now);
        let id = h.id.clone();
        let name = name.clone();
        let guest = guest.clone();
        Action {
            text: if on_duty {
                text::DUTY_END
            } else {
                text::DUTY_START
            },
            label: if on_duty {
                text::duty_end_label(&name)
            } else {
                text::duty_start_label(&name)
            },
            quiet: true,
            run: Callback::new(move |()| {
                let now = core::now_ms();
                let fields = if on_duty {
                    Fields {
                        duty_end: Some(now),
                        ..Fields::default()
                    }
                } else {
                    // A new duty: started now, an earlier end cleared.
                    Fields {
                        duty_start: Some(now),
                        duty_end: Some(0),
                        ..Fields::default()
                    }
                };
                let change = HelperChange {
                    helper: id.clone(),
                    fields,
                };
                let _ = core::helper_event(core, revision, false, &change, &name);
                // [impl->dsn~guest-access~1]
                // A guest's access ends with their duty (ADR-0077).
                if on_duty && let Some(guest) = guest.clone() {
                    core::set_guest_access(core, guest, now);
                }
            }),
        }
    });
    // A new code for a guest, e.g. for another phone; their access still ends with duty.
    let guest_action = guest
        .clone()
        .zip(person.helper.clone())
        .map(|(user, h)| Action {
            text: text::GUEST_ACCESS,
            label: text::guest_access_label(&name),
            quiet: true,
            run: Callback::new(
                move |()| match h.fields.duty_end.filter(|e| *e > core::now_ms()) {
                    Some(until) => {
                        core::create_guest(core, Some(user.clone()), until, &h.id, h.name())
                    }
                    None => error.set(Some(text::GUEST_NEEDS_DUTY_END.to_owned())),
                },
            ),
        });
    let guest_until = guest
        .as_ref()
        .and(person.helper.as_ref())
        .and_then(|h| h.fields.duty_end)
        .map(|e| text::guest_until(&core::date_time(e)));
    let edit = person.clone();
    let registered = person.helper.is_some();
    let edit_label = if registered {
        text::helper_edit_label(&name)
    } else {
        text::helper_complete_label(&name)
    };
    let roles = person.roles.clone();
    view! {
        <li class="roster__entry" data-helper=person.helper.as_ref().map(|h| h.id.clone()) data-user=account.clone()>
            {account.as_deref().and_then(|a| a.parse::<ordo_core::UserId>().ok()).map(|id| view! {
                <crate::profile::Avatar core revision user=id />
            })}
            <span class="roster__person">{crate::profile::naming({
                let person = person.clone();
                move || shown_name(core, &person)
            })}</span>
            {account.clone().filter(|_| guest.is_none()).map(|a| view! { <span class="roster__id">{a}</span> })}
            {(!line.is_empty()).then(|| view! { <span class="roster__id">{line}</span> })}
            {guest_until.map(|t| view! { <span class="roster__id helper-list__guest">{t}</span> })}
            {(!roles.is_empty()).then(|| view! {
                <ul class="roster__roles">
                    {roles.into_iter().map(|role| {
                        let user = account.clone().unwrap_or_default();
                        let label = text::withdraw_label(&user, &role);
                        let (u, r) = (user.clone(), role.clone());
                        view! {
                            <li class="roster__role">
                                {text::role_label(&role)}
                                <Show when=move || manage.get()>
                                    <button type="button" class="button button--small" aria-label=label.clone()
                                        on:click={
                                            let (u, r) = (u.clone(), r.clone());
                                            move |_| {
                                                if let Err(e) = core::change_role(core, revision, &u, &r, false) {
                                                    error.set(Some(e));
                                                }
                                            }
                                        }>
                                        {text::WITHDRAW}
                                    </button>
                                </Show>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            })}
            <span class="row helper-list__actions">
                <Show when=move || readable.get()>
                    <button type="button" class="button button--small" aria-label=edit_label.clone()
                        on:click={
                            let edit = edit.clone();
                            move |_| {
                                editing.set(Some(edit.clone()));
                                crate::views::focus("helper-name");
                            }
                        }>
                        {if registered { text::EDIT } else { text::HELPER_COMPLETE }}
                    </button>
                </Show>
                {
                    let actions: Vec<Action> = duty_action
                        .into_iter()
                        .chain(guest_action.filter(|_| manage.get_untracked()))
                        .collect();
                    (!actions.is_empty()).then(|| view! { <Actions actions /> })
                }
            </span>
        </li>
    }
}

// [impl->dsn~guest-access~1]
/// The one-time code of a guest (ADR-0077), as QR code and link, for their phone; shown
/// only on this device, until closed.
#[component]
fn GuestPassView(
    pass: ordo_sync::message::GuestPass,
    name: String,
    shown: RwSignal<Option<(ordo_sync::message::GuestPass, String)>>,
) -> impl IntoView {
    use base64::Engine;
    let image = pass.qr.as_ref().map(|svg| {
        format!(
            "data:image/svg+xml;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(svg)
        )
    });
    let link = pass.link.clone().unwrap_or_default();
    view! {
        <section class="notice guest-pass" aria-labelledby="guest-pass-title">
            <h3 id="guest-pass-title" tabindex="-1">{text::GUEST_CODE_TITLE}</h3>
            <p>{text::guest_code_for(&name, &core::clock(pass.expires))}</p>
            {image.map(|src| view! { <img class="pairing-qr__code" src=src alt=text::GUEST_CODE_ALT width="256" height="256" /> })}
            <p>{text::GUEST_LINK} " " <a href=link.clone() class="guest-pass__link">{link.clone()}</a></p>
            <div class="row">
                <button type="button" class="button" on:click=move |_| shown.set(None)>{text::GUEST_CODE_CLOSE}</button>
            </div>
        </section>
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

/// The choice of the diet field for a diet: one of the profile's, "other" (typed), or none.
const DIET_OTHER: &str = "other";

fn diet_choice(diet: &str) -> (String, String) {
    let diet = diet.trim();
    if diet.is_empty() {
        (String::new(), String::new())
    } else if let Some(d) = SETTINGS.diets.iter().find(|d| d.eq_ignore_ascii_case(diet)) {
        (d.clone(), String::new())
    } else {
        (DIET_OTHER.to_owned(), diet.to_owned())
    }
}

// [impl->dsn~helper-register~3]
/// Registers a person, or changes the one in `person` (ADR-0076): their ORDO account first,
/// which fills in the name, then their personal data and a role in the incident, granted
/// with the registration. Without access to the register only account and role are asked.
#[component]
fn HelperForm(
    core: Handle,
    revision: RwSignal<u64>,
    editing: RwSignal<Option<Person>>,
    person: Option<Person>,
    /// Whether this device may read and write the register.
    readable: bool,
    /// Whether this person may grant roles.
    manage: bool,
) -> impl IntoView {
    let helper = person.as_ref().and_then(|p| p.helper.clone());
    let before = helper
        .as_ref()
        .map(|h| h.fields.clone())
        .unwrap_or_default();
    let name = RwSignal::new(shown(before.name.as_ref()));
    let organisation = RwSignal::new(shown(before.organisation.as_ref()));
    let phone = RwSignal::new(shown(before.phone.as_ref()));
    // An address kept in one text by an older version is shown as the street.
    let street = RwSignal::new(shown(before.street.as_ref().or(before.address.as_ref())));
    let postcode = RwSignal::new(shown(before.postcode.as_ref()));
    let town = RwSignal::new(shown(before.town.as_ref()));
    let birth_date = RwSignal::new(shown(before.birth_date.as_ref()));
    let (diet_was, other_was) = diet_choice(before.diet.as_deref().unwrap_or_default());
    let diet = RwSignal::new(diet_was);
    let other_diet = RwSignal::new(other_was);
    let account = RwSignal::new(
        person
            .as_ref()
            .and_then(|p| p.account.clone())
            .unwrap_or_default(),
    );
    let role = RwSignal::new(String::new());
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
        let mut people = core::incident_people(core);
        let known =
            core.with_value(|c| c.people.iter().map(ToString::to_string).collect::<Vec<_>>());
        for p in known {
            if !people.contains(&p) {
                people.push(p);
            }
        }
        people
    });
    let id = helper.as_ref().map(|h| h.id.clone());
    let title = match (&person, &helper) {
        (Some(p), Some(_)) => text::helper_edit_title(&shown_name(core, p)),
        (Some(p), None) => text::helper_complete_title(&shown_name(core, p)),
        (None, _) if readable => text::HELPER_ADD_TITLE.to_owned(),
        (None, _) => text::ROLE_ADD_TITLE.to_owned(),
    };
    let editing_one = person.is_some();
    // Choosing an account fills in the name, unless one was typed.
    let account_chosen = move |value: String| {
        let value = value.trim().to_owned();
        if readable
            && name.get_untracked().trim().is_empty()
            && value.parse::<ordo_core::UserId>().is_ok()
        {
            let shown = crate::profile::display_text(core, &value);
            if shown != value {
                name.set(shown);
            }
        }
        account.set(value);
    };
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let chosen_role = role.get_untracked();
        let account_value = account.get_untracked().trim().to_owned();
        // [impl->dsn~guest-access~1]
        // A role without an account: a guest, with access until the end of duty (ADR-0077).
        let guest = !chosen_role.is_empty() && account_value.is_empty();
        if !chosen_role.is_empty() && !guest && account_value.parse::<ordo_core::UserId>().is_err()
        {
            error.set(Some(text::HELPER_ACCOUNT));
            crate::views::focus("helper-account");
            return;
        }
        if guest && (!readable || chosen_role == core::COMMANDER) {
            error.set(Some(if readable {
                text::GUEST_NO_COMMAND
            } else {
                text::HELPER_ACCOUNT
            }));
            crate::views::focus(if readable {
                "helper-role"
            } else {
                "helper-account"
            });
            return;
        }
        let duty_end = core::local_time_ms(end.get_untracked().trim())
            .filter(|_| !end.get_untracked().trim().is_empty());
        if guest && duty_end.is_none_or(|e| e <= core::now_ms()) {
            error.set(Some(text::GUEST_NEEDS_DUTY_END));
            crate::views::focus("helper-duty-end");
            return;
        }
        let mut saved = None;
        if readable {
            match save_helper(&HelperDraft {
                core,
                revision,
                id: id.clone(),
                before: &before,
                name,
                organisation,
                phone,
                street,
                postcode,
                town,
                birth_date,
                diet,
                other_diet,
                account,
                start,
                end,
                qualifications,
            }) {
                Ok(id) => saved = Some(id),
                Err((message, field)) => {
                    error.set(Some(message));
                    crate::views::focus(field);
                    return;
                }
            }
        }
        // A guest of an earlier save keeps their access to the end of duty.
        if let (Some(a), Some(until)) = (before.account.as_deref(), duty_end)
            && let Ok(user) = a.parse::<ordo_core::UserId>()
            && user.is_guest()
            && before.duty_end != Some(until)
        {
            core::set_guest_access(core, user, until);
        }
        if guest {
            if let (Some(helper), Some(until)) = (saved, duty_end) {
                PENDING_ROLES.with_borrow_mut(|p| p.insert(helper.clone(), chosen_role.clone()));
                core::create_guest(core, None, until, &helper, &name.get_untracked());
            }
        } else if !chosen_role.is_empty() && manage {
            if core::change_role(core, revision, &account_value, &chosen_role, true).is_err() {
                error.set(Some(text::HELPER_NOT_SAVED));
                return;
            }
        } else if !readable {
            error.set(Some(text::ROLE_MISSING));
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
                street,
                postcode,
                town,
                birth_date,
                diet,
                other_diet,
                account,
                role,
                start,
                end,
            ] {
                s.set(String::new());
            }
            qualifications.set(BTreeSet::new());
            crate::views::focus("helper-account");
        }
    };
    // [impl->dsn~registration-card~2]
    // A registration card photographed fills the form of a new helper (HELP-5).
    let card_status = RwSignal::new(None::<String>);
    let read_card = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        let Some(file) = input
            .as_ref()
            .and_then(|i| i.files())
            .and_then(|f| f.get(0))
        else {
            return;
        };
        if let Some(i) = &input {
            i.set_value(""); // the same card can be photographed again
        }
        card_status.set(Some(text::CARD_READING.to_owned()));
        leptos::task::spawn_local(async move {
            let texts = crate::images::read_qr(&file).await.unwrap_or_default();
            let Some(card) = texts.iter().find_map(|t| CARDS.read(t)) else {
                card_status.set(Some(text::CARD_NOT_RECOGNISED.to_owned()));
                return;
            };
            let label = CARDS
                .card
                .iter()
                .find(|c| c.id == card.card)
                .map_or("", |c| c.label.as_str());
            // An account's name stays; the card adds what it has.
            if name.get_untracked().trim().is_empty() || account.get_untracked().trim().is_empty() {
                name.set(card.name.clone());
            }
            for (signal, value) in [
                (birth_date, card.birth_date),
                (street, card.street),
                (postcode, card.postcode),
                (town, card.town),
                (phone, card.phone),
                (organisation, card.organisation),
                (start, card.duty_start),
                (end, card.duty_end),
            ] {
                signal.set(value.unwrap_or_default());
            }
            let (choice, other) = diet_choice(card.diet.as_deref().unwrap_or_default());
            diet.set(choice);
            other_diet.set(other);
            error.set(None);
            card_status.set(Some(text::card_read(label, &card.name)));
        });
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
    let personal = readable.then(|| view! {
        {(!editing_one && !CARDS.card.is_empty()).then(|| view! {
            <div class="row helper-card">
                <label class="button" for="helper-card-photo">{text::card_photo(match CARDS.card.as_slice() {
                    [one] => one.label.as_str(),
                    _ => text::CARD_GENERIC,
                })}</label>
                <input id="helper-card-photo" class="visually-hidden" type="file" accept="image/*" capture="environment" on:change=read_card />
                <p role="status" class="hint">{move || card_status.get()}</p>
            </div>
        })}
        <div class="form--inline">
            {text_field("helper-name", text::HELPER_NAME, name, "off")}
            {text_field("helper-organisation", text::UNIT_ORGANISATION, organisation, "off")}
            <div class="field">
                <label for="helper-phone">{text::HELPER_PHONE}</label>
                <input id="helper-phone" type="tel" autocomplete="off" aria-describedby="helper-phone-hint"
                    maxlength=ordo_helpers::TEXT_MAX.to_string() bind:value=phone />
                <p id="helper-phone-hint" class="hint">{text::HELPER_PHONE_HINT}</p>
            </div>
            <div class="field">
                <label for="helper-birth-date">{text::HELPER_BIRTH_DATE_LABEL}</label>
                <input id="helper-birth-date" type="date" bind:value=birth_date />
            </div>
        </div>
        <div class="form--inline">
            {text_field("helper-street", text::HELPER_STREET, street, "off")}
            <div class="field field--small">
                <label for="helper-postcode">{text::HELPER_POSTCODE}</label>
                <input id="helper-postcode" type="text" inputmode="numeric" autocomplete="off"
                    maxlength=SETTINGS.postcode.chars().count().to_string() bind:value=postcode />
            </div>
            {text_field("helper-town", text::HELPER_TOWN, town, "off")}
        </div>
        <div class="form--inline">
            <div class="field">
                <label for="helper-diet">{text::HELPER_DIET}</label>
                <select id="helper-diet" prop:value=move || diet.get() on:change=move |ev| diet.set(event_target_value(&ev))>
                    <option value="" selected=move || diet.get().is_empty()>{text::DIET_NONE}</option>
                    {SETTINGS.diets.iter().map(|d| {
                        let value = d.clone();
                        view! { <option value=d.clone() selected=move || diet.get() == value>{d.clone()}</option> }
                    }).collect_view()}
                    <option value=DIET_OTHER selected=move || diet.get() == DIET_OTHER>{text::DIET_OTHER}</option>
                </select>
            </div>
            <Show when=move || diet.get() == DIET_OTHER>
                {text_field("helper-diet-other", text::DIET_OTHER_LABEL, other_diet, "off")}
            </Show>
        </div>
        <CheckChips legend=text::CREW_QUALIFICATIONS options=options.clone() value=qualifications />
        <div class="form--inline">
            <div class="field">
                <label for="helper-duty-start">{text::HELPER_DUTY_START}</label>
                <input id="helper-duty-start" type="datetime-local" bind:value=start />
            </div>
            <div class="field">
                <label for="helper-duty-end">{text::HELPER_DUTY_END}</label>
                <input id="helper-duty-end" type="datetime-local" bind:value=end />
            </div>
        </div>
    });
    view! {
        <form class="form helper-form" novalidate on:submit=submit aria-labelledby="helper-form-title">
            <h3 id="helper-form-title">{title}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="helper-account">{text::HELPER_ACCOUNT_LABEL}</label>
                    <input id="helper-account" type="text" autocomplete="off" spellcheck="false" list="helper-account-people"
                        aria-describedby="helper-account-hint"
                        prop:value=move || account.get()
                        on:input=move |ev| account.set(event_target_value(&ev))
                        on:change=move |ev| account_chosen(event_target_value(&ev)) />
                    <Suggestions id="helper-account-people" values=people core />
                    <p id="helper-account-hint" class="hint">{text::HELPER_ACCOUNT_HINT}</p>
                </div>
                {manage.then(|| view! {
                    <div class="field">
                        <label for="helper-role">{text::HELPER_ROLE}</label>
                        <select id="helper-role" prop:value=move || role.get() on:change=move |ev| role.set(event_target_value(&ev))>
                            <option value="">{text::ROLE_NONE}</option>
                            // A guest never leads (ADR-0077): without an account, no Einsatzleitung.
                            {move || text::ROLES
                                .iter()
                                .filter(|(id, _)| *id != core::COMMANDER || !account.get().trim().is_empty())
                                .map(|(id, label)| view! { <option value=*id>{*label}</option> })
                                .collect_view()}
                        </select>
                    </div>
                })}
            </div>
            {personal}
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">
                    {match (editing_one, readable) {
                        (true, true) => text::SAVE,
                        (false, true) => text::HELPER_ADD,
                        (_, false) => text::GRANT,
                    }}
                </button>
                {editing_one.then(|| view! {
                    <button type="button" class="button" on:click=move |_| editing.set(None)>{text::CANCEL}</button>
                })}
            </div>
        </form>
    }
}

/// The form's fields, to save them as a registration or a change.
struct HelperDraft<'a> {
    core: Handle,
    revision: RwSignal<u64>,
    id: Option<String>,
    before: &'a Fields,
    name: RwSignal<String>,
    organisation: RwSignal<String>,
    phone: RwSignal<String>,
    street: RwSignal<String>,
    postcode: RwSignal<String>,
    town: RwSignal<String>,
    birth_date: RwSignal<String>,
    diet: RwSignal<String>,
    other_diet: RwSignal<String>,
    account: RwSignal<String>,
    start: RwSignal<String>,
    end: RwSignal<String>,
    qualifications: RwSignal<BTreeSet<String>>,
}

// [impl->dsn~helper-register~3]
/// Checks and saves the form's personal data: the phone number in international form, the
/// postcode by the profile's pattern. Returns the helper's ID; on a problem, the message and
/// the field to focus.
fn save_helper(d: &HelperDraft<'_>) -> Result<String, (&'static str, &'static str)> {
    let ms = |s: RwSignal<String>| {
        let typed = s.get_untracked();
        if typed.trim().is_empty() {
            Ok(None)
        } else {
            core::local_time_ms(typed.trim()).map(Some).ok_or(())
        }
    };
    let (Ok(duty_start), Ok(duty_end)) = (ms(d.start), ms(d.end)) else {
        return Err((text::HELPER_TIME, "helper-duty-start"));
    };
    let phone_typed = d.phone.get_untracked();
    let phone = if phone_typed.trim().is_empty() {
        String::new()
    } else {
        match SETTINGS.phone(&phone_typed) {
            Some(p) => p,
            None => return Err((text::HELPER_PHONE_INVALID, "helper-phone")),
        }
    };
    let postcode = d.postcode.get_untracked().trim().to_owned();
    if !postcode.is_empty() && !SETTINGS.postcode_ok(&postcode) {
        return Err((text::HELPER_POSTCODE_INVALID, "helper-postcode"));
    }
    let diet = match d.diet.get_untracked().as_str() {
        DIET_OTHER => d.other_diet.get_untracked(),
        chosen => chosen.to_owned(),
    };
    let quals: Vec<String> = CATALOGUE
        .qualifications
        .iter()
        .filter(|q| d.qualifications.with_untracked(|s| s.contains(&q.id)))
        .map(|q| q.id.clone())
        .collect();
    let registration = d.id.is_none();
    let before = d.before;
    // A person chosen by account is named by it until a name is typed (ORG-7).
    let typed_name = d.name.get_untracked();
    let account = d.account.get_untracked().trim().to_owned();
    let name = if typed_name.trim().is_empty() && account.parse::<ordo_core::UserId>().is_ok() {
        crate::profile::display_text(d.core, &account)
    } else {
        typed_name
    };
    let fields = if registration {
        let given = |t: String| Some(t.trim().to_owned()).filter(|t| !t.is_empty());
        Fields {
            name: given(name.clone()),
            organisation: given(d.organisation.get_untracked()),
            phone: given(phone),
            address: None,
            street: given(d.street.get_untracked()),
            postcode: given(postcode),
            town: given(d.town.get_untracked()),
            birth_date: given(d.birth_date.get_untracked()),
            diet: given(diet),
            account: given(d.account.get_untracked()),
            qualifications: (!quals.is_empty()).then_some(quals),
            duty_start,
            duty_end,
        }
    } else {
        let was_quals = before.qualifications.clone().unwrap_or_default();
        // A time cleared is 0 (see `Fields`).
        let time_changed =
            |now: Option<u64>, was: Option<u64>| (now != was).then_some(now.unwrap_or(0));
        let street = d.street.get_untracked();
        Fields {
            name: changed(name.clone(), before.name.as_ref()),
            organisation: changed(d.organisation.get_untracked(), before.organisation.as_ref()),
            phone: changed(phone, before.phone.as_ref()),
            // An address of an older version moves to the street once the form is saved.
            address: (before.address.is_some() && before.street.is_none()).then(String::new),
            street: if before.street.is_none() && before.address.is_some() {
                Some(street.trim().to_owned())
            } else {
                changed(street, before.street.as_ref())
            },
            postcode: changed(postcode, before.postcode.as_ref()),
            town: changed(d.town.get_untracked(), before.town.as_ref()),
            birth_date: changed(d.birth_date.get_untracked(), before.birth_date.as_ref()),
            diet: changed(diet, before.diet.as_ref()),
            account: changed(d.account.get_untracked(), before.account.as_ref()),
            qualifications: (quals != was_quals).then_some(quals),
            duty_start: time_changed(duty_start, before.duty_start),
            duty_end: time_changed(duty_end, before.duty_end),
        }
    };
    let helper = match &d.id {
        Some(id) => id.clone(),
        None => core::new_ulid()
            .map(|u| u.to_string())
            .ok_or((text::HELPER_NOT_SAVED, "helper-name"))?,
    };
    let change = HelperChange { helper, fields };
    if let Err(e) = change.check(registration) {
        return Err(match e {
            FieldError::NameMissing => (text::HELPER_NAME_MISSING, "helper-name"),
            FieldError::TooLong => (text::HELPER_TOO_LONG, "helper-name"),
            FieldError::NotADate => (text::HELPER_BIRTH_DATE, "helper-birth-date"),
            FieldError::NotAnAccount => (text::HELPER_ACCOUNT, "helper-account"),
        });
    }
    core::helper_event(d.core, d.revision, registration, &change, name.trim())
        .map(|()| change.helper.clone())
        .map_err(|_| (text::HELPER_NOT_SAVED, "helper-name"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::{DeviceId, Hlc, Ulid};

    fn helper(id: &str, diet: Option<&str>, duty_end: Option<u64>) -> Helper {
        let node = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        Helper {
            id: id.into(),
            fields: Fields {
                name: Some(id.into()),
                diet: diet.map(Into::into),
                duty_end,
                ..Fields::default()
            },
            registered: (
                Hlc {
                    wall_ms: 1,
                    counter: 0,
                    node,
                },
                "ben@ordo.example.org".parse().expect("valid user"),
            ),
        }
    }

    // [utest->dsn~helper-register~3]
    #[test]
    fn diets_are_counted_for_those_still_on_duty() {
        let helpers = [
            helper("a", Some("vegetarisch"), None),
            helper("b", Some("Vegetarisch "), Some(9_000)),
            helper("c", Some("vegan"), None),
            helper("d", None, None),
            helper("e", Some("halal"), Some(1_000)), // duty ended
        ];
        assert_eq!(
            diet_counts(&helpers, 5_000),
            [
                ("vegetarisch".to_owned(), 2),
                ("vegan".to_owned(), 1),
                (text::DIET_NONE.to_owned(), 1)
            ]
        );
        assert!(
            diet_counts(&[helper("d", None, None)], 5_000).is_empty(),
            "nothing to show without diets"
        );
    }
}
