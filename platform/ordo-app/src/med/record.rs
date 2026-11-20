//! The patient record at a glance (EPCR-14, EPCR-15, ADR-0088): the header, the quick
//! actions, the overview's tiles and the timeline. Which forms get a
//! tile or a quick action, which fields show in the header and their short labels come
//! from the profile; this module knows only the marks. It shows documented values with
//! their time and never rates them (EPCR-9).

use leptos::prelude::*;
use ordo_med_patients::orders::OrderBook;
use ordo_med_patients::record::{self, Field, FieldType, Form, FormKind, Overview, Value};
use ordo_med_patients::transport::TransportStatus;
use ordo_med_patients::{self as med, PatientRegister, Stay};

use crate::core::{self, Handle};
use crate::med::patients::FORMS;
use crate::med::triage::TriageBadge;
use crate::shell::ui::Ago;
use crate::text;
use ordo_med_patients::record::Records;

/// The tabs of the record page.
pub(crate) const OVERVIEW: &str = "overview";
pub(crate) const TIMELINE: &str = "timeline";
pub(crate) const DOCUMENTATION: &str = "documentation";
pub(crate) const HANDOVER: &str = "handover";

/// The record's tabs, in order.
pub(crate) fn tabs() -> Vec<(&'static str, String)> {
    vec![
        (OVERVIEW, text::RECORD_OVERVIEW.to_owned()),
        (TIMELINE, text::RECORD_TIMELINE.to_owned()),
        (DOCUMENTATION, text::RECORD_DOCUMENTATION.to_owned()),
        (HANDOVER, text::RECORD_HANDOVER.to_owned()),
    ]
}

/// A field's label where room is scarce: its short label, else its label.
fn short(field: &Field) -> String {
    field.short.clone().unwrap_or_else(|| field.label.clone())
}

/// When an entry was documented: the time of day its time field gives (on or before the
/// day it was saved), else when it was saved.
fn documented_at(form: &Form, entry: &record::Entry) -> u64 {
    form.fields
        .iter()
        .find(|f| f.field_type == FieldType::Time)
        .and_then(|f| match entry.values.get(&f.id) {
            Some(Value::One(t)) if !t.is_empty() => core::today_at(t, entry.at.wall_ms),
            _ => None,
        })
        .unwrap_or(entry.at.wall_ms)
}

/// The documented values of an entry, with short labels and the profile's sums; the
/// time field is shown apart.
fn entry_facts(
    form: &Form,
    values: &std::collections::BTreeMap<String, Value>,
) -> Vec<(String, String)> {
    let mut facts: Vec<(String, String)> = form
        .fields
        .iter()
        .filter(|f| f.field_type != FieldType::Time)
        .filter_map(|f| {
            let value = values.get(&f.id).filter(|v| !v.is_empty())?;
            let shown = f.display(value);
            Some((short(f), shown))
        })
        .collect();
    facts.extend(form.sums(values).into_iter().map(|(sum, total)| {
        (
            sum.short.clone().unwrap_or_else(|| sum.label.clone()),
            total,
        )
    }));
    facts
}

// [impl->dsn~record-overview~1]
/// The values of the fields the profile puts in the header (e.g. name, allergies).
fn header_facts(record: Option<&record::Record>) -> Vec<(String, String)> {
    let Some(record) = record else {
        return Vec::new();
    };
    FORMS
        .forms
        .iter()
        .flat_map(|form| {
            form.fields
                .iter()
                .filter(|f| f.header)
                .filter_map(move |f| {
                    let value = match form.kind {
                        FormKind::Snapshot => {
                            record.current(&form.id, &f.id).map(|s| s.value.clone())
                        }
                        FormKind::Series => record
                            .current_entries(&form.id)
                            .filter_map(|e| e.values.get(&f.id).filter(|v| !v.is_empty()).cloned())
                            .last(),
                    }?;
                    Some((short(f), f.display(&value)))
                })
        })
        .collect()
}

// [impl->dsn~record-overview~1]
/// The header of the record: tag, triage category, the profile's header fields, where
/// the patient is and since when, the transport, and whether it is discharged.
#[component]
pub(crate) fn RecordHeader(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
) -> impl IntoView {
    let facts = {
        let tag = tag.clone();
        Memo::new(move |_| records.with(|r| header_facts(r.get(&tag))))
    };
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            let patient = register.with(|r| r.patient(&tag).cloned())?;
            let place = patient.place.as_ref().and_then(|id| {
                core.with_value(|c| {
                    med::places(&c.events(med::UHS_COMPARTMENT))
                        .remove(&patient.uhs)
                        .and_then(|list| list.into_iter().find(|p| &p.id == id))
                        .map(|p| p.label)
                })
            });
            let transport = crate::med::transports::patient_transports(core, &tag)
                .into_iter()
                .rev()
                .find(|t| t.status != TransportStatus::Cancelled)
                .map(|t| {
                    format!(
                        "{} · {}",
                        text::transport_status(t.status),
                        crate::med::transports::transport_line(&t)
                    )
                });
            let triaged = patient.history.last().map(|h| h.at.wall_ms);
            let since = patient.stay.last().map(|s| s.at.wall_ms);
            let discharged = match patient.discharge().map(|d| &d.change) {
                Some(Stay::Discharged { outcome, .. }) => Some(text::discharged_as(*outcome)),
                _ => None,
            };
            Some((
                patient.category(),
                core::uhs_name(core, &patient.uhs),
                place,
                triaged,
                since,
                transport,
                discharged,
            ))
        })
    };
    view! {
        <h2 id="record-title" tabindex="-1" class="record__title">{text::patient_title(&tag)}</h2>
        {move || state.get().map(|(category, uhs, place, triaged, since, transport, discharged)| view! {
            <div class="record__badges">
                <TriageBadge category />
                {discharged.map(|d| view! { <strong class="record__discharged">{d}</strong> })}
            </div>
            <dl class="record__facts">
                {facts
                    .get()
                    .into_iter()
                    .map(|(label, value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> })
                    .collect_view()}
            </dl>
            <dl class="record__where">
                <div>
                    <dt>{text::RECORD_AT}</dt>
                    <dd>{uhs}{place.map(|p| view! { " · " {p} })}</dd>
                </div>
                {triaged.map(|at| view! {
                    <div><dt>{text::RECORD_TRIAGED}</dt><dd>{core::clock(at)} " " <Ago at /></dd></div>
                })}
                {since.map(|at| view! {
                    <div><dt>{text::RECORD_SINCE}</dt><dd>{core::clock(at)}</dd></div>
                })}
                {transport.map(|t| view! {
                    <div><dt>{text::TRANSPORT_TITLE}</dt><dd>{t}</dd></div>
                })}
            </dl>
        })}
    }
}

// [impl->dsn~record-overview~1]
/// The quick actions: one per form the profile marks `quick`, then triage and photo.
/// Each opens its input in *Dokumentation* (the tab shows first).
#[component]
pub(crate) fn QuickActions() -> impl IntoView {
    let forms = FORMS.forms.iter().filter(|f| f.shows(Overview::Quick)).map(|form| {
        let (target, label) = match form.kind {
            FormKind::Series => (format!("new-{}", form.id), text::new_entry_label(&form.label)),
            FormKind::Snapshot => (format!("edit-{}", form.id), text::edit_label(&form.label)),
        };
        view! {
            <button type="button" class="button button--small button--with-icon quick-actions__button"
                aria-label=label on:click=move |_| crate::shell::views::click(&target)>
                <crate::shell::ui::Icon name="plus" />{form.label.clone()}
            </button>
        }
    });
    view! {
        <nav class="quick-actions" aria-label=text::QUICK_ACTIONS>
            {forms.collect_view()}
            <button type="button" class="button button--small button--with-icon quick-actions__button"
                on:click=move |_| crate::shell::views::open_inputs("section-triage")>
                <crate::shell::ui::Icon name="pencil" />{text::TRIAGE_OPEN}
            </button>
            <button type="button" class="button button--small button--with-icon quick-actions__button"
                on:click=move |_| crate::shell::views::focus("section-photo")>
                <crate::shell::ui::Icon name="camera" />{text::PHOTO_TITLE}
            </button>
        </nav>
    }
}

/// A button that shows a tab, e.g. "Ganzer Verlauf".
#[component]
fn TabLink(
    chosen: RwSignal<&'static str>,
    to: &'static str,
    #[prop(into)] label: String,
    #[prop(optional, into)] accessible: Option<String>,
    #[prop(optional)] filter: Option<(RwSignal<String>, String)>,
) -> impl IntoView {
    view! {
        <button type="button" class="button button--small button--quiet" aria-label=accessible
            on:click=move |_| {
                if let Some((signal, value)) = &filter {
                    signal.set(value.clone());
                }
                chosen.set(to);
                crate::shell::views::after_render(move || {
                    crate::shell::views::focus(&format!("record-tab-{to}"));
                });
            }>
            {label}
        </button>
    }
}

// [impl->dsn~record-overview~1]
/// The overview: a tile per form the profile marks `tile`, the open orders and the
/// patient's way.
#[component]
pub(crate) fn OverviewTiles(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
    chosen: RwSignal<&'static str>,
    filter: RwSignal<String>,
) -> impl IntoView {
    let tiles = FORMS
        .forms
        .iter()
        .filter(|f| f.shows(Overview::Tile))
        .map(|form| view! { <FormTile tag=tag.clone() form records chosen filter /> })
        .collect_view();
    let orders = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            core.with_value(|c| {
                OrderBook::from_events(&c.events(med::orders::COMPARTMENT))
                    .of_patient(&tag)
                    .into_iter()
                    .filter(|o| o.status.is_open())
                    .map(|o| {
                        let status = if o.unconfirmed() {
                            text::ORDER_UNCONFIRMED.to_owned()
                        } else {
                            text::order_status(o.status).to_owned()
                        };
                        (crate::med::orders::ordered_line(&o.given), status)
                    })
                    .collect::<Vec<_>>()
            })
        })
    };
    let way = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            way_steps(core, &tag, register)
        })
    };
    view! {
        <div class="record-tiles">
            {tiles}
            <section class="record-tile" aria-labelledby="tile-orders">
                <div class="record-tile__head">
                    <h3 id="tile-orders">{text::ORDERS_OPEN_TITLE}</h3>
                    <TabLink chosen to=DOCUMENTATION label=text::ALL accessible=text::ORDERS_ALL />
                </div>
                {move || {
                    let list = orders.get();
                    if list.is_empty() {
                        return view! { <p class="hint">{text::ORDERS_NONE_OPEN}</p> }.into_any();
                    }
                    view! {
                        <ul class="record-tile__list">
                            {list.into_iter().map(|(what, status)| view! {
                                <li><strong>{what}</strong> " · " <span class="hint">{status}</span></li>
                            }).collect_view()}
                        </ul>
                    }
                    .into_any()
                }}
            </section>
            <section class="record-tile record-tile--wide" aria-labelledby="tile-way">
                <div class="record-tile__head">
                    <h3 id="tile-way">{text::STAY_TITLE}</h3>
                    <TabLink chosen to=HANDOVER label=text::RECORD_HANDOVER />
                </div>
                <ol class="way">
                    {move || way.get().into_iter().map(|(time, what, now)| view! {
                        <li class="way__step" class:way__step--now=now>
                            <span class="hint">{time}</span> " " {what}
                            {now.then(|| view! { " · " <strong>{text::NOW}</strong> })}
                        </li>
                    }).collect_view()}
                </ol>
            </section>
        </div>
    }
}

/// The patient's way: registration, moves, discharge and transports, oldest first; the
/// current step marked.
fn way_steps(
    core: Handle,
    tag: &str,
    register: Memo<PatientRegister>,
) -> Vec<(String, String, bool)> {
    let Some(patient) = register.with_untracked(|r| r.patient(tag).cloned()) else {
        return Vec::new();
    };
    let mut steps: Vec<(u64, String)> = Vec::new();
    // The UHS of the registration: the one the first move left, else the current one.
    let first = patient
        .stay
        .iter()
        .find_map(|s| match &s.change {
            Stay::Moved { from, .. } => Some(from.clone()),
            _ => None,
        })
        .unwrap_or_else(|| patient.uhs.clone());
    for (i, entry) in patient.stay.iter().enumerate() {
        let what = match &entry.change {
            Stay::Admitted if i == 0 => text::way_registered(&core::uhs_name(core, &first)),
            Stay::Admitted => text::READMITTED_STEP.to_owned(),
            Stay::Moved { to, .. } => core::uhs_name(core, to),
            Stay::Discharged { outcome, .. } => text::discharged_as(*outcome),
        };
        steps.push((entry.at.wall_ms, what));
    }
    let present = patient.is_present();
    let mut out: Vec<(String, String, bool)> = steps
        .iter()
        .enumerate()
        .map(|(i, (at, what))| {
            (
                core::clock(*at),
                what.clone(),
                present && i + 1 == steps.len(),
            )
        })
        .collect();
    for t in crate::med::transports::patient_transports(core, tag)
        .into_iter()
        .filter(|t| t.status != TransportStatus::Cancelled)
    {
        out.push((
            core::clock(t.requested_at().wall_ms),
            format!(
                "{} · {}",
                crate::med::transports::transport_line(&t),
                text::transport_status(t.status)
            ),
            false,
        ));
    }
    out
}

// [impl->dsn~record-overview~1]
/// The tile of one form: its latest entry (series) or its current values (snapshot), with
/// the time and how long ago.
#[component]
fn FormTile(
    tag: String,
    form: &'static Form,
    records: Memo<Records>,
    chosen: RwSignal<&'static str>,
    filter: RwSignal<String>,
) -> impl IntoView {
    let heading = format!("tile-{}", form.id);
    let labelled = heading.clone();
    let latest = {
        let tag = tag.clone();
        Memo::new(move |_| {
            records.with(|r| {
                let record = r.get(&tag)?;
                match form.kind {
                    FormKind::Series => {
                        let count = record.current_entries(&form.id).count();
                        let entry = record.current_entries(&form.id).last()?;
                        Some((
                            documented_at(form, entry),
                            entry_facts(form, &entry.values),
                            count,
                        ))
                    }
                    FormKind::Snapshot => {
                        let mut at = 0;
                        let facts: Vec<(String, String)> = form
                            .fields
                            .iter()
                            .filter_map(|f| {
                                let s = record.current(&form.id, &f.id)?;
                                at = at.max(s.at.wall_ms);
                                Some((short(f), f.display(&s.value)))
                            })
                            .collect();
                        (!facts.is_empty()).then_some((at, facts, 0))
                    }
                }
            })
        })
    };
    let numbers = form
        .fields
        .iter()
        .filter(|f| f.field_type == FieldType::Number)
        .count()
        * 2
        > form.fields.len();
    view! {
        <section class="record-tile" class:record-tile--wide=numbers aria-labelledby=labelled>
            <div class="record-tile__head">
                <h3 id=heading>
                    {form.label.clone()}
                    {move || latest.get().map(|(at, ..)| view! {
                        <span class="record-tile__when">" · " {text::LATEST} " " {core::clock(at)} " " <Ago at /></span>
                    })}
                </h3>
                {(form.kind == FormKind::Series).then(|| {
                    let all = move || latest.get().map_or(0, |(.., n)| n);
                    view! {
                        <Show when=move || { all() > 1 }>
                            <TabLink chosen to=TIMELINE label=text::entries_all(all())
                                accessible=text::entries_all_label(&form.label, all())
                                filter=(filter, form.id.clone()) />
                        </Show>
                    }
                })}
            </div>
            {move || match latest.get() {
                None => view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any(),
                Some((_, facts, _)) => view! {
                    <dl class="record-tile__values" class:record-tile__values--numbers=numbers>
                        {facts.into_iter().map(|(label, value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> }).collect_view()}
                    </dl>
                }
                .into_any(),
            }}
        </section>
    }
}

/// One line of the timeline.
#[derive(Clone, PartialEq)]
pub(crate) struct Line {
    pub(crate) at: u64,
    /// Filter key: a form ID, or one of the other kinds.
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) text: String,
    pub(crate) by: String,
    pub(crate) corrected: bool,
    /// The time of the entry this one corrects.
    pub(crate) corrects: Option<u64>,
    /// When it was saved: orders entries of the same documented minute.
    pub(crate) saved: u64,
}

const TRIAGE: &str = "triage";
const WAY: &str = "way";
const ORDERS: &str = "orders";
const TRANSPORT: &str = "transport";
const PHOTOS: &str = "photos";

// [impl->dsn~record-timeline~1]
/// Every entry of the record in one list, newest first.
pub(crate) fn timeline(
    core: Handle,
    tag: &str,
    register: &PatientRegister,
    records: &Records,
) -> Vec<Line> {
    let name = |by: &ordo_core::UserId| crate::shell::profile::display_name(core, by);
    let mut lines = Vec::new();
    if let Some(record) = records.get(tag) {
        for form in &FORMS.forms {
            match form.kind {
                FormKind::Series => {
                    let entries = record.entries.get(&form.id).cloned().unwrap_or_default();
                    for e in &entries {
                        let corrects = e.replaces.as_ref().and_then(|id| {
                            entries
                                .iter()
                                .find(|o| &o.id == id)
                                .map(|o| documented_at(form, o))
                        });
                        let facts = entry_facts(form, &e.values);
                        lines.push(Line {
                            at: documented_at(form, e),
                            kind: form.id.clone(),
                            label: form.label.clone(),
                            text: facts
                                .iter()
                                .map(|(l, v)| format!("{l} {v}"))
                                .collect::<Vec<_>>()
                                .join(" · "),
                            by: name(&e.by),
                            corrected: e.replaced_by.is_some(),
                            corrects,
                            saved: e.at.wall_ms,
                        });
                    }
                }
                FormKind::Snapshot => {
                    // Changes made by one saving, together.
                    let mut saved: std::collections::BTreeMap<(u64, String), Vec<String>> =
                        Default::default();
                    for f in &form.fields {
                        for s in record
                            .fields
                            .get(&(form.id.clone(), f.id.clone()))
                            .into_iter()
                            .flatten()
                        {
                            let shown = if s.value.is_empty() {
                                text::CLEARED.to_owned()
                            } else {
                                f.display(&s.value)
                            };
                            saved
                                .entry((s.at.wall_ms, s.by.to_string()))
                                .or_default()
                                .push(format!("{} {shown}", short(f)));
                        }
                    }
                    for ((at, by), parts) in saved {
                        lines.push(Line {
                            at,
                            kind: form.id.clone(),
                            label: form.label.clone(),
                            text: parts.join(" · "),
                            by: crate::shell::profile::display_text(core, &by),
                            corrected: false,
                            corrects: None,
                            saved: 0,
                        });
                    }
                }
            }
        }
        for p in &record.photos {
            lines.push(Line {
                at: p.at.wall_ms,
                kind: PHOTOS.to_owned(),
                label: text::PHOTO_TITLE.to_owned(),
                text: text::PHOTO_ADDED.to_owned(),
                by: String::new(),
                corrected: false,
                corrects: None,
                saved: 0,
            });
        }
    }
    if let Some(patient) = register.patient(tag) {
        for (i, h) in patient.history.iter().enumerate() {
            let mut what = text::label(h.category).to_owned();
            if let Some(a) = &h.algorithm {
                let name = crate::med::patients::TRIAGE
                    .get(a)
                    .map_or(a.clone(), |x| x.name.clone());
                what = format!("{what} {}", text::triage_with(&name));
            }
            if i == 0 {
                what = text::registered_as(&what);
            }
            lines.push(Line {
                at: h.at.wall_ms,
                kind: TRIAGE.to_owned(),
                label: text::TRIAGE_TITLE.to_owned(),
                text: what,
                by: name(&h.by),
                corrected: false,
                corrects: None,
                saved: 0,
            });
        }
        for s in &patient.stay {
            let what = match &s.change {
                Stay::Admitted => format!(
                    "{} · {}",
                    text::ADMITTED,
                    core::uhs_name(core, &patient.uhs)
                ),
                Stay::Moved { from, to } => {
                    text::moved(&core::uhs_name(core, from), &core::uhs_name(core, to))
                }
                Stay::Discharged { outcome, .. } => text::discharged_as(*outcome),
            };
            lines.push(Line {
                at: s.at.wall_ms,
                kind: WAY.to_owned(),
                label: text::STAY_TITLE.to_owned(),
                text: what,
                by: name(&s.by),
                corrected: false,
                corrects: None,
                saved: 0,
            });
        }
    }
    core.with_value(|c| {
        for o in OrderBook::from_events(&c.events(med::orders::COMPARTMENT)).of_patient(tag) {
            for (i, step) in o.history.iter().enumerate() {
                let what = crate::med::orders::ordered_line(&o.given);
                let text = if i == 0 {
                    what
                } else {
                    format!("{what} · {}", text::order_status(step.status))
                };
                lines.push(Line {
                    at: step.at.wall_ms,
                    kind: ORDERS.to_owned(),
                    label: text::ORDER.to_owned(),
                    text,
                    by: crate::shell::profile::display_name(core, &step.by),
                    corrected: false,
                    corrects: None,
                    saved: 0,
                });
            }
        }
    });
    for t in crate::med::transports::patient_transports(core, tag) {
        let what = crate::med::transports::transport_line(&t);
        for step in &t.history {
            lines.push(Line {
                at: step.at.wall_ms,
                kind: TRANSPORT.to_owned(),
                label: text::TRANSPORT_TITLE.to_owned(),
                text: format!("{what} · {}", text::transport_status(step.status)),
                by: name(&step.by),
                corrected: false,
                corrects: None,
                saved: 0,
            });
        }
    }
    lines.sort_by_key(|l| std::cmp::Reverse((l.at, l.saved)));
    lines
}

// [impl->dsn~record-timeline~1]
/// *Verlauf*: the timeline with its filter chips.
#[component]
pub(crate) fn Timeline(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
    filter: RwSignal<String>,
) -> impl IntoView {
    let lines = Memo::new(move |_| {
        revision.track();
        register.with(|r| records.with(|rec| timeline(core, &tag, r, rec)))
    });
    // The kinds this record has, in the order of the page.
    let kinds = Memo::new(move |_| {
        let mut kinds: Vec<(String, String)> = Vec::new();
        lines.with(|lines| {
            let order = |k: &str| {
                FORMS
                    .forms
                    .iter()
                    .position(|f| f.id == k)
                    .unwrap_or(usize::MAX)
            };
            for l in lines {
                if !kinds.iter().any(|(k, _)| *k == l.kind) {
                    kinds.push((l.kind.clone(), l.label.clone()));
                }
            }
            kinds.sort_by_key(|(k, _)| order(k));
        });
        kinds
    });
    let chip = move |key: String, label: String| {
        let on = {
            let key = key.clone();
            move || filter.get() == key
        };
        view! {
            <button type="button" class="chip" aria-pressed=move || on().to_string()
                on:click=move |_| filter.set(key.clone())>
                {label}
            </button>
        }
    };
    view! {
        <div class="timeline">
            <div class="timeline__filters" role="group" aria-label=text::TIMELINE_FILTER>
                {chip(String::new(), text::ALL_KINDS.to_owned())}
                {move || kinds.get().into_iter().map(|(k, l)| chip(k, l)).collect_view()}
            </div>
            {move || {
                let shown: Vec<Line> = lines.with(|lines| {
                    let f = filter.get();
                    lines.iter().filter(|l| f.is_empty() || l.kind == f).cloned().collect()
                });
                if shown.is_empty() {
                    return view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any();
                }
                view! {
                    <ol class="timeline__list">
                        {shown.into_iter().map(|l| view! {
                            <li class="timeline__line" class:timeline__line--corrected=l.corrected>
                                <time class="timeline__time" datetime=core::iso_time(l.at)>{core::clock(l.at)}</time>
                                <span class="timeline__kind">{l.label}</span>
                                <span class="timeline__text">
                                    {l.text}
                                    {l.corrected.then(|| view! { " · " <em>{text::CORRECTED}</em> })}
                                    {l.corrects.map(|at| view! { " · " <span class="hint">{text::corrects_entry(&core::clock(at))}</span> })}
                                </span>
                                <span class="timeline__by hint">{l.by}</span>
                            </li>
                        }).collect_view()}
                    </ol>
                }
                .into_any()
            }}
        </div>
    }
}
