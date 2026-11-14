//! Reports (REP-1…5, ADR-0075): recorded and followed by the incident command in
//! *Führung – Disposition*, assigned to units as linked tasks; the crews of those units see
//! their patient cases in *UHS – Patienten* and register the patient from them.

use leptos::prelude::*;
use ordo_map::ObjectKind;
use ordo_reports::{Recorded, Report, ReportKind, ReportList};
use ordo_tasks::TaskGiven;
use ordo_units::UnitRegister;

use crate::core::{self, Handle};
use crate::text;
use crate::ui::{Disclosure, Icon};

/// The incident's reports, computed from the shared events.
pub fn list(core: Handle) -> ReportList {
    core.with_value(|c| ReportList::from_events(&c.events(ordo_reports::COMPARTMENT)))
}

/// The kind as the profile's symbol rules name it.
pub fn kind_id(kind: ReportKind) -> &'static str {
    match kind {
        ReportKind::Event => "event",
        ReportKind::Patient => "patient",
    }
}

/// A report in a few words, e.g. "Patient: Bühne Nord", for lists and the map.
pub fn label(r: &Recorded) -> String {
    text::report_label(r.kind, &r.place)
}

/// The map object of a report's place.
fn object(report: &str) -> String {
    ObjectKind::Report.object_id(report)
}

/// The report's place on the map, if it was placed: longitude, latitude.
fn placed_at(core: Handle, report: &str) -> Option<[f64; 2]> {
    core.with_value(|c| {
        ordo_map::SituationMap::from_events(&c.events(ordo_map::COMPARTMENT))
            .objects
            .get(&object(report))
            .and_then(|o| o.placed.geometry.anchor())
    })
}

/// A `geo:` URI (RFC 5870) that opens the device's maps app at a point.
pub fn geo_uri([lon, lat]: [f64; 2]) -> String {
    format!("geo:{lat:.6},{lon:.6}")
}

/// The patient case being registered from, shared by the cases and the registration form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    /// The report.
    pub report: String,
    /// The unit whose crew takes the patient over.
    pub unit: String,
    /// The report's place, to say which case it is.
    pub place: String,
}

/// The case the registration form registers from, if any (context of *Patienten*).
#[derive(Clone, Copy)]
pub struct ActiveCase(pub RwSignal<Option<Case>>);

// [impl->dsn~reports~1]
/// The UHS to offer first for a unit's patient: one under the unit's position in the command
/// structure, else under the nearest position above it.
pub fn unit_uhs(structure: &ordo_command::CommandStructure, unit: &str) -> Option<String> {
    let mut position = structure.unit_positions.get(unit)?;
    for _ in 0..structure.positions.len() {
        let p = structure.positions.get(position)?;
        if let Some(uhs) = p.uhs.first() {
            return Some(uhs.clone());
        }
        position = p.created.parent.as_ref()?;
    }
    None
}

// [impl->dsn~reports~1]
/// *Führung – Disposition*, at the top: the open reports with their units, and the form
/// to record one (REP-1, REP-2, REP-5).
#[component]
pub fn ReportsPanel(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let editable = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
    // Each open report with its tasks and whether it is on the map, so a card is drawn
    // anew when any of them changes.
    let open = Memo::new(move |_| {
        revision.track();
        let tasks = crate::tasks::task_list(core);
        list(core)
            .open()
            .into_iter()
            .map(|r| {
                let id = &r.recorded.report;
                Shown {
                    tasks: tasks
                        .of_report(id)
                        .into_iter()
                        .map(|t| {
                            (
                                t.given.unit.clone().unwrap_or_default(),
                                text::task_status(t.status),
                            )
                        })
                        .collect(),
                    placed: placed_at(core, id).is_some(),
                    report: r.clone(),
                }
            })
            .collect::<Vec<_>>()
    });
    let recording = RwSignal::new(false);
    view! {
        <section class="card reports-panel" aria-labelledby="reports-title">
            <h2 id="reports-title">{move || text::reports_title(open.with(Vec::len))}</h2>
            {move || {
                let reports = open.get();
                if reports.is_empty() {
                    return view! { <p>{text::REPORTS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="report-list" data-keys="list">
                        {reports.into_iter().map(|shown| view! {
                            <ReportCard core revision shown editable=editable.get() />
                        }).collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <Show when=move || editable.get()>
                <Disclosure
                    id="record-report"
                    open=recording
                    class="tasks-panel__give"
                    button_class="button button--small"
                    panel_class="tasks-panel__form"
                    button=Box::new(|| view! { {text::REPORT_RECORD} }.into_any())
                >
                    <RecordReport core revision />
                </Disclosure>
            </Show>
        </section>
    }
}

/// What, where and when, with a word and an icon for the kind (never colour alone).
#[component]
fn ReportHead(recorded: Recorded) -> impl IntoView {
    let icon = match recorded.kind {
        ReportKind::Event => "zap",
        ReportKind::Patient => "users",
    };
    view! {
        <p>
            <span class="report-card__kind"><Icon name=icon />{text::report_kind(recorded.kind)}</span>
            " · " {text::report_received(&core::clock(recorded.received_at))}
        </p>
        <p><strong>{recorded.place.clone()}</strong> " – " {recorded.text.clone()}</p>
    }
}

/// An open report as Disposition shows it.
#[derive(Clone, PartialEq)]
struct Shown {
    report: Report,
    /// Units of its tasks, with each task's status.
    tasks: Vec<(String, &'static str)>,
    /// Whether it is on the map.
    placed: bool,
}

/// One open report: its units with their task status, whether a patient was taken over,
/// and the command's actions.
#[component]
fn ReportCard(
    core: Handle,
    revision: RwSignal<u64>,
    shown: Shown,
    editable: bool,
) -> impl IntoView {
    let Shown {
        report,
        tasks,
        placed,
    } = shown;
    let id = report.recorded.report.clone();
    let taken: Vec<String> = report
        .patients
        .iter()
        .map(|p| text::report_taken(&p.unit, &core::clock(p.at.wall_ms)))
        .collect();
    let patient = report.recorded.kind == ReportKind::Patient;
    let data_id = id.clone();
    let place_route = crate::situation::editor_route(Some(&crate::situation::place_record(
        ObjectKind::Report,
        &id,
    )));
    view! {
        <li class="report-card" class:report-card--patient=patient data-report=data_id>
            <ReportHead recorded=report.recorded.clone() />
            {(!tasks.is_empty()).then(|| view! {
                <ul class="report-card__units">
                    {tasks.into_iter().map(|(unit, status)| view! { <li>{unit} " – " {status}</li> }).collect_view()}
                </ul>
            })}
            {taken.into_iter().map(|t| view! { <p class="report-card__taken">{t}</p> }).collect_view()}
            {placed.then(|| view! { <p class="hint"><Icon name="map-pin" />{text::REPORT_ON_MAP}</p> })}
            {editable.then(|| view! {
                <AssignReport core revision recorded=report.recorded.clone() />
                <div class="row">
                    <a class="button button--small" href=place_route.to_path()
                        aria-label=text::report_place_label(placed, &label(&report.recorded))
                        on:click=crate::views::follow_route(place_route.clone())>
                        {if placed { text::REPORT_MOVE_ON_MAP } else { text::REPORT_PLACE_ON_MAP }}
                    </a>
                    <button type="button" class="button button--small button--quiet"
                        aria-label=text::report_close_label(&label(&report.recorded))
                        on:click=move |_| { let _ = core::close_report(core, revision, &id); }>
                        {text::REPORT_CLOSE}
                    </button>
                </div>
            })}
        </li>
    }
}

/// Assigning a report to a unit (REP-2): gives the unit a task that names the report.
#[component]
fn AssignReport(core: Handle, revision: RwSignal<u64>, recorded: Recorded) -> impl IntoView {
    let unit = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let units = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .present()
                .map(|u| u.unit.clone())
                .collect::<Vec<_>>()
        })
    });
    let field = format!("assign-{}", recorded.report);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let chosen = unit.get_untracked();
        if chosen.is_empty() {
            error.set(Some(text::REPORT_UNIT_MISSING));
            return;
        }
        let task = TaskGiven {
            task: String::new(),
            unit: Some(chosen),
            sector: None,
            text: text::report_task(&recorded),
            template: None,
            follow_up: None,
            report: Some(recorded.report.clone()),
        };
        if core::give_task(core, revision, task).is_ok() {
            unit.set(String::new());
            error.set(None);
        }
    };
    view! {
        <form class="form--inline report-card__assign" novalidate on:submit=submit>
            <div class="field">
                <label for=field.clone()>{text::REPORT_ASSIGN_TO}</label>
                <select id=field prop:value=move || unit.get() on:change=move |ev| unit.set(event_target_value(&ev))>
                    <option value="">{text::TASK_UNIT_CHOOSE}</option>
                    {move || units.get().into_iter().map(|u| view! { <option value=u.clone()>{u.clone()}</option> }).collect_view()}
                </select>
            </div>
            <button type="submit" class="button button--small">{text::REPORT_ASSIGN}</button>
            <p role="alert" class="error">{move || error.get()}</p>
        </form>
    }
}

/// Recording a report (REP-1): kind, place, a short description without medical details,
/// and when it came in (empty: now).
#[component]
fn RecordReport(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let kind = RwSignal::new(None::<ReportKind>);
    let place = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let received = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(chosen) = kind.get_untracked() else {
            error.set(Some(text::REPORT_KIND_MISSING));
            return;
        };
        let now = core::now_ms();
        let received_at = match received.get_untracked().trim() {
            "" => now,
            time => match core::today_at(time, now) {
                Some(at) => at,
                None => {
                    error.set(Some(text::REPORT_TIME_INVALID));
                    return;
                }
            },
        };
        let report = Recorded {
            report: String::new(),
            kind: chosen,
            place: place.get_untracked().trim().to_owned(),
            text: description.get_untracked().trim().to_owned(),
            received_at,
        };
        if let Err(e) = report.check() {
            error.set(Some(match e {
                ordo_reports::ReportError::NoPlace => text::REPORT_PLACE_MISSING,
                ordo_reports::ReportError::NoText => text::REPORT_TEXT_MISSING,
                ordo_reports::ReportError::TooLong => text::REPORT_TOO_LONG,
            }));
            return;
        }
        if core::record_report(core, revision, report).is_ok() {
            kind.set(None);
            for s in [place, description, received] {
                s.set(String::new());
            }
            error.set(None);
        }
    };
    view! {
        <form class="form record-report" novalidate on:submit=submit>
            <fieldset class="choices">
                <legend>{text::REPORT_KIND}</legend>
                {[ReportKind::Patient, ReportKind::Event].into_iter().map(|k| view! {
                    <label class="choice">
                        <input type="radio" name="report-kind" value=kind_id(k)
                            prop:checked=move || kind.get() == Some(k)
                            on:change=move |_| kind.set(Some(k)) />
                        {text::report_kind(k)}
                    </label>
                }).collect_view()}
            </fieldset>
            <div class="field">
                <label for="report-place">{text::REPORT_PLACE}</label>
                <input id="report-place" type="text" autocomplete="off" maxlength=ordo_reports::PLACE_MAX bind:value=place />
            </div>
            <div class="field">
                <label for="report-text">{text::REPORT_TEXT}</label>
                <input id="report-text" type="text" autocomplete="off" aria-describedby="report-text-hint"
                    maxlength=ordo_reports::TEXT_MAX bind:value=description />
                <p id="report-text-hint" class="hint">{text::REPORT_TEXT_HINT}</p>
            </div>
            <div class="field field--small">
                <label for="report-received">{text::REPORT_RECEIVED}</label>
                <input id="report-received" type="time" autocomplete="off"
                    aria-describedby="report-received-hint" bind:value=received />
                <p id="report-received-hint" class="hint">{text::REPORT_RECEIVED_HINT}</p>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button">{text::REPORT_RECORD}</button>
            </div>
        </form>
    }
}

/// An open patient case assigned to one of this person's units, not yet taken over by it.
#[derive(Clone, PartialEq)]
struct AssignedCase {
    recorded: Recorded,
    unit: String,
    at: Option<[f64; 2]>,
}

// [impl->dsn~reports~1]
/// *UHS – Patienten*, at the top: the open patient cases assigned to the units whose crew
/// names this person (REP-3, REP-4), with the place on a map and for the maps app, and
/// "Patient aufnehmen", which fills the registration with the case.
#[component]
pub fn AssignedCases(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let active = use_context::<ActiveCase>();
    let cases = Memo::new(move |_| {
        revision.track();
        let me = core.with_value(|c| c.replica.user().to_string());
        let units: Vec<String> = core.with_value(|c| {
            UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .crewed_by(&me)
                .map(|u| u.unit.clone())
                .collect()
        });
        let tasks = crate::tasks::task_list(core);
        let mut cases = Vec::new();
        for report in list(core).open() {
            if report.recorded.kind != ReportKind::Patient {
                continue;
            }
            for task in tasks.of_report(&report.recorded.report) {
                let Some(unit) = task.given.unit.as_ref() else {
                    continue;
                };
                let taken = report.patients.iter().any(|p| &p.unit == unit);
                if task.status.is_open() && !taken && units.contains(unit) {
                    cases.push(AssignedCase {
                        recorded: report.recorded.clone(),
                        unit: unit.clone(),
                        at: placed_at(core, &report.recorded.report),
                    });
                }
            }
        }
        cases
    });
    view! {
        <Show when=move || cases.with(|c| !c.is_empty())>
            <section class="card assigned-cases" aria-labelledby="assigned-cases-title">
                <h2 id="assigned-cases-title">{text::ASSIGNED_CASES}</h2>
                <ul class="report-list">
                    {move || cases.get().into_iter().map(|case| {
                        let object = object(&case.recorded.report);
                        let chosen = Case {
                            report: case.recorded.report.clone(),
                            unit: case.unit.clone(),
                            place: case.recorded.place.clone(),
                        };
                        let take = move |_| {
                            if let Some(active) = active {
                                active.0.set(Some(chosen.clone()));
                                request_animation_frame(|| crate::views::focus("tag"));
                            }
                        };
                        view! {
                            <li class="report-card report-card--patient" data-report=case.recorded.report.clone()>
                                <ReportHead recorded=case.recorded.clone() />
                                <p>{text::case_for(&case.unit)}</p>
                                {case.at.map(|at| view! {
                                    <crate::situation::PlaceMap core revision object />
                                    <p><a href=geo_uri(at)><Icon name="map-pin" />{text::OPEN_IN_MAPS_APP}</a></p>
                                })}
                                <div class="row">
                                    <button type="button" class="button button--primary" on:click=take>
                                        {text::CASE_REGISTER}
                                    </button>
                                </div>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </section>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~reports~1]
    #[test]
    fn a_geo_uri_names_latitude_first() {
        assert_eq!(geo_uri([8.2473, 49.9929]), "geo:49.992900,8.247300");
    }
}
