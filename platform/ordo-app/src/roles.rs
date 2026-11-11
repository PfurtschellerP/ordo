//! Organisation workspace: organisation roles (ORG-9). Administrators grant a person the
//! role of administrator or planner, optionally until a date, and withdraw it. Roles from
//! the server's configuration are shown, not changed.

use leptos::prelude::*;
use ordo_core::UserId;
use ordo_sync::message::{ORGANISATION_ROLES, OrganisationRole, OrganisationRoleChange};

use crate::core::{self, Handle};
use crate::text;
use crate::ui::Suggestions;

// [impl->dsn~organisation-roles~1]
/// The roles of the organisation, by person, and the form to grant one.
#[component]
pub fn RolesView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    // Ask after every new connection (an answer may be lost when it is cut).
    Effect::new(move |_| {
        if ready.get() && welcomes.get() > 0 {
            core::get_organisation_roles(core);
            core::list_people(core);
        }
    });
    let roles = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.organisation_roles.clone())
    });
    let me = core.with_value(|c| c.replica.user().clone());
    view! {
        <section aria-labelledby="org-roles-title" class="card">
            <h2 id="org-roles-title" tabindex="-1">{text::ORG_ROLES_TITLE}</h2>
            <p>{text::ORG_ROLES_INTRO}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::ORG_ROLES_OFFLINE}</p> }.into_any();
                }
                match roles.get() {
                    None => view! { <p>{text::ORG_ROLES_LOADING}</p> }.into_any(),
                    Some(list) if list.is_empty() => view! { <p>{text::ORG_ROLES_EMPTY}</p> }.into_any(),
                    Some(list) => view! { <RoleList core list me=me.clone() /> }.into_any(),
                }
            }}
            <GrantForm core revision />
        </section>
    }
}

/// The people with their roles.
#[component]
fn RoleList(core: Handle, list: Vec<OrganisationRole>, me: UserId) -> impl IntoView {
    let mut people: Vec<(UserId, Vec<OrganisationRole>)> = Vec::new();
    for role in list {
        match people.last_mut() {
            Some((user, roles)) if *user == role.user => roles.push(role),
            _ => people.push((role.user.clone(), vec![role])),
        }
    }
    let now = core::now_ms();
    view! {
        <ul class="roster role-list" data-keys="list">
            {people
                .into_iter()
                .map(|(user, roles)| {
                    let name = {
                        let user = user.clone();
                        crate::profile::naming(move || crate::profile::display_name(core, &user))
                    };
                    let person = user.to_string();
                    view! {
                        <li class="roster__entry" data-user=person.clone()>
                            <span class="roster__person">{name}</span>
                            <span class="roster__id">{person.clone()}</span>
                            <ul class="roster__roles">
                                {roles
                                    .into_iter()
                                    .map(|r| {
                                        let own_admin = r.user == me && r.role == "admin";
                                        let lapsed = r.valid_until.is_some_and(|u| u <= now);
                                        let mut facts = Vec::new();
                                        if r.fixed {
                                            facts.push(text::ORG_ROLE_FIXED.to_owned());
                                        } else if own_admin {
                                            facts.push(text::ORG_ROLE_OWN_ADMIN.to_owned());
                                        }
                                        if let Some(until) = r.valid_until {
                                            facts.push(text::org_role_until(&core::date(until.saturating_sub(1))));
                                        }
                                        if lapsed {
                                            facts.push(text::ORG_ROLE_LAPSED.to_owned());
                                        }
                                        let label = text::org_role_withdraw_label(&r.role, &person);
                                        let change = OrganisationRoleChange {
                                            user: r.user.clone(),
                                            role: r.role.clone(),
                                            grant: false,
                                            valid_until: None,
                                        };
                                        view! {
                                            <li class="roster__role" data-role=r.role.clone()>
                                                <span>
                                                    <strong>{text::org_role(&r.role).to_owned()}</strong>
                                                    {(!facts.is_empty()).then(|| format!(" · {}", facts.join(" · ")))}
                                                </span>
                                                {(!r.fixed && !own_admin).then(|| view! {
                                                    <button type="button" class="button button--small" aria-label=label
                                                        on:click=move |_| core::set_organisation_role(core, change.clone())>
                                                        {text::ORG_ROLE_WITHDRAW}
                                                    </button>
                                                })}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

/// Grants a role to a person, optionally until the end of a day.
#[component]
fn GrantForm(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let person = RwSignal::new(String::new());
    let role = RwSignal::new(ORGANISATION_ROLES[1].to_owned());
    let until = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let people = Signal::derive(move || {
        revision.track();
        core.with_value(|c| c.people.iter().map(ToString::to_string).collect::<Vec<_>>())
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(user) = person.get_untracked().trim().parse::<UserId>() else {
            error.set(Some(text::ORG_ROLE_INVALID));
            crate::views::focus("org-role-person");
            return;
        };
        let date = until.get_untracked();
        let valid_until = if date.trim().is_empty() {
            None
        } else {
            match core::end_of_day_ms(&date) {
                Some(ms) if ms > core::now_ms() => Some(ms),
                _ => {
                    error.set(Some(text::ORG_ROLE_PAST));
                    crate::views::focus("org-role-until");
                    return;
                }
            }
        };
        error.set(None);
        core::set_organisation_role(
            core,
            OrganisationRoleChange {
                user,
                role: role.get_untracked(),
                grant: true,
                valid_until,
            },
        );
        person.set(String::new());
        until.set(String::new());
    };
    view! {
        <form class="form" novalidate on:submit=submit aria-labelledby="org-role-grant-title">
            <h3 id="org-role-grant-title">{text::ORG_ROLE_GRANT_TITLE}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="org-role-person">{text::ORG_ROLE_PERSON}</label>
                    <input id="org-role-person" type="text" autocomplete="off" list="org-role-people" bind:value=person />
                    <Suggestions id="org-role-people".to_owned() values=people core />
                </div>
                <div class="field">
                    <label for="org-role-role">{text::ORG_ROLE_ROLE}</label>
                    <select id="org-role-role" bind:value=role>
                        {ORGANISATION_ROLES
                            .into_iter()
                            .map(|r| view! { <option value=r>{text::org_role(r).to_owned()}</option> })
                            .collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="org-role-until">{text::ORG_ROLE_UNTIL}</label>
                    <input id="org-role-until" type="date" aria-describedby="org-role-until-hint" bind:value=until />
                </div>
            </div>
            <p id="org-role-until-hint" class="hint">{text::ORG_ROLE_UNTIL_HINT}</p>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">{text::ORG_ROLE_GRANT}</button>
            </div>
        </form>
    }
}
