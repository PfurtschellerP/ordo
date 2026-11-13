//! The automatic incident log, *Einsatztagebuch* (LOG-1, LOG-3, LOG-6): every event this
//! device holds for the incident, newest first, in words, with time, author and device.
//! The log is computed from the events (`Replica::log`), so it can only grow; a
//! correction is a new entry that links to the one it corrects.

use std::collections::BTreeMap;

use leptos::prelude::*;
use ordo_core::codec;
use ordo_sync::AppliedEvent;
use serde::de::DeserializeOwned;

use crate::core::{self, Handle};
use crate::text;

/// What other entries of the log say about the things an entry names: names of UHS,
/// positions, hospitals and places, task texts, the patient of an order.
#[derive(Default)]
struct Context {
    uhs: BTreeMap<String, String>,
    positions: BTreeMap<String, String>,
    tasks: BTreeMap<String, String>,
    orders: BTreeMap<String, String>,
    transports: BTreeMap<String, (String, String)>,
    hospitals: BTreeMap<String, String>,
    places: BTreeMap<String, String>,
    objects: BTreeMap<String, String>,
    /// Event ID → time, for the entries corrections refer to.
    times: BTreeMap<String, u64>,
}

fn decode<T: DeserializeOwned>(e: &AppliedEvent) -> Option<T> {
    codec::from_cbor(&e.payload).ok()
}

impl Context {
    fn from_log(log: &[AppliedEvent]) -> Self {
        use ordo_med_patients as med;
        let mut c = Self::default();
        // Oldest first, so later names win.
        for e in log.iter().rev() {
            c.times.insert(e.body.id.to_string(), e.body.hlc.wall_ms);
            match e.body.event_type.as_str() {
                med::UHS_CREATED => {
                    if let Some(p) = decode::<med::UhsCreated>(e) {
                        c.uhs.insert(p.id, p.name);
                    }
                }
                med::UHS_UPDATED => {
                    if let Some(med::UhsUpdated {
                        id,
                        name: Some(name),
                        ..
                    }) = decode(e)
                    {
                        c.uhs.insert(id, name);
                    }
                }
                med::UHS_PLACES => {
                    if let Some(p) = decode::<med::PlacesSet>(e) {
                        c.places
                            .extend(p.places.into_iter().map(|p| (p.id, p.label)));
                    }
                }
                ordo_command::POSITION_CREATED => {
                    if let Some(p) = decode::<ordo_command::PositionCreated>(e) {
                        let title =
                            format!("{} {}", crate::command::STRUCTURE.label(&p.kind), p.name);
                        c.positions.insert(p.position, title);
                    }
                }
                ordo_tasks::GIVEN => {
                    if let Some(p) = decode::<ordo_tasks::TaskGiven>(e) {
                        c.tasks.insert(p.task, p.text);
                    }
                }
                med::orders::GIVEN => {
                    if let Some(p) = decode::<med::orders::OrderGiven>(e) {
                        c.orders.insert(p.order, p.tag);
                    }
                }
                med::orders::RECORDED => {
                    if let Some(p) = decode::<med::orders::OrderRecorded>(e) {
                        c.orders.insert(p.order, p.tag);
                    }
                }
                med::transport::REQUESTED => {
                    if let Some(p) = decode::<med::transport::TransportRequested>(e) {
                        c.transports.insert(p.request, (p.transport_type, p.uhs));
                    }
                }
                med::hospitals::SET => {
                    if let Some(p) = decode::<med::hospitals::HospitalSet>(e) {
                        c.hospitals.insert(p.hospital, p.name);
                    }
                }
                ordo_map::PLACED => {
                    if let Some(p) = decode::<ordo_map::ObjectPlaced>(e) {
                        c.objects.insert(p.object, p.label);
                    }
                }
                _ => {}
            }
        }
        c
    }

    fn name<'a>(map: &'a BTreeMap<String, String>, id: &'a str) -> &'a str {
        map.get(id).map_or(id, String::as_str)
    }

    fn uhs<'a>(&'a self, id: &'a str) -> &'a str {
        Self::name(&self.uhs, id)
    }

    fn position<'a>(&'a self, id: &'a str) -> &'a str {
        Self::name(&self.positions, id)
    }

    fn transport(&self, id: &str) -> String {
        self.transports.get(id).map_or_else(
            || id.to_owned(),
            |(kind, uhs)| text::log_transport(kind, self.uhs(uhs)),
        )
    }
}

/// One entry in words, and the entry it corrects (LOG-3), if any.
struct Line {
    text: String,
    corrects: Option<String>,
}

impl From<String> for Line {
    fn from(text: String) -> Self {
        Self {
            text,
            corrects: None,
        }
    }
}

// [impl->dsn~incident-log~1]
/// An event in words. An event type this app does not know, or cannot read, is shown
/// with its module and type: the log leaves nothing out (LOG-1).
fn describe(e: &AppliedEvent, c: &Context, name: &dyn Fn(&str) -> String) -> Line {
    use ordo_med_patients as med;
    use ordo_sync::incidents as inc;
    let unknown = || text::log_unknown(e.body.module.as_str(), e.body.event_type.as_str());
    let line: Option<Line> = (|| {
        Some(match e.body.event_type.as_str() {
            inc::PLANNED => text::log_planned(&decode::<inc::Planned>(e)?.name).into(),
            inc::STATE_CHANGED => {
                text::incident_state_changed(decode::<inc::StateChanged>(e)?.state).into()
            }
            inc::DETAILS_CHANGED => {
                decode::<inc::Details>(e)?;
                text::INCIDENT_DETAILS_SAVED.to_owned().into()
            }
            inc::PAUSE_CHANGED => text::incident_paused(decode::<inc::PauseChange>(e)?.paused)
                .to_owned()
                .into(),
            inc::ROLE_GRANTED => {
                let p = decode::<inc::RoleChange>(e)?;
                text::role_granted(&name(p.user.as_str()), &p.role).into()
            }
            inc::ROLE_WITHDRAWN => {
                let p = decode::<inc::RoleChange>(e)?;
                text::role_withdrawn(&name(p.user.as_str()), &p.role).into()
            }
            ordo_units::ADDED => {
                let p = decode::<ordo_units::UnitAdded>(e)?;
                text::log_unit_added(&p.unit, &p.unit_type, &p.organisation).into()
            }
            ordo_units::STATUS_CHANGED => {
                let p = decode::<ordo_units::StatusChanged>(e)?;
                text::status_set(&p.unit, &p.status).into()
            }
            ordo_units::RELEASED => {
                text::unit_released(&decode::<ordo_units::Released>(e)?.unit).into()
            }
            ordo_units::CREW_SET => {
                let p = decode::<ordo_units::CrewSet>(e)?;
                text::log_crew(&p.unit, p.members.len()).into()
            }
            ordo_tasks::GIVEN => {
                let p = decode::<ordo_tasks::TaskGiven>(e)?;
                let to = p
                    .unit
                    .as_deref()
                    .or(p.sector.as_deref().map(|s| c.position(s)))
                    .unwrap_or_default();
                text::log_task_given(to, &p.text).into()
            }
            ordo_tasks::UPDATED => {
                let p = decode::<ordo_tasks::TaskUpdated>(e)?;
                let task = Context::name(&c.tasks, &p.task);
                text::log_task_updated(task, p.status.map(text::task_status)).into()
            }
            ordo_command::POSITION_CREATED => {
                let p = decode::<ordo_command::PositionCreated>(e)?;
                text::position_created(c.position(&p.position)).into()
            }
            ordo_command::POSITION_CLOSED => {
                let p = decode::<ordo_command::PositionClosed>(e)?;
                text::log_position_closed(c.position(&p.position)).into()
            }
            ordo_command::LEADER_SET => {
                let p = decode::<ordo_command::LeaderSet>(e)?;
                text::log_leader(c.position(&p.position), &name(&p.leader)).into()
            }
            ordo_command::ASSIGNED => {
                let p = decode::<ordo_command::Assigned>(e)?;
                let what = p
                    .unit
                    .as_deref()
                    .or(p.uhs.as_deref().map(|u| c.uhs(u)))
                    .unwrap_or_default();
                text::log_assigned(what, p.position.as_deref().map(|p| c.position(p))).into()
            }
            ordo_map::PLACED => {
                text::log_map_placed(&decode::<ordo_map::ObjectPlaced>(e)?.label).into()
            }
            ordo_map::REMOVED => {
                let p = decode::<ordo_map::ObjectRemoved>(e)?;
                text::log_map_removed(Context::name(&c.objects, &p.object)).into()
            }
            med::UHS_CREATED => text::uhs_created(&decode::<med::UhsCreated>(e)?.name).into(),
            med::UHS_UPDATED => {
                let p = decode::<med::UhsUpdated>(e)?;
                match p.name {
                    Some(name) => text::uhs_renamed(&name).into(),
                    None => text::uhs_updated(c.uhs(&p.id)).into(),
                }
            }
            med::UHS_PLACES => {
                let p = decode::<med::PlacesSet>(e)?;
                text::log_places(c.uhs(&p.uhs), p.places.len()).into()
            }
            med::REGISTERED => {
                let p = decode::<med::Registered>(e)?;
                text::registered(&p.tag, p.category, c.uhs(&p.uhs)).into()
            }
            med::RETRIAGED => {
                let p = decode::<med::Retriaged>(e)?;
                text::retriaged(&p.tag, p.category).into()
            }
            med::MOVED => {
                let p = decode::<med::Moved>(e)?;
                text::patient_moved(&p.tag, c.uhs(&p.uhs)).into()
            }
            med::DISCHARGED => {
                let p = decode::<med::Discharged>(e)?;
                text::discharged(&p.tag, p.outcome).into()
            }
            med::READMITTED => text::readmitted(&decode::<med::Readmitted>(e)?.tag).into(),
            med::PLACE_ASSIGNED => {
                let p = decode::<med::PlaceAssigned>(e)?;
                let place = p.place.as_deref().map(|id| Context::name(&c.places, id));
                text::place_assigned(&p.tag, place).into()
            }
            med::TRANSPORT_PRIORITY => {
                let p = decode::<med::TransportPrioritySet>(e)?;
                text::priority_changed(&p.tag, p.level.is_some()).into()
            }
            med::transport::LINKED => {
                text::transport_requested(&decode::<med::transport::TransportLinked>(e)?.tag).into()
            }
            med::transport::REQUESTED => {
                let p = decode::<med::transport::TransportRequested>(e)?;
                text::log_transport_requested(&c.transport(&p.request)).into()
            }
            med::transport::UPDATED => {
                let p = decode::<med::transport::TransportUpdated>(e)?;
                text::log_transport_updated(
                    &c.transport(&p.request),
                    text::transport_status(p.status),
                    p.unit.as_deref(),
                )
                .into()
            }
            med::orders::GIVEN => {
                text::order_given(&decode::<med::orders::OrderGiven>(e)?.tag).into()
            }
            med::orders::RECORDED => {
                text::order_recorded(&decode::<med::orders::OrderRecorded>(e)?.tag).into()
            }
            med::orders::CONFIRMED => {
                let p = decode::<med::orders::OrderConfirmed>(e)?;
                text::log_order_confirmed(Context::name(&c.orders, &p.order)).into()
            }
            med::orders::UPDATED => {
                let p = decode::<med::orders::OrderUpdated>(e)?;
                let tag = Context::name(&c.orders, &p.order);
                let status = match &p.performed_by {
                    Some(by) => text::order_status_performed(text::order_status(p.status), by),
                    None => text::order_status(p.status).to_owned(),
                };
                text::log_order_updated(tag, &status).into()
            }
            med::record::ENTRY_RECORDED => {
                let p = decode::<med::record::EntryRecorded>(e)?;
                let form = crate::patients::FORMS
                    .form(&p.form)
                    .map_or(p.form.as_str(), |f| f.label.as_str());
                Line {
                    text: text::entry_saved(form, &p.tag),
                    corrects: p.replaces,
                }
            }
            med::record::PHOTO_ATTACHED => {
                text::photo_added(&decode::<med::record::PhotoAttached>(e)?.tag).into()
            }
            med::hospitals::SET => {
                text::hospital_saved(&decode::<med::hospitals::HospitalSet>(e)?.name).into()
            }
            med::hospitals::CAPACITY_SET => {
                let p = decode::<med::hospitals::CapacitySet>(e)?;
                text::capacity_saved(Context::name(&c.hospitals, &p.hospital)).into()
            }
            _ => return None,
        })
    })();
    line.unwrap_or_else(|| unknown().into())
}

/// The log of the open incident, as this device knows it.
fn log(core: Handle) -> Vec<AppliedEvent> {
    core.with_value(|c| {
        c.incident
            .as_ref()
            .map(|i| c.replica.log(i))
            .unwrap_or_default()
    })
}

/// The view *Einsatztagebuch*: the entries grouped by day, newest first.
#[component]
pub fn LogView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let days = Memo::new(move |_| {
        revision.track();
        let log = log(core);
        let context = Context::from_log(&log);
        let mut days: Vec<(String, Vec<(AppliedEvent, Line)>)> = Vec::new();
        for e in log {
            let day = core::date(e.body.hlc.wall_ms);
            let line = describe(&e, &context, &|text: &str| {
                crate::profile::display_text(core, text)
            });
            let line = Line {
                corrects: line.corrects.filter(|id| context.times.contains_key(id)),
                ..line
            };
            match days.last_mut() {
                Some((d, entries)) if *d == day => entries.push((e, line)),
                _ => days.push((day, vec![(e, line)])),
            }
        }
        // Times of corrected entries, looked up once.
        let times = context.times;
        days.into_iter()
            .map(|(day, entries)| {
                let entries: Vec<Entry> = entries
                    .into_iter()
                    .map(|(e, line)| Entry {
                        id: e.body.id.to_string(),
                        at: e.body.hlc.wall_ms,
                        text: line.text,
                        corrects: line
                            .corrects
                            .and_then(|id| times.get(&id).map(|at| (id, *at))),
                        by: crate::profile::display_name(core, &e.body.author.user),
                        device: core::short_device(&e.body.author.device),
                        event_type: e.body.event_type.as_str().to_owned(),
                        sent: e.seq.is_some(),
                    })
                    .collect();
                (day, entries)
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section class="card incident-log" aria-labelledby="log-title">
            <h2 id="log-title">{text::LOG_TITLE}</h2>
            <p class="hint">{text::LOG_HINT}</p>
            {move || {
                let days = days.get();
                if days.is_empty() {
                    return view! { <p>{text::LOG_EMPTY}</p> }.into_any();
                }
                days.into_iter()
                    .map(|(day, entries)| {
                        let heading = format!("log-day-{}", day.replace('.', "-"));
                        view! {
                            <h3 id=heading.clone()>{day}</h3>
                            <ol class="log-entries" aria-labelledby=heading>
                                {entries.into_iter().map(|entry| view! { <LogEntry entry /> }).collect_view()}
                            </ol>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </section>
    }
}

/// One entry, ready to show.
#[derive(Clone, PartialEq)]
struct Entry {
    id: String,
    at: u64,
    text: String,
    corrects: Option<(String, u64)>,
    by: String,
    device: String,
    event_type: String,
    sent: bool,
}

#[component]
fn LogEntry(entry: Entry) -> impl IntoView {
    view! {
        <li class="log-entry" id=format!("log-{}", entry.id) data-type=entry.event_type>
            <time class="log-entry__time" datetime=core::iso_time(entry.at)>{core::clock_seconds(entry.at)}</time>
            <span class="log-entry__text">
                {entry.text}
                {entry.corrects.map(|(id, at)| view! {
                    " " <a class="log-entry__corrects" href=format!("#log-{id}")>{text::log_corrects(&core::clock_seconds(at))}</a>
                })}
            </span>
            <span class="log-entry__by">
                {text::log_by(&entry.by, &entry.device)}
                {(!entry.sent).then(|| view! { " · " <span class="log-entry__pending">{text::LOG_PENDING}</span> })}
            </span>
        </li>
    }
}
