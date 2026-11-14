//! Units (UNIT-1…6): the view *Einsatzmittel* of the incident command, and the profile's
//! status codes, services and qualifications (checked at build time, see `build.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_units::{
    Assignments, CrewMember, Post, Special, StatusCatalogue, Strength, Unit, UnitAdded,
    UnitCatalogue, UnitRegister,
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

// [impl->dsn~detailed-strength~1]
/// Crew strength in tactical notation, the total underlined: 0/1/8/9, or in the detailed
/// notation 0/0/1/8/9 with the special functions after it, "(davon 2 × Ärzte)" (UNIT-7).
#[component]
pub(crate) fn StrengthText(
    strength: Strength,
    #[prop(optional)] special: Special,
    #[prop(optional)] detailed: bool,
) -> impl IntoView {
    let parts = strength.parts(detailed);
    let (total, rest) = parts.split_last().map_or((0, &[][..]), |(t, r)| (*t, r));
    let leading: String = rest.iter().map(|n| format!("{n}/")).collect();
    let special = if detailed {
        special_text(&special)
    } else {
        None
    };
    view! {
        <span class="strength" aria-label=text::strength_label(&strength, detailed, special.as_deref())>
            {leading}
            <span class="strength__total">{total}</span>
            {special.clone().map(|s| format!(" ({s})"))}
        </span>
    }
}

/// "davon 2 × Ärzte, 6 × AGT", with the profile's labels; none without special functions.
pub(crate) fn special_text(special: &Special) -> Option<String> {
    let listed: Vec<String> = CATALOGUE
        .special_functions
        .iter()
        .filter_map(|f| {
            special
                .get(&f.id)
                .filter(|n| **n > 0)
                .map(|n| format!("{n} × {}", f.label))
        })
        .collect();
    (!listed.is_empty()).then(|| text::special_functions(&listed.join(", ")))
}

/// Whether the chosen incident shows strengths in the detailed notation (UNIT-7).
pub(crate) fn detailed(core: Handle) -> bool {
    core.with_value(|c| {
        c.incident_info()
            .is_some_and(|i| i.details.strength_detailed)
    })
}

/// What someone types as a strength: leaders of formations, leaders, sub-leaders, crew,
/// and the special functions by ID.
#[derive(Clone, Copy)]
pub(crate) struct StrengthDraft {
    pub parts: [RwSignal<String>; 4],
    pub special: RwSignal<BTreeMap<String, String>>,
}

impl StrengthDraft {
    /// A draft filled with `strength` and `special`.
    pub fn new(strength: Option<Strength>, special: &Special) -> Self {
        let parts = strength.map(|s| [s.unit_leaders, s.leaders, s.sub_leaders, s.crew]);
        Self {
            parts: [0, 1, 2, 3]
                .map(|i| RwSignal::new(parts.map(|p| p[i].to_string()).unwrap_or_default())),
            special: RwSignal::new(
                special
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
            ),
        }
    }

    /// Empties it.
    pub fn clear(&self) {
        for s in self.parts {
            s.set(String::new());
        }
        self.special.set(BTreeMap::new());
    }

    /// The strength (none if all its fields are empty; a missing leader of formations
    /// counts as 0) and the special functions, or the error to show.
    pub fn read(&self) -> Result<(Option<Strength>, Special), &'static str> {
        let texts = self.parts.map(|s| s.get_untracked().trim().to_owned());
        let number = |t: &str| t.parse::<u32>().ok().filter(|n| *n < 1000);
        let strength = if texts.iter().all(String::is_empty) {
            None
        } else {
            let unit_leaders = if texts[0].is_empty() {
                Some(0)
            } else {
                number(&texts[0])
            };
            match [
                unit_leaders,
                number(&texts[1]),
                number(&texts[2]),
                number(&texts[3]),
            ] {
                [
                    Some(unit_leaders),
                    Some(leaders),
                    Some(sub_leaders),
                    Some(crew),
                ] => Some(Strength {
                    unit_leaders,
                    leaders,
                    sub_leaders,
                    crew,
                }),
                _ => return Err(text::STRENGTH_NUMBERS),
            }
        };
        let mut special = Special::new();
        for (id, value) in self.special.get_untracked() {
            match value.trim() {
                "" => {}
                v => match number(v) {
                    Some(0) => {}
                    Some(n) => {
                        special.insert(id, n);
                    }
                    None => return Err(text::STRENGTH_NUMBERS),
                },
            }
        }
        Ok((strength, special))
    }
}

// [impl->dsn~detailed-strength~1]
/// The fields of a strength: leaders of formations and special functions only in the
/// detailed notation (UNIT-7).
#[component]
pub(crate) fn StrengthFields(
    draft: StrengthDraft,
    /// Prefix of the fields' IDs, e.g. `unit` for `unit-leaders`.
    prefix: String,
    /// What the strength is of, e.g. "Stärke" or "Stärke der UHS".
    title: &'static str,
    detailed: Signal<bool>,
) -> impl IntoView {
    // Labels follow the notation: the detailed one names Zugführer and Einsatzkräfte.
    let field = move |value: RwSignal<String>, id: String, part: usize| {
        view! {
            <div class="field field--small">
                <label for=id.clone()>{move || text::strength_part(part, detailed.get())}</label>
                <input id=id type="text" inputmode="numeric" autocomplete="off" bind:value=value />
            </div>
        }
    };
    let [unit_leaders, leaders, sub_leaders, crew] = draft.parts;
    let p = prefix.clone();
    view! {
        <fieldset class="choices strength-input">
            <legend>{move || text::strength_legend(title, detailed.get())}</legend>
            {move || detailed.get().then(|| field(unit_leaders, format!("{p}-unit-leaders"), 0))}
            {field(leaders, format!("{prefix}-leaders"), 1)}
            {field(sub_leaders, format!("{prefix}-sub-leaders"), 2)}
            {field(crew, format!("{prefix}-crew"), 3)}
        </fieldset>
        {move || (detailed.get() && !CATALOGUE.special_functions.is_empty()).then(|| {
            let prefix = prefix.clone();
            view! {
                <fieldset class="choices strength-input">
                    <legend>{text::SPECIAL_FUNCTIONS_LEGEND}</legend>
                    {CATALOGUE.special_functions.iter().map(|f| {
                        let id = format!("{prefix}-special-{}", f.id);
                        let key = f.id.clone();
                        let current = {
                            let key = key.clone();
                            move || draft.special.with(|m| m.get(&key).cloned().unwrap_or_default())
                        };
                        view! {
                            <div class="field field--small">
                                <label for=id.clone()>{f.label.clone()}</label>
                                <input id=id type="text" inputmode="numeric" autocomplete="off" prop:value=current
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        draft.special.update(|m| { m.insert(key.clone(), value); });
                                    } />
                            </div>
                        }
                    }).collect_view()}
                </fieldset>
            }
        })}
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

// [impl->dsn~unit-register~3]
// [impl->dsn~uhs-as-unit~1]
/// Units and persons on scene, in total, per organisation and per service (UNIT-4), every
/// UHS counted as a unit (UHS-12).
#[component]
fn StrengthOnScene(
    core: Handle,
    revision: RwSignal<u64>,
    register: Memo<UnitRegister>,
) -> impl IntoView {
    let summary = Memo::new(move |_| {
        revision.track();
        let events = core.with_value(|c| c.events(ordo_med_patients::UHS_COMPARTMENT));
        register.with(|r| on_scene(core, r, &events))
    });
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
                            view! { <li>{name} ": " {text::units_count(units)} " · " <StrengthText strength detailed=detailed(core) /></li> }
                        })
                        .collect_view()
                };
                // [impl->dsn~double-booking~1]
                // People on more than one post may be counted twice: marked, and named.
                let several = s.people.several();
                view! {
                    <p class="strength-on-scene__total">
                        <strong>{text::units_count(s.units)}</strong> " · " {text::STRENGTH} " " <StrengthText strength=s.total special=s.special.clone() detailed=detailed(core) />
                        {(!several.is_empty()).then(|| view! { <span class="strength-on-scene__mark" aria-hidden="true">"*"</span> })}
                        {(s.unreported > 0).then(|| format!(" · {}", text::unreported(s.unreported)))}
                    </p>
                    {(!several.is_empty()).then(|| view! {
                        <p class="hint strength-on-scene__several">
                            <span aria-hidden="true">"* "</span>
                            {text::double_booked(&several_labels(&several))}
                        </p>
                    })}
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

/// "Ben (RTW 1, UHS Süd)" for each person on several posts.
pub(crate) fn several_labels(several: &[(String, Vec<Post>)]) -> Vec<String> {
    several
        .iter()
        .map(|(person, posts)| {
            let at: Vec<&str> = posts.iter().map(|p| p.label.as_str()).collect();
            format!("{person} ({})", at.join(", "))
        })
        .collect()
}

// [impl->dsn~double-booking~1]
/// Where people are assigned in the chosen incident: the crews of the units present and the
/// UHS staff lists (UNIT-8).
pub(crate) fn assignments(core: Handle) -> Assignments {
    let events = core.with_value(|c| c.events(ordo_units::COMPARTMENT));
    on_scene(core, &UnitRegister::from_events(&events), &events).people
}

// [impl->dsn~crews-from-register~2]
/// The strength on scene with every UHS (UHS-12), where a helper's account on a UHS staff
/// list counts as that helper (HELP-2), so double bookings are found across both.
pub(crate) fn on_scene(
    core: Handle,
    units: &UnitRegister,
    events: &[ordo_sync::AppliedEvent],
) -> ordo_units::StrengthOnScene {
    let mut s = ordo_med_patients::strength_on_scene(units, events);
    for h in crate::helpers::register(core)
        .iter()
        .flat_map(|r| r.helpers.values())
    {
        if let Some(account) = &h.fields.account {
            s.people.link(account, &h.id);
        }
    }
    s
}

/// A person being assigned: as typed, the helper of the register if linked, and the
/// helper's account, which UHS staff lists use.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Who {
    pub name: String,
    pub helper: Option<String>,
    pub account: Option<String>,
}

impl Who {
    /// Whether the crew member `m` is this person: by helper where both are linked, else
    /// by name.
    fn is_member(&self, m: &CrewMember) -> bool {
        match (&self.helper, &m.helper) {
            (Some(a), Some(b)) => a == b,
            _ => ordo_units::same_person(&m.name, &self.name),
        }
    }

    /// Whether the UHS staff entry `s` is this person: by name or by the helper's account.
    fn is_staff(&self, s: &str) -> bool {
        ordo_units::same_person(s, &self.name)
            || self
                .account
                .as_deref()
                .is_some_and(|a| ordo_units::same_person(s, a))
    }

    /// Where else this person is assigned.
    pub fn elsewhere(&self, core: Handle, here: &Post) -> Vec<Post> {
        assignments(core).elsewhere(&self.name, self.helper.as_deref(), here)
    }
}

/// Takes `who` off the crews and UHS staff lists of `from`, to assign them elsewhere.
pub(crate) fn move_away(core: Handle, revision: RwSignal<u64>, who: &Who, from: &[Post]) {
    let (register, uhs) = core.with_value(|c| {
        let events = c.events(ordo_units::COMPARTMENT);
        (
            UnitRegister::from_events(&events),
            ordo_med_patients::uhs_details(&events),
        )
    });
    for post in from {
        if let Some(unit) = post.call_sign().and_then(|call| register.units.get(call)) {
            let crew = unit
                .crew
                .iter()
                .filter(|m| !who.is_member(m))
                .cloned()
                .collect();
            let _ = core::set_crew(core, revision, &unit.unit, crew);
        } else if let Some(id) = ordo_med_patients::uhs_of_post(post) {
            let staff = uhs
                .get(id)
                .map(|d| {
                    d.staff
                        .iter()
                        .filter(|s| !who.is_staff(s))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            let update = ordo_med_patients::UhsUpdated {
                id: id.to_owned(),
                staff: Some(staff),
                ..Default::default()
            };
            let _ = core::update_uhs(core, revision, &update);
        }
    }
}

// [impl->dsn~double-booking~1]
/// Said when the person being added is assigned elsewhere already (UNIT-8): where, and a way
/// to move them here instead of counting them twice.
#[component]
pub(crate) fn AssignedElsewhere(
    /// The person being added, as entered.
    person: RwSignal<String>,
    /// Where they are assigned; empty: nothing to say.
    elsewhere: RwSignal<Vec<Post>>,
    /// The label of the post they are added to.
    here: String,
    /// Moves them: takes them off `elsewhere` and adds them here.
    on_move: Callback<()>,
) -> impl IntoView {
    let posts = move || elsewhere.with(|e| e.iter().map(|p| p.label.clone()).collect::<Vec<_>>());
    view! {
        <Show when=move || elsewhere.with(|e| !e.is_empty())>
            <div class="notice notice--warning assigned-elsewhere" role="alert">
                <p>{move || text::assigned_elsewhere(person.get().trim(), &posts())}</p>
                <div class="row">
                    <button type="button" class="button button--small"
                        aria-label={
                            let here = here.clone();
                            move || text::move_here_label(person.get().trim(), &here)
                        }
                        on:click=move |_| on_move.run(())>
                        {text::MOVE_HERE}
                    </button>
                    <button type="button" class="button button--small button--quiet" on:click=move |_| elsewhere.set(Vec::new())>
                        {text::CANCEL}
                    </button>
                </div>
            </div>
        </Show>
    }
}

/// What someone types into a unit's crew panel, and whether it is open: kept by the view
/// under its owner, so the list can update while they type.
#[derive(Clone, Copy)]
struct CrewDraft {
    open: RwSignal<bool>,
    person: RwSignal<String>,
    qualifications: RwSignal<BTreeSet<String>>,
    /// Where the person being added is assigned already (UNIT-8).
    elsewhere: RwSignal<Vec<Post>>,
    /// The person being added is not in the helper register (HELP-3).
    unregistered: RwSignal<bool>,
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
        elsewhere,
        unregistered,
        ..
    } = draft;
    // [impl->dsn~crews-from-register~2]
    // Crews come from the helper register where this device reads it (HELP-3).
    let register = Memo::new(move |_| {
        revision.track();
        crate::helpers::register(core)
    });
    let people = Memo::new(move |_| {
        revision.track();
        match register.get() {
            Some(r) => r.by_name().iter().map(|h| h.name().to_owned()).collect(),
            None => core::incident_people(core),
        }
    });
    // The person typed, from the register; `None`: not in it.
    let resolve = move |typed: &str| -> Option<Who> {
        let Some(r) = register.get_untracked() else {
            return Some(Who {
                name: typed.to_owned(),
                ..Who::default()
            });
        };
        let h = r.by_name().into_iter().find(|h| {
            ordo_units::same_person(h.name(), typed)
                || h.fields
                    .account
                    .as_deref()
                    .is_some_and(|a| ordo_units::same_person(a, typed))
        })?;
        Some(Who {
            name: h.name().to_owned(),
            helper: Some(h.id.clone()),
            account: h.fields.account.clone(),
        })
    };
    let field = format!("crew-{}", core::slug(&name));
    let commit = {
        let (name, members) = (name.clone(), members.clone());
        Callback::new(move |who: Who| {
            let mut list = members.clone();
            // Moved here while on the list already: kept once.
            list.retain(|m| !who.is_member(m));
            let mut quals: Vec<String> = CATALOGUE
                .qualifications
                .iter()
                .filter(|q| qualifications.with_untracked(|s| s.contains(&q.id)))
                .map(|q| q.id.clone())
                .collect();
            if quals.is_empty() {
                // None chosen: the helper's, from the register.
                quals = who
                    .helper
                    .as_ref()
                    .and_then(|id| {
                        register.with_untracked(|r| {
                            r.as_ref()?.helpers.get(id)?.fields.qualifications.clone()
                        })
                    })
                    .unwrap_or_default();
            }
            list.push(CrewMember {
                name: who.name,
                qualifications: quals,
                helper: who.helper,
                account: who.account,
            });
            if core::set_crew(core, revision, &name, list).is_ok() {
                person.set(String::new());
                qualifications.set(BTreeSet::new());
                elsewhere.set(Vec::new());
                unregistered.set(false);
            }
        })
    };
    let here = Post::unit(&name);
    // [impl->dsn~double-booking~1]
    // Someone on another crew or UHS is moved here, not counted twice (UNIT-8).
    let place = {
        let here = here.clone();
        Callback::new(move |who: Who| {
            let away = who.elsewhere(core, &here);
            if away.is_empty() {
                commit.run(who);
            } else {
                elsewhere.set(away);
            }
        })
    };
    let add = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let typed = person.get_untracked().trim().to_owned();
        if typed.is_empty() {
            return;
        }
        match resolve(&typed) {
            Some(who) => place.run(who),
            None => unregistered.set(true),
        }
    };
    // Registered with the name alone, then added (HELP-3).
    let register_and_add = Callback::new(move |()| {
        let typed = person.get_untracked().trim().to_owned();
        let Some(id) = core::new_ulid() else { return };
        // A user ID typed is the helper's account, as on UHS staff lists (HELP-2).
        let account = typed
            .parse::<ordo_core::UserId>()
            .is_ok()
            .then(|| typed.clone());
        let change = ordo_helpers::HelperChange {
            helper: id.to_string(),
            fields: ordo_helpers::Fields {
                name: Some(typed.clone()),
                account: account.clone(),
                ..ordo_helpers::Fields::default()
            },
        };
        if change.check(true).is_ok()
            && core::helper_event(core, revision, true, &change, &typed).is_ok()
        {
            unregistered.set(false);
            place.run(Who {
                name: typed,
                helper: Some(change.helper),
                account,
            });
        }
    });
    let on_move = Callback::new(move |()| {
        let typed = person.get_untracked().trim().to_owned();
        if let Some(who) = resolve(&typed) {
            move_away(core, revision, &who, &elsewhere.get_untracked());
            commit.run(who);
        }
    });
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
                <AssignedElsewhere person elsewhere here=here.label.clone() on_move />
                <Show when=move || unregistered.get()>
                    <div class="notice notice--warning assigned-elsewhere" role="alert">
                        <p>{move || text::not_registered(person.get().trim())}</p>
                        <div class="row">
                            <button type="button" class="button button--small"
                                aria-label=move || text::register_and_add_label(person.get().trim())
                                on:click=move |_| register_and_add.run(())>
                                {text::REGISTER_AND_ADD}
                            </button>
                            <button type="button" class="button button--small button--quiet" on:click=move |_| unregistered.set(false)>
                                {text::CANCEL}
                            </button>
                        </div>
                    </div>
                </Show>
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
    /// The open positions (ID, title) and the unit's position (CMD-2).
    positions: Vec<(String, String)>,
    position: Option<String>,
) -> impl IntoView {
    let u = unit;
    let name = u.unit.clone();
    let shown = status_shown(&u);
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
                {u.strength.map(|strength| view! { " · " {text::STRENGTH} " " <StrengthText strength special=u.special.clone() detailed=detailed(core) /> })}
            </p>
            // The status is set in *Führung – Disposition* (ADR-0070); shown here.
            <p class="unit-card__status">{text::COL_STATUS} ": " <strong>{shown}</strong></p>
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

/// "3 – Einsatz übernommen, seit 10:12", or that no status was set.
fn status_shown(u: &Unit) -> String {
    u.status.as_ref().map_or_else(
        || text::NO_STATUS.to_owned(),
        |s| {
            text::status_since(
                &text::status_label(&s.status),
                &core::clock(s.since.wall_ms),
            )
        },
    )
}

/// Choosing a unit's status (UNIT-3).
#[component]
fn StatusSelect(
    core: Handle,
    revision: RwSignal<u64>,
    unit: String,
    current: String,
) -> impl IntoView {
    view! {
        <select aria-label=text::set_status_label(&unit) on:change=move |ev| {
            let value = event_target_value(&ev);
            if !value.is_empty() {
                let _ = core::set_status(core, revision, &unit, &value);
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
}

// [impl->dsn~workspaces-by-phase~4]
// [impl->dsn~tasks~2]
/// The units on scene in *Führung – Disposition* (ADR-0070): status since when, set here, and
/// their open tasks, each with a way to give one (UNIT-3, TASK-3).
#[component]
pub fn UnitStatusBoard(
    core: Handle,
    revision: RwSignal<u64>,
    editable: Memo<bool>,
    /// Opens the form to give a task to the unit with this call sign.
    give: Callback<String>,
) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let tasks = crate::tasks::task_list(core);
        core.with_value(|c| UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)))
            .present()
            .map(|u| {
                let open: Vec<String> = tasks
                    .of_unit(&u.unit)
                    .iter()
                    .map(|t| t.given.text.clone())
                    .collect();
                (u.clone(), open)
            })
            .collect::<Vec<_>>()
    });
    let prepare = crate::nav::Route::view("command.prepare", "units");
    view! {
        <section class="card unit-status" aria-labelledby="unit-status-title">
            <h2 id="unit-status-title">{text::UNIT_STATUS_TITLE}</h2>
            {move || {
                let list = rows.get();
                if list.is_empty() {
                    let prepare = prepare.clone();
                    return view! {
                        <p>
                            {text::UNIT_STATUS_EMPTY} " "
                            <a href=prepare.to_path() on:click=crate::views::follow_route(prepare.clone())>{text::OPEN_UNITS}</a>
                        </p>
                    }
                    .into_any();
                }
                let editable = editable.get();
                view! {
                    <ul class="unit-list unit-status__list" data-keys="list">
                        {list
                            .into_iter()
                            .map(|(u, open)| {
                                let name = u.unit.clone();
                                let current = u.status.as_ref().map(|s| s.status.clone()).unwrap_or_default();
                                let call = name.clone();
                                view! {
                                    <li class="unit-card unit-status__item" data-unit=name.clone()>
                                        <h3 class="unit-card__name">{name.clone()}</h3>
                                        <p class="unit-card__details">{details(&u)}</p>
                                        <p class="unit-status__current">{text::COL_STATUS} ": " <strong>{status_shown(&u)}</strong></p>
                                        {editable.then(|| view! { <StatusSelect core revision unit=name.clone() current /> })}
                                        <p class="unit-status__tasks">
                                            {text::open_tasks_of(open.len())}
                                            {(!open.is_empty()).then(|| format!(": {}", open.join(" · ")))}
                                        </p>
                                        {editable.then(|| view! {
                                            <button type="button" class="button button--small" aria-label=text::task_give_label(&name)
                                                on:click=move |_| give.run(call.clone())>
                                                {text::TASK_GIVE}
                                            </button>
                                        })}
                                    </li>
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

// [impl->dsn~unit-register~3]
// [impl->dsn~views-by-role~1]
// [impl->dsn~workspaces-by-phase~4]
/// The incident's units as planned in *Vorbereitung* (UNIT-1…6, ADR-0070): strength on
/// scene, every unit with its crew, position and release; released units apart; adding
/// units. Adding units: incident command only (`edit_units`).
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
                    elsewhere: RwSignal::new(Vec::new()),
                    unregistered: RwSignal::new(false),
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
            <StrengthOnScene core revision register />
            {move || {
                let units = present.get();
                let editable = can_edit.get();
                revision.track();
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
                                let position = structure.unit_positions.get(&unit.unit).cloned();
                                let positions = positions.clone();
                                view! { <UnitCard core revision unit editable draft positions position /> }
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
                                        special: u.special.clone(),
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

/// Adding a unit (UNIT-1, UNIT-2): organisations and types of the incident's units are
/// offered as choices; anything can be entered.
#[component]
fn AddUnitForm(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let call_sign = RwSignal::new(String::new());
    let unit_type = RwSignal::new(String::new());
    let organisation = RwSignal::new(String::new());
    let service = RwSignal::new(String::new());
    let strength = StrengthDraft::new(None, &Special::new());
    let detailed = Signal::derive(move || {
        revision.track();
        detailed(core)
    });
    let error = RwSignal::new(None::<&'static str>);
    // Organisations and types offered as choices: those of the units already in the
    // incident; anything else can still be typed.
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
        let (reported, special) = match strength.read() {
            Ok(read) => read,
            Err(e) => {
                error.set(Some(e));
                return;
            }
        };
        let service = service.get_untracked();
        let unit = UnitAdded {
            unit: u,
            unit_type: t,
            organisation: organisation.get_untracked().trim().to_owned(),
            service: (!service.is_empty()).then_some(service),
            strength: reported,
            special,
        };
        if core::add_unit(core, revision, unit).is_ok() {
            for s in [call_sign, unit_type, organisation] {
                s.set(String::new());
            }
            strength.clear();
            error.set(None);
        }
    };
    view! {
        <form on:submit=submit class="form add-unit" novalidate>
            <h3>{text::UNIT_SUBMIT}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="unit-call-sign">{text::UNIT_CALL_SIGN}</label>
                    <input id="unit-call-sign" type="text" required autocomplete="off" bind:value=call_sign />
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
            <StrengthFields draft=strength prefix="unit".to_owned() title=text::STRENGTH detailed />
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
            special: Special::new(),
            status: None,
            crew: Vec::new(),
            presence: Vec::new(),
        };
        assert!(unit.is_present());
        assert_eq!(details(&unit), "RTW · DRK · Rettungsdienst");
    }
}
