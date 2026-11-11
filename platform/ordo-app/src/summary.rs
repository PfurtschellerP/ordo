//! The incident command's overview (ADR-0061): one short summary per page of the package
//! *Einsatzleitung*, each linking to its page, and quick actions. Details stay on the
//! pages; the overview only counts.

use leptos::prelude::*;
use ordo_med_patients as med;
use ordo_sync::incidents::IncidentState;

use crate::core::{self, Handle};
use crate::nav::Route;
use crate::text;

/// What the overview shows, computed from the incident's events.
#[derive(Clone, Debug, Default, PartialEq)]
struct Summary {
    name: String,
    state: Option<IncidentState>,
    people: usize,
    units: u32,
    persons: u32,
    open_tasks: usize,
    due_tasks: usize,
    transports: usize,
    patients: u32,
    uhs: usize,
    positions: usize,
    on_map: usize,
}

fn summary(core: Handle) -> Summary {
    let now = core::now_ms();
    let info = core.with_value(|c| c.incident_info());
    let units = core
        .with_value(|c| ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)));
    let on_scene = units.strength_on_scene();
    let tasks = crate::tasks::task_list(core);
    let open = tasks.open(now);
    let transports = core.with_value(|c| {
        med::transport::TransportBoard::from_events(&c.events(med::transport::COMPARTMENT))
    });
    let counts = core.with_value(|c| med::latest_counts(&c.events(med::UHS_COMPARTMENT)));
    let patients = counts
        .values()
        .map(|c| {
            crate::views::CATEGORIES
                .iter()
                .map(|k| c.get(*k))
                .sum::<u32>()
        })
        .sum();
    let structure = crate::command::structure(core);
    let on_map = core.with_value(|c| {
        ordo_map::SituationMap::from_events(&c.events(ordo_map::COMPARTMENT))
            .objects
            .len()
    });
    Summary {
        name: info.as_ref().map(|i| i.name.clone()).unwrap_or_default(),
        state: info.as_ref().map(|i| i.state),
        people: info.as_ref().map_or(0, |i| i.roster.len()),
        units: on_scene.units,
        persons: on_scene.total.total(),
        open_tasks: open.len(),
        due_tasks: open.iter().filter(|t| t.is_due(now)).count(),
        transports: transports
            .transports
            .values()
            .filter(|t| t.status.is_open())
            .count(),
        patients,
        uhs: counts.len(),
        positions: structure.positions.values().filter(|p| p.open).count(),
        on_map,
    }
}

/// One card: a page's name, what it holds, a link to it and perhaps quick actions.
#[component]
fn Card(
    id: &'static str,
    title: &'static str,
    route: Route,
    #[prop(into)] text: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let heading = format!("summary-{id}");
    view! {
        <li class="summary__card card">
            <h2 id=heading>{title}</h2>
            <p class="summary__text">{move || text.get()}</p>
            <div class="row">
                <a class="button button--small" href=route.to_path() on:click=crate::views::follow_route(route.clone())>{text::summary_open(title)}</a>
                {children.map(|c| c())}
            </div>
        </li>
    }
}

// [impl->dsn~workspaces-by-phase~1]
/// The overview of the incident command: start view of *Einsatzleitung – Lage*.
#[component]
pub fn CommandOverview(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let s = Memo::new(move |_| {
        revision.track();
        summary(core)
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
    let field = move |f: fn(&Summary) -> String| Signal::derive(move || s.with(f));
    view! {
        <section class="summary" aria-labelledby="summary-title">
            <h2 id="summary-title" class="visually-hidden">{text::OVERVIEW}</h2>
            <ul class="summary__cards">
                <Card id="incident" title=text::INCIDENT_PAGE route=Route::view("command.prepare", "incident")
                    text=field(|s| text::summary_incident(&s.name, s.state.map(text::state_label)))>
                    {move || (manage.get() && s.with(|s| s.state == Some(IncidentState::Planned))).then(|| view! {
                        <button type="button" class="button button--small button--primary" on:click=start>{text::START_INCIDENT}</button>
                    })}
                </Card>
                <Card id="map" title=text::MAP_TITLE route=Route::view("command.lead", "map")
                    text=field(|s| text::summary_map(s.on_map)) />
                <Card id="patients" title=text::COUNTS_TITLE route=Route::view("command.lead", "map")
                    text=field(|s| text::summary_patients(s.patients, s.uhs)) />
                <Card id="tasks" title=text::DISPATCH route=Route::view("command.lead", "dispatch")
                    text=field(|s| text::summary_tasks(s.open_tasks, s.due_tasks)) />
                <Card id="transports" title=text::TRANSPORTS route=Route::view("command.lead", "transports")
                    text=field(|s| text::summary_transports(s.transports)) />
                <Card id="units" title=text::TAB_RESOURCES route=Route::view("command.prepare", "units")
                    text=field(|s| text::summary_units(s.units, s.persons)) />
                <Card id="people" title=text::PEOPLE route=Route::view("command.prepare", "people")
                    text=field(|s| text::summary_people(s.people)) />
                <Card id="structure" title=text::STRUCTURE_TITLE route=Route::view("command.prepare", "structure")
                    text=field(|s| text::summary_positions(s.positions)) />
                <Card id="log" title=text::LOG_TITLE route=Route::view("incident.log", "log")
                    text=Signal::derive(|| text::SUMMARY_LOG.to_owned()) />
            </ul>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}
