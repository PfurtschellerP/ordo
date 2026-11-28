//! The incident command's overview (ADR-0067): under the incident's name and state, tiles
//! of different sizes for what the command steers the running incident with: the map,
//! *Disposition*, *Personal*, the UHS and quick links. Counts only; the orders are listed
//! by their follow-up time, nothing else is ranked. Details stay on the pages.

use leptos::prelude::*;
use ordo_med_patients as med;
use ordo_sync::incidents::IncidentState;

use crate::core::{self, Handle};
use crate::med::places::{Level, PlaceRow, TreatmentPlaces};
use crate::med::triage::CATEGORIES;
use crate::shared::text;
use crate::shell::nav::Route;
use crate::shell::ui::Icon;
use crate::shell::views::follow_route;

/// One open order as the tile shows it.
#[derive(Clone, Debug, PartialEq)]
struct NextTask {
    id: String,
    text: String,
    assignee: String,
    status: &'static str,
    follow_up: Option<String>,
    due: bool,
}

/// What the overview shows, computed from the incident's events.
#[derive(Clone, Debug, Default, PartialEq)]
struct Overview {
    name: String,
    state: Option<IncidentState>,
    paused: bool,
    times: Option<String>,
    open_tasks: usize,
    due_tasks: usize,
    next: Vec<NextTask>,
    persons: u32,
    units: u32,
    strength: [u32; 3],
    unreported: u32,
    /// People on more than one post, with their posts (UNIT-8).
    several: Vec<String>,
    uhs: Vec<PlaceRow>,
    /// The patients with units' crews, in words, if any (ADR-0087).
    with_units: Option<String>,
    /// The units by group and type (UNIT-10).
    unit_counts: crate::shared::lage::UnitCounts,
}

fn overview(core: Handle) -> Overview {
    let now = core::now_ms();
    let info = core.with_value(|c| c.incident_info());
    let on_scene = {
        let events = core.with_value(|c| c.events(ordo_units::COMPARTMENT));
        let units = ordo_units::UnitRegister::from_events(&events);
        // [impl->dsn~uhs-as-unit~1]
        crate::shared::units::on_scene(core, &units, &events)
    };
    let tasks = crate::shared::tasks::task_list(core);
    let open = tasks.open(now);
    let structure = crate::shared::command::structure(core);
    let next = open
        .iter()
        .take(3)
        .map(|t| {
            let due = t.is_due(now);
            NextTask {
                id: t.given.task.clone(),
                text: crate::shared::tasks::label(t),
                assignee: crate::shared::tasks::assignee(core, t, &structure),
                status: text::task_phase(t.phase()),
                follow_up: t.follow_up.map(|at| {
                    let at = core::clock(at);
                    if due {
                        text::follow_up_due(&at)
                    } else {
                        text::follow_up_at(&at)
                    }
                }),
                due,
            }
        })
        .collect();
    // [impl->dsn~command-treatment-places~2]
    // From the shared compartment only: the places and the patient counts (UHS-16).
    let counts = core.with_value(|c| med::latest_counts(&c.events(med::UHS_COMPARTMENT)));
    // [impl->dsn~patient-holder~1]
    // Patients with units' crews count under an empty UHS, without naming the units.
    let with_units = counts.get("").and_then(|c| {
        let by = CATEGORIES.map(|k| (text::label(k), c.get(k)));
        let total: u32 = by.iter().map(|(_, n)| n).sum();
        (total > 0).then(|| text::with_units(total, &by))
    });
    let uhs = crate::med::places::place_rows(core, false);
    let details = info.as_ref().map(|i| i.details.clone()).unwrap_or_default();
    Overview {
        name: info.as_ref().map(|i| i.name.clone()).unwrap_or_default(),
        state: info.as_ref().map(|i| i.state),
        paused: info.as_ref().is_some_and(|i| i.paused),
        times: text::incident_times(
            details.starts_at.map(core::date_time).as_deref(),
            details.ends_at.map(core::date_time).as_deref(),
        ),
        open_tasks: open.len(),
        due_tasks: open.iter().filter(|t| t.is_due(now)).count(),
        next,
        persons: on_scene.total.total(),
        units: on_scene.units,
        strength: [
            on_scene.total.leaders,
            on_scene.total.sub_leaders,
            on_scene.total.crew,
        ],
        unreported: on_scene.unreported,
        several: crate::shared::units::several_labels(&on_scene.people.several()),
        uhs,
        with_units,
        unit_counts: crate::shared::lage::UnitCounts::of(&crate::shared::lage::present(core)),
    }
}

/// A tile's head: icon, heading and the link to its page.
#[component]
pub(crate) fn TileHead(
    id: &'static str,
    icon: &'static str,
    title: &'static str,
    #[prop(optional)] route: Option<Route>,
    #[prop(optional)] link: &'static str,
) -> impl IntoView {
    view! {
        <div class="tile__head">
            <span class="tile__icon"><Icon name=icon /></span>
            <h3 id=id class="tile__title">{title}</h3>
            {route.map(|r| view! {
                <a class="tile__link" href=r.to_path() on:click=follow_route(r.clone())>{link}</a>
            })}
        </div>
    }
}

// [impl->dsn~workspaces-by-phase~6]
/// The overview of the incident command: start view of *Einsatzleitung – Führung*.
#[component]
pub fn CommandOverview(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let o = Memo::new(move |_| {
        revision.track();
        overview(core)
    });
    let manage = Memo::new(move |_| {
        revision.track();
        crate::shared::incidents::may_manage(core)
    });
    let error = RwSignal::new(None::<String>);
    let start = move |_| {
        if let Err(e) = core::set_incident_state(core, revision, IncidentState::Active) {
            error.set(Some(e));
        }
    };
    let map = Route::view("command.lead", "map");
    let dispatch = Route::view("command.lead", "dispatch");
    let transports = Route::view("command.lead", "transports");
    // [impl->dsn~command-lage~1]
    // The tiles open their tab of *Lage* (ADR-0099).
    let uhs = Route::record("command.lead", "map", "places");
    let lage_units = Route::record("command.lead", "map", "units");
    let log = Route::view("incident.log", "log");
    // The command structure is on Führung & Raum (ADR-0095).
    let structure = Route::view("command.prepare", "map");
    let quick = [
        (
            "quick-task",
            "clipboard-list",
            text::task_give(),
            dispatch.clone(),
            true,
        ),
        (
            "quick-transports",
            "ambulance",
            text::transports(),
            transports,
            false,
        ),
        ("quick-log", "notebook-text", text::log_title(), log, false),
        (
            "quick-structure",
            "network",
            text::structure_title(),
            structure,
            false,
        ),
    ];
    view! {
        <section class="overview" aria-labelledby="overview-title">
            <div class="overview__head">
                <h2 id="overview-title">{move || o.with(|o| o.name.clone())}</h2>
                <span id="overview-state" class="tag state-tag" data-state=move || o.with(|o| match (o.state, o.paused) {
                    (Some(IncidentState::Active), true) => "paused",
                    (Some(IncidentState::Active), false) => "active",
                    (Some(IncidentState::Closed), _) => "closed",
                    _ => "planned",
                })>
                    {move || o.with(|o| o.state.map(|s| text::incident_state(s, o.paused)))}
                </span>
                <span class="hint">{move || o.with(|o| o.times.clone())}</span>
                {move || (manage.get() && o.with(|o| o.state == Some(IncidentState::Planned))).then(|| view! {
                    <button type="button" class="button button--primary" on:click=start>{text::start_incident()}</button>
                })}
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="bento">
                <section class="tile tile--map" aria-labelledby="tile-map">
                    <TileHead id="tile-map" icon="map" title=text::map_title() route=map link=text::open_map() />
                    <crate::shared::situation::MapPreview core revision />
                </section>

                <section class="tile tile--dispatch" aria-labelledby="tile-dispatch">
                    <TileHead id="tile-dispatch" icon="clipboard-list" title=text::dispatch() route=dispatch.clone() link=text::open_dispatch() />
                    <p class="tile__figure">
                        <span class="tile__number">{move || o.with(|o| o.open_tasks)}</span>
                        " " {text::open_label()}
                        {move || o.with(|o| (o.due_tasks > 0).then(|| view! {
                            <span class="tag tag--warning">{text::follow_ups_due(o.due_tasks)}</span>
                        }))}
                    </p>
                    {move || {
                        let next = o.with(|o| o.next.clone());
                        if next.is_empty() {
                            return view! { <p class="hint">{text::no_open_tasks()}</p> }.into_any();
                        }
                        view! {
                            <ul class="tile__list">
                                {next.into_iter().map(|t| view! {
                                    <li class="tile__task" data-task=t.id>
                                        <span class="tile__task-text">
                                            <strong>{t.text}</strong>
                                            <span class="hint">{text::task_for(&t.assignee)} {t.follow_up.map(|f| format!(" · {f}"))}</span>
                                        </span>
                                        <span class=if t.due { "tag tag--warning" } else { "tag" }>{t.status}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>

                <section class="tile tile--people" aria-labelledby="tile-people">
                    // Leading stays in Führung: to the units in *Lage*, not where they are
                    // planned (feedback of 2026-11-27).
                    <TileHead id="tile-people" icon="users" title=text::people() route=lage_units.clone() link=text::open_units() />
                    <div class="tile__pair">
                        <p class="tile__figure tile__figure--stacked">
                            <span id="overview-persons" class="tile__number">
                                {move || o.with(|o| o.persons)}
                                // [impl->dsn~double-booking~1]
                                {move || o.with(|o| !o.several.is_empty()).then(|| view! { <span aria-hidden="true">"*"</span> })}
                            </span>
                            <span>{text::persons_on_scene()}</span>
                        </p>
                        <p class="tile__figure tile__figure--stacked">
                            <span id="overview-units" class="tile__number">{move || o.with(|o| o.units)}</span>
                            <span>{text::units_on_scene()}</span>
                        </p>
                    </div>
                    <dl class="tile__strength" aria-label=text::strength_legend_title()>
                        {move || {
                            let [l, s, c] = o.with(|o| o.strength);
                            view! {
                                <div><dt>{text::leaders()}</dt><dd>{l}</dd></div>
                                <div><dt>{text::sub_leaders()}</dt><dd>{s}</dd></div>
                                <div><dt>{text::crew_count()}</dt><dd>{c}</dd></div>
                            }
                        }}
                    </dl>
                    <p class="hint">{move || o.with(|o| text::unreported_strength(o.unreported))}</p>
                    {move || o.with(|o| (!o.several.is_empty()).then(|| view! {
                        <p class="hint tile__several"><span aria-hidden="true">"* "</span>{text::double_booked(&o.several)}</p>
                    }))}
                </section>

                // [impl->dsn~command-treatment-places~2]
                <section class="tile tile--uhs" aria-labelledby="tile-uhs">
                    <TileHead id="tile-uhs" icon="tent" title=text::places() route=uhs link=text::open_lage() />
                    <TreatmentPlaces rows=Signal::derive(move || o.with(|o| o.uhs.clone())) level=Level::H4 />
                    {move || o.with(|o| o.with_units.clone()).map(|t| view! { <p class="tile__with-units">{t}</p> })}
                </section>

                // [impl->dsn~unit-overview~1]
                // Units by group, free per type (UNIT-10), beside the places and the people.
                <section class="tile tile--units" aria-labelledby="tile-units">
                    <TileHead id="tile-units" icon="truck" title=text::units_title() route=lage_units link=text::open_lage() />
                    {move || {
                        let c = o.with(|o| o.unit_counts.clone());
                        view! {
                            <p class="tile__figure">
                                <span id="overview-available" class="tile__number">{c.groups[0]}</span>
                                " " {text::units_available_of(c.total)}
                            </p>
                            <ul class="tile__groups">
                                {ordo_units::StatusGroup::ALL.iter().zip(c.groups).map(|(g, n)| view! {
                                    <li data-group=format!("{g:?}").to_lowercase()>
                                        <crate::shared::lage::GroupLabel group=Some(*g) />
                                        <span class="tile__count">{n}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                            {(!c.types.is_empty()).then(|| view! {
                                <table class="table tile__types">
                                    <thead>
                                        <tr>
                                            <th scope="col">{text::col_type()}</th>
                                            <th scope="col" class="number">{text::col_free()}</th>
                                            <th scope="col" class="number">{text::col_all()}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {c.types.into_iter().map(|(t, free, all)| view! {
                                            <tr><th scope="row">{t}</th><td class="number">{free}</td><td class="number">{all}</td></tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            })}
                        }
                    }}
                </section>

                // [impl->dsn~tasks~9]
                <crate::med::transports::TransportsTile core revision />

                <section class="tile tile--quick" aria-labelledby="tile-quick">
                    <TileHead id="tile-quick" icon="zap" title=text::quick_links() />
                    <ul class="quick-links">
                        {quick.into_iter().map(|(id, icon, label, route, primary)| view! {
                            <li>
                                <a id=id class=if primary { "quick-link quick-link--primary" } else { "quick-link" }
                                    href=route.to_path() on:click=follow_route(route.clone())>
                                    <Icon name=icon />{label}
                                </a>
                            </li>
                        }).collect_view()}
                    </ul>
                </section>
            </div>
        </section>
    }
}
