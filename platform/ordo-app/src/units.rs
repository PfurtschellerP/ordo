//! Units (UNIT-1…6): the view *Einsatzmittel* of the incident command, and the profile's
//! status codes, services and qualifications (checked at build time, see `build.rs`).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_units::{
    CrewMember, StatusCatalogue, Strength, Unit, UnitAdded, UnitCatalogue, UnitRegister,
};

use crate::core::{self, Handle};
use crate::text;
use crate::ui::{Action, Actions, CheckChips, Disclosure, Suggestions};

/// Status codes from the profile (`units/status-codes.toml`).
pub static STATUS: LazyLock<StatusCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/status-codes.json")))
        .expect("status codes were checked at build time")
});

/// Services and qualifications from the profile (`units/catalogue.toml`).
pub static CATALOGUE: LazyLock<UnitCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json")))
        .expect("unit catalogue was checked at build time")
});

/// Crew strength in tactical notation: 0/1/8/9, the total underlined.
#[component]
fn StrengthText(strength: Strength) -> impl IntoView {
    let s = strength;
    view! {
        <span class="strength" aria-label=text::strength_label(s.leaders, s.sub_leaders, s.crew, s.total())>
            {format!("{}/{}/{}/", s.leaders, s.sub_leaders, s.crew)}
            <span class="strength__total">{s.total()}</span>
        </span>
    }
}

/// Type, organisation and service of a unit as one line.
fn details(u: &Unit) -> String {
    let service = u
        .service
        .as_deref()
        .map(|s| UnitCatalogue::label(&CATALOGUE.services, s).to_owned());
    [
        Some(u.unit_type.clone()),
        Some(u.organisation.clone()),
        service,
    ]
    .into_iter()
    .flatten()
    .filter(|t| !t.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

// [impl->dsn~unit-register~2]
/// Units and persons on scene, in total, per organisation and per service (UNIT-4).
#[component]
fn StrengthOnScene(register: Memo<UnitRegister>) -> impl IntoView {
    let summary = Memo::new(move |_| register.with(UnitRegister::strength_on_scene));
    view! {
        <section class="strength-on-scene" aria-labelledby="strength-title">
            <h3 id="strength-title">{text::STRENGTH_TITLE}</h3>
            {move || {
                let s = summary.get();
                let rows = |map: std::collections::BTreeMap<String, (u32, Strength)>, service: bool| {
                    map.into_iter()
                        .map(|(key, (units, strength))| {
                            let name = if service {
                                if key.is_empty() { text::NO_SERVICE.to_owned() } else { UnitCatalogue::label(&CATALOGUE.services, &key).to_owned() }
                            } else if key.is_empty() { text::NO_ORGANISATION.to_owned() } else { key };
                            view! { <li>{name} ": " {text::units_count(units)} " · " <StrengthText strength /></li> }
                        })
                        .collect_view()
                };
                view! {
                    <p class="strength-on-scene__total">
                        <strong>{text::units_count(s.units)}</strong> " · " {text::STRENGTH} " " <StrengthText strength=s.total />
                        {(s.unreported > 0).then(|| format!(" · {}", text::unreported(s.unreported)))}
                    </p>
                    <div class="strength-on-scene__groups">
                        <div>
                            <h4>{text::BY_ORGANISATION}</h4>
                            <ul>{rows(s.by_organisation, false)}</ul>
                        </div>
                        <div>
                            <h4>{text::BY_SERVICE}</h4>
                            <ul>{rows(s.by_service, true)}</ul>
                        </div>
                    </div>
                }
            }}
        </section>
    }
}

/// What someone types into a unit's crew panel, and whether it is open: kept by the view
/// under its owner, so the list can update while they type.
#[derive(Clone, Copy)]
struct CrewDraft {
    open: RwSignal<bool>,
    person: RwSignal<String>,
    qualifications: RwSignal<BTreeSet<String>>,
}

/// The crew of a unit (UNIT-5): members with qualifications; adding and removing writes the
/// whole list.
#[component]
fn CrewEditor(
    core: Handle,
    revision: RwSignal<u64>,
    unit: Unit,
    editable: bool,
    draft: CrewDraft,
) -> impl IntoView {
    let name = unit.unit.clone();
    let members = unit.crew.clone();
    let CrewDraft {
        person,
        qualifications,
        ..
    } = draft;
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    let field = format!("crew-{}", core::slug(&name));
    let add = {
        let (name, members) = (name.clone(), members.clone());
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let who = person.get_untracked().trim().to_owned();
            if who.is_empty() {
                return;
            }
            let mut list = members.clone();
            list.push(CrewMember {
                name: who,
                qualifications: CATALOGUE
                    .qualifications
                    .iter()
                    .filter(|q| qualifications.with_untracked(|s| s.contains(&q.id)))
                    .map(|q| q.id.clone())
                    .collect(),
            });
            if core::set_crew(core, revision, &name, list).is_ok() {
                person.set(String::new());
                qualifications.set(BTreeSet::new());
            }
        }
    };
    let options: Vec<(String, String)> = CATALOGUE
        .qualifications
        .iter()
        .map(|q| (q.id.clone(), q.label.clone()))
        .collect();
    view! {
        {if members.is_empty() {
            view! { <p class="hint">{text::CREW_NONE}</p> }.into_any()
        } else {
            view! {
                <ul class="crew-list">
                    {members
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            let quals = m
                                .qualifications
                                .iter()
                                .map(|q| UnitCatalogue::label(&CATALOGUE.qualifications, q).to_owned())
                                .collect::<Vec<_>>()
                                .join(", ");
                            let remove = editable.then(|| {
                                let (name, members) = (name.clone(), members.clone());
                                vec![Action {
                                    text: text::CREW_REMOVE,
                                    label: text::crew_remove_label(&m.name),
                                    quiet: true,
                                    run: Callback::new(move |()| {
                                        let mut list = members.clone();
                                        list.remove(i);
                                        let _ = core::set_crew(core, revision, &name, list);
                                    }),
                                }]
                            });
                            view! {
                                <li>
                                    <strong>{m.name.clone()}</strong>
                                    {(!quals.is_empty()).then(|| format!(" · {quals}"))}
                                    {remove.map(|actions| view! { <Actions actions /> })}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        }}
        {editable.then(|| view! {
            <form class="form crew-form" novalidate on:submit=add>
                <div class="field">
                    <label for=field.clone()>{text::CREW_MEMBER}</label>
                    <input id=field.clone() type="text" autocomplete="off" list=format!("{field}-people") bind:value=person />
                    <Suggestions id=format!("{field}-people") values=people core />
                </div>
                <CheckChips legend=text::CREW_QUALIFICATIONS options=options.clone() value=qualifications />
                <div class="row">
                    <button type="submit" class="button button--small">{text::CREW_ADD}</button>
                </div>
            </form>
        })}
    }
}

/// One unit: what it is, its strength, status since when, crew, and releasing it.
#[component]
fn UnitCard(
    core: Handle,
    revision: RwSignal<u64>,
    unit: Unit,
    editable: bool,
    draft: CrewDraft,
    /// The unit's open tasks (TASK-3).
    tasks: Vec<String>,
    /// The open positions (ID, title) and the unit's position (CMD-2).
    positions: Vec<(String, String)>,
    position: Option<String>,
) -> impl IntoView {
    let u = unit;
    let name = u.unit.clone();
    let current = u
        .status
        .as_ref()
        .map(|s| s.status.clone())
        .unwrap_or_default();
    let shown = u.status.as_ref().map_or_else(
        || text::NO_STATUS.to_owned(),
        |s| {
            text::status_since(
                &text::status_label(&s.status),
                &core::clock(s.since.wall_ms),
            )
        },
    );
    let crew_open = draft.open;
    let release = editable.then(|| {
        let name = name.clone();
        vec![Action {
            text: text::RELEASE,
            label: text::release_label(&name),
            quiet: true,
            run: Callback::new(move |()| {
                let _ = core::release_unit(core, revision, &name);
            }),
        }]
    });
    let crew_label = text::crew_label(&name, u.crew.len());
    view! {
        <li class="unit-card" data-unit=name.clone()>
            <h3 class="unit-card__name">{name.clone()}</h3>
            <p class="unit-card__details">
                {details(&u)}
                {u.strength.map(|strength| view! { " · " {text::STRENGTH} " " <StrengthText strength /> })}
            </p>
            <p class="unit-card__status">{text::COL_STATUS} ": " <strong>{shown}</strong></p>
            // [impl->dsn~tasks~2]
            {(!tasks.is_empty()).then(|| view! {
                <p class="unit-card__tasks">{text::TASKS_OF_UNIT} ": " {tasks.join(" · ")}</p>
            })}
            // [impl->dsn~command-structure~1]
            // Under which position: one choice, two taps (CMD-2).
            {(editable && !positions.is_empty()).then(|| {
                let name = name.clone();
                let current = position.clone().unwrap_or_default();
                view! {
                    <select class="unit-card__position" aria-label=text::position_of(&name) on:change=move |ev| {
                        let id = event_target_value(&ev);
                        let a = ordo_command::Assigned {
                            unit: Some(name.clone()),
                            uhs: None,
                            position: (!id.is_empty()).then_some(id),
                        };
                        let _ = core::command_event(core, revision, ordo_command::ASSIGNED, &a.encode().unwrap_or_default());
                    }>
                        <option value="" selected=current.is_empty()>{text::NO_POSITION}</option>
                        {positions
                            .iter()
                            .map(|(id, title)| view! { <option value=id.clone() selected=*id == current>{title.clone()}</option> })
                            .collect_view()}
                    </select>
                }
            })}
            {(!editable).then(|| position.as_ref().and_then(|id| positions.iter().find(|(p, _)| p == id)).map(|(_, title)| view! {
                <p class="unit-card__position">{text::POSITION} ": " {title.clone()}</p>
            }))}
            {editable.then(|| {
                let name = name.clone();
                view! {
                    <select aria-label=text::set_status_label(&name) on:change=move |ev| {
                        let value = event_target_value(&ev);
                        if !value.is_empty() {
                            let _ = core::set_status(core, revision, &name, &value);
                        }
                    }>
                        <option value="" selected=current.is_empty()>{text::COL_SET_STATUS}</option>
                        {STATUS
                            .codes
                            .iter()
                            .map(|s| {
                                let code = s.code.clone();
                                view! { <option value=code.clone() selected=code == current>{text::status_label(&code)}</option> }
                            })
                            .collect_view()}
                    </select>
                }
            })}
            <Disclosure
                id=format!("crew-{}", core::slug(&name))
                open=crew_open
                class="unit-card__crew"
                button_class="button button--small button--quiet"
                panel_class="unit-card__crew-panel"
                label=Some(Signal::derive(move || crew_label.clone()))
                button=Box::new(|| view! { {text::CREW} }.into_any())
            >
                <CrewEditor core revision unit=u.clone() editable draft />
            </Disclosure>
            {release.map(|actions| view! { <Actions actions /> })}
        </li>
    }
}

// [impl->dsn~unit-register~2]
// [impl->dsn~views-by-role~1]
/// The incident's units (UNIT-1…6): strength on scene, every unit with status, crew and
/// release; released units apart; adding from the installation's vehicles or ad hoc.
/// Adding units and setting their status: incident command only (`edit_units`).
#[component]
pub fn ResourcesView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let can_edit = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
    Effect::new(move || {
        if can_edit.get() {
            core::list_vehicles(core);
        }
    });
    let register = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)))
    });
    let present = Memo::new(move |_| register.with(|r| r.present().cloned().collect::<Vec<_>>()));
    // Drafts of the crew panels, created under this view's owner (see `CrewDraft`).
    let owner = Owner::current().expect("a view has an owner");
    let drafts = StoredValue::new(std::collections::HashMap::<String, CrewDraft>::new());
    let draft = move |unit: &str| {
        let mut out = None;
        drafts.update_value(|m| {
            out = Some(*m.entry(unit.to_owned()).or_insert_with(|| {
                owner.with(|| CrewDraft {
                    open: RwSignal::new(false),
                    person: RwSignal::new(String::new()),
                    qualifications: RwSignal::new(BTreeSet::new()),
                })
            }));
        });
        out.expect("inserted")
    };
    let released = Memo::new(move |_| {
        register.with(|r| {
            r.units
                .values()
                .filter(|u| !u.is_present())
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    view! {
        <section aria-labelledby="units-title" class="card">
            <h2 id="units-title">{text::UNITS_TITLE}</h2>
            {move || {
                register
                    .get()
                    .lost_updates
                    .into_iter()
                    .map(|l| view! { <p class="notice notice--warning">{text::lost_update(&l.unit, &l.overwritten, &l.kept)}</p> })
                    .collect_view()
            }}
            <StrengthOnScene register />
            {move || {
                let units = present.get();
                let editable = can_edit.get();
                revision.track();
                let task_list = crate::tasks::task_list(core);
                let structure = crate::command::structure(core);
                let positions = crate::command::position_choices(&structure);
                if units.is_empty() {
                    return view! { <p>{text::UNITS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="unit-list" data-keys="list">
                        {units
                            .into_iter()
                            .map(|unit| {
                                let draft = draft(&unit.unit);
                                let tasks = task_list
                                    .of_unit(&unit.unit)
                                    .iter()
                                    .map(|t| t.given.text.clone())
                                    .collect::<Vec<_>>();
                                let position = structure.unit_positions.get(&unit.unit).cloned();
                                let positions = positions.clone();
                                view! { <UnitCard core revision unit editable draft tasks positions position /> }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <details class="released-list" hidden=move || released.with(Vec::is_empty)>
                <summary>{move || text::released_list(released.with(Vec::len))}</summary>
                <ul class="unit-list">
                    {move || {
                        let editable = can_edit.get();
                        released
                            .get()
                            .into_iter()
                            .map(|u| {
                                let since = u.presence.last().map(|e| core::clock(e.at.wall_ms)).unwrap_or_default();
                                let back = editable.then(|| {
                                    let again = UnitAdded {
                                        unit: u.unit.clone(),
                                        unit_type: u.unit_type.clone(),
                                        organisation: u.organisation.clone(),
                                        service: u.service.clone(),
                                        strength: u.strength,
                                    };
                                    vec![Action {
                                        text: text::BACK_IN,
                                        label: text::back_in_label(&u.unit),
                                        quiet: false,
                                        run: Callback::new(move |()| {
                                            let _ = core::add_unit(core, revision, again.clone());
                                        }),
                                    }]
                                });
                                view! {
                                    <li class="unit-card unit-card--released" data-unit=u.unit.clone()>
                                        <h3 class="unit-card__name">{u.unit.clone()}</h3>
                                        <p class="unit-card__details">{details(&u)}</p>
                                        <p>{text::released_at(&since)}</p>
                                        {back.map(|actions| view! { <Actions actions /> })}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </details>
            <Show when=move || can_edit.get()>
                <AddUnitForm core revision />
            </Show>
        </section>
        <crate::views::UhsResources core revision />
    }
}

/// Adding a unit (UNIT-1, UNIT-2): a call sign of the installation's vehicles fills in
/// the rest; anything can be entered ad hoc.
#[component]
fn AddUnitForm(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let call_sign = RwSignal::new(String::new());
    let unit_type = RwSignal::new(String::new());
    let organisation = RwSignal::new(String::new());
    let service = RwSignal::new(String::new());
    let (leaders, sub_leaders, crew) = (
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
    );
    let error = RwSignal::new(None::<&'static str>);
    let vehicles = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.vehicles.clone())
    });
    let call_signs = Signal::derive(move || {
        vehicles.with(|v| v.iter().map(|v| v.unit.clone()).collect::<Vec<_>>())
    });
    // Organisations and types offered as choices: those of the installation's vehicles and
    // of the units already in the incident; anything else can still be typed.
    let in_incident = Memo::new(move |_| {
        revision.track();
        let events = core.with_value(|c| c.events(ordo_units::COMPARTMENT));
        ordo_units::UnitRegister::from_events(&events)
            .units
            .into_values()
            .map(|u| (u.organisation, u.unit_type))
            .collect::<Vec<_>>()
    });
    let known = move |organisations: bool| {
        Signal::derive(move || {
            let mut set = BTreeSet::new();
            vehicles.with(|v| {
                set.extend(v.iter().map(|v| {
                    if organisations {
                        v.organisation.clone()
                    } else {
                        v.unit_type.clone()
                    }
                }));
            });
            in_incident.with(|units| {
                set.extend(
                    units
                        .iter()
                        .map(|(o, t)| if organisations { o.clone() } else { t.clone() }),
                );
            });
            set.into_iter()
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>()
        })
    };
    let organisations = known(true);
    let types = known(false);
    // A known call sign fills in type, organisation, service and default strength.
    let fill = move |value: String| {
        if let Some(v) =
            vehicles.with_untracked(|list| list.iter().find(|v| v.unit == value).cloned())
        {
            unit_type.set(v.unit_type);
            organisation.set(v.organisation);
            service.set(v.service.unwrap_or_default());
            let [a, b, c] = v.strength.unwrap_or_default();
            if v.strength.is_some() {
                leaders.set(a.to_string());
                sub_leaders.set(b.to_string());
                crew.set(c.to_string());
            }
        }
        call_sign.set(value);
    };
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (u, t) = (
            call_sign.get_untracked().trim().to_owned(),
            unit_type.get_untracked().trim().to_owned(),
        );
        if u.is_empty() || t.is_empty() {
            error.set(Some(text::UNIT_MISSING));
            return;
        }
        let number = |s: RwSignal<String>| s.get_untracked().trim().parse::<u32>().ok();
        let parts = [number(leaders), number(sub_leaders), number(crew)];
        let given = [leaders, sub_leaders, crew]
            .iter()
            .any(|s| !s.get_untracked().trim().is_empty());
        let strength = match parts {
            [Some(a), Some(b), Some(c)] => Some(Strength::from([a, b, c])),
            _ if !given => None,
            _ => {
                error.set(Some(text::STRENGTH_NUMBERS));
                return;
            }
        };
        let service = service.get_untracked();
        let unit = UnitAdded {
            unit: u,
            unit_type: t,
            organisation: organisation.get_untracked().trim().to_owned(),
            service: (!service.is_empty()).then_some(service),
            strength,
        };
        if core::add_unit(core, revision, unit).is_ok() {
            for s in [
                call_sign,
                unit_type,
                organisation,
                leaders,
                sub_leaders,
                crew,
            ] {
                s.set(String::new());
            }
            error.set(None);
        }
    };
    view! {
        <form on:submit=submit class="form add-unit" novalidate>
            <h3>{text::UNIT_SUBMIT}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="unit-call-sign">{text::UNIT_CALL_SIGN}</label>
                    <input id="unit-call-sign" type="text" required autocomplete="off" list="unit-call-signs"
                        prop:value=move || call_sign.get() on:input=move |ev| fill(event_target_value(&ev)) />
                    <Suggestions id="unit-call-signs" values=call_signs />
                </div>
                <div class="field">
                    <label for="unit-type">{text::UNIT_TYPE}</label>
                    <input id="unit-type" type="text" required autocomplete="off" list="unit-types" bind:value=unit_type />
                    <Suggestions id="unit-types" values=types />
                </div>
                <div class="field">
                    <label for="unit-organisation">{text::UNIT_ORGANISATION}</label>
                    <input id="unit-organisation" type="text" autocomplete="off" list="unit-organisations" bind:value=organisation />
                    <Suggestions id="unit-organisations" values=organisations />
                </div>
                <div class="field">
                    <label for="unit-service">{text::UNIT_SERVICE}</label>
                    <select id="unit-service" prop:value=move || service.get() on:change=move |ev| service.set(event_target_value(&ev))>
                        <option value="">{text::NO_SERVICE}</option>
                        {CATALOGUE.services.iter().map(|s| view! { <option value=s.id.clone()>{s.label.clone()}</option> }).collect_view()}
                    </select>
                </div>
            </div>
            <fieldset class="choices strength-input">
                <legend>{text::STRENGTH_LEGEND}</legend>
                {[(leaders, "unit-leaders", text::LEADERS), (sub_leaders, "unit-sub-leaders", text::SUB_LEADERS), (crew, "unit-crew", text::CREW_COUNT)]
                    .into_iter()
                    .map(|(value, id, label)| view! {
                        <div class="field field--small">
                            <label for=id>{label}</label>
                            <input id=id type="text" inputmode="numeric" autocomplete="off" bind:value=value />
                        </div>
                    })
                    .collect_view()}
            </fieldset>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button">{text::UNIT_SUBMIT}</button>
            </div>
        </form>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unit_is_described_by_type_organisation_and_service() {
        let unit = Unit {
            unit: "RTW 1".into(),
            unit_type: "RTW".into(),
            organisation: "DRK".into(),
            service: Some("rescue".into()),
            strength: None,
            status: None,
            crew: Vec::new(),
            presence: Vec::new(),
        };
        assert!(unit.is_present());
        assert_eq!(details(&unit), "RTW · DRK · Rettungsdienst");
    }
}
