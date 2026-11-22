//! The patient record at a glance (EPCR-14, EPCR-15, ADR-0088): the header, the quick
//! actions, the overview's tiles and the timeline. Which forms get a
//! tile or a quick action, which fields show in the header and their short labels come
//! from the profile; this module knows only the marks. It shows documented values with
//! their time and never rates them (EPCR-9).

use leptos::prelude::*;
use ordo_med_patients::orders::OrderBook;
use ordo_med_patients::record::{self, Field, FieldType, Form, FormKind, Overview, Value};
use ordo_med_patients::transport::TransportStatus;
use ordo_med_patients::{self as med, Holder, Patient, PatientRegister, Stay, StayEntry};

use crate::core::{self, Handle};
use crate::med::patients::FORMS;
use crate::med::text;
use crate::med::triage::TriageBadge;
use crate::shell::ui::Ago;
use ordo_med_patients::record::Records;

/// The tabs of the record page.
pub(crate) const OVERVIEW: &str = "overview";
pub(crate) const TIMELINE: &str = "timeline";
pub(crate) const DOCUMENTATION: &str = "documentation";
pub(crate) const HANDOVER: &str = "handover";

/// The record's tabs, in order, with a short label for phones where needed.
pub(crate) fn tabs() -> Vec<crate::shell::ui::Tab> {
    vec![
        (OVERVIEW, text::RECORD_OVERVIEW.to_owned(), None),
        (TIMELINE, text::RECORD_TIMELINE.to_owned(), None),
        (
            DOCUMENTATION,
            text::RECORD_DOCUMENTATION.to_owned(),
            Some(text::RECORD_DOCUMENTATION_SHORT.to_owned()),
        ),
        (HANDOVER, text::RECORD_HANDOVER.to_owned(), None),
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

// [impl->dsn~record-overview~3]
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

// [impl->dsn~unique-triage-tag~2]
/// For whoever registered a card later: the new card's number, and "Nummer ändern".
#[component]
fn Retag(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    event: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let number = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let id = format!("retag-{event}");
    let change = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let new_tag = number.get_untracked().trim().to_owned();
        if new_tag.is_empty() {
            return;
        }
        if register.with_untracked(|r| r.patient_with_card(&new_tag).is_some()) {
            error.set(Some(text::retag_in_use(&new_tag)));
            return;
        }
        let change = med::retag::Retagged {
            tag: tag.clone(),
            registration: event.clone(),
            new_tag,
        };
        match core::retag(core, revision, &change) {
            Ok(()) => error.set(None),
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <form class="form--inline retag" on:submit=change>
            <p class="retag__ask">{text::RETAG_ASK}</p>
            <div class="field">
                <label for=id.clone()>{text::RETAG_NUMBER}</label>
                <input id=id.clone() type="text" inputmode="numeric" autocomplete="off" required class="input--code" bind:value=number />
            </div>
            <button type="submit" class="button button--primary">{text::RETAG_SUBMIT}</button>
            <p role="alert" class="error">{move || error.get()}</p>
        </form>
    }
}

// [impl->dsn~record-overview~3]
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
            // [impl->dsn~unique-triage-tag~2]
            // [impl->dsn~unique-triage-tag~2]
            // Each later registration, and whether this person changes its number: who
            // made it, or the people of its holder (ADR-0092).
            let me = core.with_value(|c| c.replica.user().clone());
            let again: Vec<(String, Option<String>)> = patient
                .registered_again
                .iter()
                .map(|r| {
                    let mine = r.by == me || crate::med::patients::may_write(core, &r.holder);
                    (
                        text::tag_registered_again(
                            &crate::shell::profile::display_name(core, &r.by),
                            &core::holder_name(core, &r.holder),
                            &core::clock(r.at.wall_ms),
                        ),
                        mine.then(|| r.event.clone()),
                    )
                })
                .collect();
            let triaged = patient.history.last().map(|h| h.at.wall_ms);
            let since = patient.stay.last().map(|s| s.at.wall_ms);
            let discharged = match patient.discharge().map(|d| &d.change) {
                Some(Stay::Discharged { outcome, .. }) => Some(text::discharged_as(*outcome)),
                Some(Stay::HandedOutside { receiver, .. }) => {
                    Some(text::handed_outside_step(receiver))
                }
                _ => None,
            };
            Some((
                patient.category(),
                core::holder_name(core, &patient.holder()),
                place,
                triaged,
                since,
                transport,
                discharged,
                again,
            ))
        })
    };
    let title = text::patient_title(&tag);
    view! {
        {move || state.get().map(|(category, uhs, place, triaged, since, transport, discharged, again)| view! {
            <div class="record__id">
                <h2 id="record-title" tabindex="-1" class="record__title">{title.clone()}</h2>
                <TriageBadge category />
                {discharged.map(|d| view! { <strong class="record__discharged">{d}</strong> })}
            </div>
            {(!again.is_empty()).then(|| view! {
                <div class="notice notice--warning record__twice" role="note">
                    <p><strong><crate::shell::ui::Icon name="triangle-alert" />{text::TAG_TWICE}</strong></p>
                    {again.into_iter().map(|(line, event)| view! {
                        <p>{line}</p>
                        {event.map(|event| view! { <Retag core revision tag=tag.clone() event register /> })}
                    }).collect_view()}
                    <p class="hint">{text::TAG_TWICE_HINT}</p>
                </div>
            })}
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
                    <div class="record__when"><dt>{text::RECORD_TRIAGED}</dt><dd>{core::clock(at)} " " <Ago at /></dd></div>
                })}
                {since.map(|at| view! {
                    <div class="record__when"><dt>{text::RECORD_SINCE}</dt><dd>{core::clock(at)}</dd></div>
                })}
                {transport.map(|t| view! {
                    <div><dt>{text::TRANSPORT_TITLE}</dt><dd>{t}</dd></div>
                })}
            </dl>
        })}
    }
}

// [impl->dsn~record-overview~3]
/// The quick actions: one per form the profile marks `quick`, then triage, photo and the
/// handover. Each opens its input in *Dokumentation*, or the tab *Übergabe* (the tab shows
/// first). In the page on wide screens; on phones a floating button *Erfassen* above the
/// bottom bar opens the forms and the handover in a sheet, within thumb reach, and triage
/// and photo stay in the page.
#[component]
pub(crate) fn QuickActions(chosen: RwSignal<&'static str>) -> impl IntoView {
    let actions = move |bar: bool| {
        let forms = FORMS.forms.iter().filter(|f| f.shows(Overview::Quick)).map(move |form| {
            let (target, label) = match form.kind {
                FormKind::Series => (format!("new-{}", form.id), text::new_entry_label(&form.label)),
                FormKind::Snapshot => (format!("edit-{}", form.id), text::edit_label(&form.label)),
            };
            view! {
                <button type="button" class="button button--small button--with-icon quick-actions__button quick-actions__button--bar"
                    aria-label=label on:click=move |_| crate::shell::views::click(&target)>
                    <crate::shell::ui::Icon name="plus" /><span>{form.label.clone()}</span>
                </button>
            }
        });
        let handover = move |_| {
            chosen.set(HANDOVER);
            crate::shell::views::after_render(|| crate::shell::views::focus("section-way"));
        };
        view! {
            {forms.collect_view()}
            {(!bar).then(|| view! {
                <button type="button" class="button button--small button--with-icon quick-actions__button"
                    on:click=move |_| crate::shell::views::open_inputs("section-triage")>
                    <crate::shell::ui::Icon name="pencil" />{text::TRIAGE_OPEN}
                </button>
                <button type="button" class="button button--small button--with-icon quick-actions__button"
                    on:click=move |_| crate::shell::views::focus("section-photo")>
                    <crate::shell::ui::Icon name="camera" />{text::PHOTO_TITLE}
                </button>
            })}
            <button type="button" class="button button--small button--with-icon quick-actions__button quick-actions__button--bar quick-actions__handover"
                on:click=handover>
                <crate::shell::ui::Icon name="arrow-right-left" /><span>{text::HANDOVER_BUTTON}</span>
            </button>
        }
    };
    // The floating button sits above the bottom bar, whose height depends on the words and
    // the text size: measured once it is there, and again on resizing.
    request_animation_frame(measure_bottom_bar);
    let resized = window_event_listener(leptos::ev::resize, move |_| measure_bottom_bar());
    on_cleanup(move || resized.remove());
    let open = RwSignal::new(false);
    let close = move || {
        open.set(false);
        crate::shell::views::focus("quick-sheet-button");
    };
    view! {
        <nav class="quick-actions quick-actions--page" aria-label=text::QUICK_ACTIONS>
            {actions(false)}
        </nav>
        // [impl->dsn~record-overview~3]
        // On phones: the bottom bar stays as on every page; the forms and the handover open
        // from one floating button (feedback of 2026-11-22).
        <div class="quick-sheet" on:keydown=move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" && open.get_untracked() {
                ev.prevent_default();
                close();
            }
        }>
            <nav id="quick-sheet-panel" class="quick-actions quick-actions--sheet" aria-label=text::QUICK_ACTIONS
                hidden=move || !open.get()
                // Choosing an action closes the sheet; the action moves the focus itself.
                on:click=move |ev: leptos::ev::MouseEvent| {
                    use wasm_bindgen::JsCast;
                    let on_button = ev
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .is_some_and(|t| t.closest("button").ok().flatten().is_some());
                    if on_button {
                        open.set(false);
                    }
                }>
                {actions(true)}
            </nav>
            <crate::shell::ui::DisclosureButton id="quick-sheet-button" controls="quick-sheet-panel" open
                class="button quick-sheet__button">
                <crate::shell::ui::Icon name="plus" /><span>{text::QUICK_CAPTURE}</span>
            </crate::shell::ui::DisclosureButton>
        </div>
    }
}

/// Puts the height of the phone's bottom bar into `--bottom-bar-height` on the root, so the
/// floating button sits right above it and the page keeps that much space free below and
/// as scroll padding (WCAG 2.4.11). Set through the style object, which the content
/// security policy allows.
fn measure_bottom_bar() {
    use wasm_bindgen::JsCast;
    request_animation_frame(|| {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let height = document
            .query_selector(".navigation")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
            .map_or(0, |e| e.offset_height());
        if let Some(root) = document
            .document_element()
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = root
                .style()
                .set_property("--bottom-bar-height", &format!("{height}px"));
        }
    });
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

// [impl->dsn~record-overview~3]
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
    // The person's tile (the first with fields in the header) shows the photo too.
    let person = FORMS
        .forms
        .iter()
        .filter(|f| f.shows(Overview::Tile))
        .find(|f| f.fields.iter().any(|x| x.header))
        .map(|f| f.id.as_str());
    let tiles = FORMS
        .forms
        .iter()
        .filter(|f| f.shows(Overview::Tile))
        .map(|form| {
            let photo = person == Some(form.id.as_str());
            view! { <FormTile tag=tag.clone() form records chosen filter photo /> }
        })
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
    let way_tag = tag.clone();
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
                <crate::med::flow::PatientFlow core revision tag=way_tag register />
            </section>
        </div>
    }
}

/// The holder the patient was registered with: the one the first change of holder left,
/// else the current one.
pub(crate) fn first_holder(patient: &Patient) -> Holder {
    patient
        .stay
        .iter()
        .find_map(|s| match &s.change {
            Stay::Moved { from, .. } => Some(Holder::Uhs(from.clone())),
            Stay::Taken { from, .. } => Some(from.clone()),
            _ => None,
        })
        .unwrap_or_else(|| patient.holder())
}

// [impl->dsn~patient-handover~1]
/// The `i`-th entry of a patient's stay in words, for the way, the timeline and the stay's
/// history: e.g. "Erfasst · SanTrp 1", "Übergabe an UHS Eingang", "Übernommen: UHS
/// Eingang".
pub(crate) fn stay_words(core: Handle, patient: &Patient, i: usize, entry: &StayEntry) -> String {
    let name = |h: &Holder| core::holder_name(core, h);
    match &entry.change {
        Stay::Admitted if i == 0 => text::way_registered(&name(&first_holder(patient))),
        Stay::Admitted => text::READMITTED_STEP.to_owned(),
        Stay::Moved { from, to } => {
            text::moved(&core::uhs_name(core, from), &core::uhs_name(core, to))
        }
        Stay::Discharged { outcome, .. } => text::discharged_as(*outcome),
        Stay::HandedOver { to: Some(to), .. } => text::handover_to(&name(to)),
        Stay::HandedOver { to: None, .. } => text::HANDOVER_QUICK_STEP.to_owned(),
        Stay::Taken { from, to } if from == to => text::handover_taken_back(&name(to)),
        Stay::Taken { to, .. } => text::handover_taken_by(&name(to)),
        Stay::HandedOutside { receiver, .. } => text::handed_outside_step(receiver),
    }
}

// [impl->dsn~record-overview~3]
/// The tile of one form: its latest entry (series) or its current values (snapshot), with
/// the time and how long ago.
#[component]
fn FormTile(
    tag: String,
    form: &'static Form,
    records: Memo<Records>,
    chosen: RwSignal<&'static str>,
    filter: RwSignal<String>,
    /// Whether it shows the patient's photo (the person's tile), if there is one.
    photo: bool,
) -> impl IntoView {
    let heading = format!("tile-{}", form.id);
    // [impl->dsn~record-overview~3]
    // The photo's preview, inside its event: shown at once, offline too.
    let image = {
        let tag = tag.clone();
        Memo::new(move |_| {
            use base64::Engine;
            photo.then(|| {
                records.with(|r| {
                    r.get(&tag).and_then(|r| r.photo()).map(|p| {
                        format!(
                            "data:{};base64,{}",
                            p.media_type,
                            base64::engine::general_purpose::STANDARD.encode(&p.data)
                        )
                    })
                })
            })?
        })
    };
    let alt = text::photo_alt(&tag);
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
            {move || image.get().map(|src| view! {
                <img class="record-tile__photo" src=src alt=alt.clone() />
            })}
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
                by: name(&p.by),
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
        for (i, s) in patient.stay.iter().enumerate() {
            let what = stay_words(core, patient, i, s);
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
