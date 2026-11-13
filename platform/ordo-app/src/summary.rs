//! The incident command's overview (ADR-0067): under the incident's name and state, tiles
//! of different sizes for what the command steers the running incident with: the map,
//! *Disposition*, *Personal*, the UHS and quick links. Counts only; the orders are listed
//! by their follow-up time, nothing else is ranked. Details stay on the pages.

use leptos::prelude::*;
use ordo_med_patients as med;
use ordo_sync::incidents::IncidentState;

use crate::core::{self, Handle};
use crate::nav::Route;
use crate::text;
use crate::ui::Icon;
use crate::views::{CATEGORIES, TriageBadge, follow_route};

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

/// One UHS: its load and counts.
#[derive(Clone, Debug, PartialEq)]
struct UhsTile {
    name: String,
    capacity: u32,
    /// Patients per triage category, in the order of [`CATEGORIES`].
    per: [u32; 6],
    discharged: u32,
    transports: usize,
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
    uhs: Vec<UhsTile>,
}

fn overview(core: Handle) -> Overview {
    let now = core::now_ms();
    let info = core.with_value(|c| c.incident_info());
    let on_scene = core.with_value(|c| {
        let events = c.events(ordo_units::COMPARTMENT);
        let units = ordo_units::UnitRegister::from_events(&events);
        // [impl->dsn~uhs-as-unit~1]
        med::strength_on_scene(&units, &events)
    });
    let tasks = crate::tasks::task_list(core);
    let open = tasks.open(now);
    let structure = crate::command::structure(core);
    let next = open
        .iter()
        .take(3)
        .map(|t| {
            let due = t.is_due(now);
            NextTask {
                id: t.given.task.clone(),
                text: t.given.text.clone(),
                assignee: crate::tasks::assignee(t, &structure),
                status: text::task_status(t.status),
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
    let transports = core.with_value(|c| {
        med::transport::TransportBoard::from_events(&c.events(med::transport::COMPARTMENT))
    });
    let counts = core.with_value(|c| med::latest_counts(&c.events(med::UHS_COMPARTMENT)));
    let uhs = crate::views::uhs_list(core)
        .into_iter()
        .map(|u| UhsTile {
            transports: transports
                .transports
                .values()
                .filter(|t| t.status.is_open() && t.request.uhs == u.id)
                .count(),
            per: {
                let c = counts.get(&u.id).cloned().unwrap_or_default();
                CATEGORIES.map(|k| c.get(k))
            },
            discharged: counts.get(&u.id).map_or(0, |c| c.discharged),
            name: u.name,
            capacity: u.capacity,
        })
        .collect();
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
        uhs,
    }
}

/// A tile's head: icon, heading and the link to its page.
#[component]
fn TileHead(
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

/// One UHS (or all of them): load against capacity, patients per category, transports.
#[component]
fn UhsCard(tile: UhsTile, #[prop(optional)] total: bool) -> impl IntoView {
    let patients: u32 = tile.per.iter().sum();
    let share = (patients * 100)
        .checked_div(tile.capacity)
        .unwrap_or(0)
        .min(100);
    view! {
        <article class=if total { "uhs-tile uhs-tile--total" } else { "uhs-tile" } aria-label=tile.name.clone()>
            <h4 class="uhs-tile__name">{tile.name.clone()}</h4>
            <p class="uhs-tile__load">
                <span class="tile__number tile__number--small">{patients}</span>
                " " {text::uhs_load(patients, tile.capacity)}
            </p>
            {(tile.capacity > 0).then(|| view! {
                <div class="uhs-tile__bar" aria-hidden="true">
                    <div class="uhs-tile__fill" style:width=format!("{share}%")></div>
                </div>
            })}
            <ul class="uhs-tile__counts" aria-label=text::BY_CATEGORY>
                {CATEGORIES
                    .into_iter()
                    .zip(tile.per)
                    .filter(|(_, n)| *n > 0)
                    .map(|(k, n)| view! { <li><TriageBadge category=k /> " " {n}</li> })
                    .collect_view()}
            </ul>
            <p class="hint">
                {text::transports_open(tile.transports)}
                {(tile.discharged > 0).then(|| format!(" · {}", text::discharged_count(tile.discharged)))}
            </p>
        </article>
    }
}

// [impl->dsn~workspaces-by-phase~2]
/// The overview of the incident command: start view of *Einsatzleitung – Führung*.
#[component]
pub fn CommandOverview(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let o = Memo::new(move |_| {
        revision.track();
        overview(core)
    });
    let manage = Memo::new(move |_| {
        revision.track();
        crate::incidents::may_manage(core)
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
    let units = Route::view("command.prepare", "units");
    let uhs = Route::view("command.lead", "map");
    let log = Route::view("incident.log", "log");
    let people = Route::view("command.prepare", "people");
    let quick = [
        (
            "quick-task",
            "clipboard-list",
            text::TASK_GIVE,
            dispatch.clone(),
            true,
        ),
        (
            "quick-transports",
            "ambulance",
            text::TRANSPORTS,
            transports,
            false,
        ),
        ("quick-log", "notebook-text", text::LOG_TITLE, log, false),
        (
            "quick-structure",
            "network",
            text::STRUCTURE_TITLE,
            people,
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
                    <button type="button" class="button button--primary" on:click=start>{text::START_INCIDENT}</button>
                })}
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="bento">
                <section class="tile tile--map" aria-labelledby="tile-map">
                    <TileHead id="tile-map" icon="map" title=text::MAP_TITLE route=map link=text::OPEN_MAP />
                    <crate::situation::MapPreview core revision />
                </section>

                <section class="tile tile--dispatch" aria-labelledby="tile-dispatch">
                    <TileHead id="tile-dispatch" icon="clipboard-list" title=text::DISPATCH route=dispatch.clone() link=text::OPEN_DISPATCH />
                    <p class="tile__figure">
                        <span class="tile__number">{move || o.with(|o| o.open_tasks)}</span>
                        " " {text::OPEN_LABEL}
                        {move || o.with(|o| (o.due_tasks > 0).then(|| view! {
                            <span class="tag tag--warning">{text::follow_ups_due(o.due_tasks)}</span>
                        }))}
                    </p>
                    {move || {
                        let next = o.with(|o| o.next.clone());
                        if next.is_empty() {
                            return view! { <p class="hint">{text::NO_OPEN_TASKS}</p> }.into_any();
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
                    <TileHead id="tile-people" icon="users" title=text::PEOPLE route=units link=text::OPEN_UNITS />
                    <div class="tile__pair">
                        <p class="tile__figure tile__figure--stacked">
                            <span id="overview-persons" class="tile__number">{move || o.with(|o| o.persons)}</span>
                            <span>{text::PERSONS_ON_SCENE}</span>
                        </p>
                        <p class="tile__figure tile__figure--stacked">
                            <span id="overview-units" class="tile__number">{move || o.with(|o| o.units)}</span>
                            <span>{text::UNITS_ON_SCENE}</span>
                        </p>
                    </div>
                    <dl class="tile__strength" aria-label=text::STRENGTH_LEGEND>
                        {move || {
                            let [l, s, c] = o.with(|o| o.strength);
                            view! {
                                <div><dt>{text::LEADERS}</dt><dd>{l}</dd></div>
                                <div><dt>{text::SUB_LEADERS}</dt><dd>{s}</dd></div>
                                <div><dt>{text::CREW_COUNT}</dt><dd>{c}</dd></div>
                            }
                        }}
                    </dl>
                    <p class="hint">{move || o.with(|o| text::unreported_strength(o.unreported))}</p>
                </section>

                <section class="tile tile--uhs" aria-labelledby="tile-uhs">
                    <TileHead id="tile-uhs" icon="tent" title=text::UHS_TILE route=uhs link=text::OPEN_COUNTS />
                    {move || {
                        let list = o.with(|o| o.uhs.clone());
                        if list.is_empty() {
                            return view! { <p class="hint">{text::UHS_EMPTY}</p> }.into_any();
                        }
                        let total = (list.len() > 1).then(|| UhsTile {
                            name: text::COL_TOTAL.to_owned(),
                            capacity: list.iter().map(|u| u.capacity).sum(),
                            per: std::array::from_fn(|i| list.iter().map(|u| u.per[i]).sum()),
                            discharged: list.iter().map(|u| u.discharged).sum(),
                            transports: list.iter().map(|u| u.transports).sum(),
                        });
                        view! {
                            <div class="uhs-tiles">
                                {list.into_iter().map(|tile| view! { <UhsCard tile /> }).collect_view()}
                                {total.map(|tile| view! { <UhsCard tile total=true /> })}
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="tile tile--quick" aria-labelledby="tile-quick">
                    <TileHead id="tile-quick" icon="zap" title=text::QUICK_LINKS />
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
