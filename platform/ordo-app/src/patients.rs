//! Med workspace: patient registration, the patient list and the patient care record.
//!
//! Record editing happens in place, on the record page (master–detail on wide screens,
//! a page of its own on phones), never in a modal dialog (ADR-0029). Unsaved input is
//! kept per patient and form while the app runs, so switching views loses nothing.

use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use leptos::prelude::*;
use leptos::task::spawn_local;
use ordo_contracts::TriageCategory;
use ordo_med_patients::record::{
    self, EntryRecorded, Field, FieldType, Form, FormCatalogue, FormKind, PhotoAdded, Records,
    Value,
};
use ordo_med_patients::{self as med, PatientRegister};
use wasm_bindgen::JsCast;

use crate::core::{self, Handle};
use crate::nav::Route;
use crate::text;
use crate::views::{CATEGORIES, Nav, TriageBadge, after_render, focus, uhs_list};

const WORKSPACE: &str = "med.uhs";
const VIEW: &str = "patients";

/// The ePCR forms of the profile, checked at build time (see `build.rs`).
pub static FORMS: LazyLock<FormCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/epcr-forms.json")))
        .expect("ePCR forms were checked at build time")
});

/// Route to the record of the patient with triage tag `tag`.
pub fn record_route(tag: &str) -> Route {
    Route::record(WORKSPACE, VIEW, tag)
}

/// How an editor was opened.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// Edit a snapshot form.
    Edit,
    /// New entry of a series form.
    New,
    /// Correction of a series entry.
    Correct(String),
}

/// Unsaved input per patient and form, kept while the app runs (UX-5). Clinical input is
/// not written to browser storage unencrypted.
#[derive(Clone, Copy)]
pub struct Drafts(StoredValue<HashMap<String, Draft>, LocalStorage>);

/// An open editor and its values.
type Draft = (Mode, BTreeMap<String, Value>);

impl Default for Drafts {
    fn default() -> Self {
        Self(StoredValue::new_local(HashMap::new()))
    }
}

fn draft_key(tag: &str, form: &str) -> String {
    format!("{tag}/{form}")
}

/// The patients view: registration and list, and the record of the chosen patient.
#[component]
pub fn PatientsView(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    record: Signal<Option<String>>,
) -> impl IntoView {
    let events = move || core.with_value(|c| c.events(med::COMPARTMENT));
    let register = Memo::new(move |_| {
        revision.track();
        PatientRegister::from_events(&events())
    });
    let records = Memo::new(move |_| {
        revision.track();
        Records::from_events(&events())
    });
    view! {
        <div class="master-detail" data-open=move || record.get().is_some().to_string()>
            <div class="master">
                <RegisterForm core revision nav />
                <PatientList core revision nav register records selected=record />
            </div>
            <div class="detail">
                {move || match record.get() {
                    Some(tag) => view! { <PatientRecord core revision nav tag register records /> }.into_any(),
                    None => view! { <section class="card"><p>{text::CHOOSE_PATIENT}</p></section> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn RegisterForm(core: Handle, revision: RwSignal<u64>, nav: Nav) -> impl IntoView {
    let tag = RwSignal::new(String::new());
    // Remembered between registrations: a device usually works at one UHS.
    let uhs = RwSignal::new(String::new());
    // No preselection: a forgotten choice must not become a wrong triage category.
    let category = RwSignal::new(None::<TriageCategory>);
    let error = RwSignal::new(None::<String>);
    let last = RwSignal::new(None::<String>);
    let list = Memo::new(move |_| {
        revision.track();
        uhs_list(core)
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let value = tag.get().trim().to_owned();
        let Some(chosen) = category.get() else { return };
        if value.is_empty() || uhs.get().is_empty() {
            return;
        }
        match core::register(core, revision, &value, chosen, &uhs.get()) {
            Ok(()) => {
                tag.set(String::new());
                category.set(None);
                error.set(None);
                last.set(Some(value));
                focus("tag");
            }
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <section aria-labelledby="register-title" class="card">
            <h2 id="register-title">{text::REGISTER_TITLE}</h2>
            {move || {
                if list.with(Vec::is_empty) {
                    return view! { <p class="notice">{text::NO_UHS}</p> }.into_any();
                }
                view! {
                    <form on:submit=submit class="form">
                        <label for="tag">{text::REGISTER_TAG}</label>
                        <input id="tag" type="text" inputmode="numeric" autocomplete="off" required bind:value=tag class="input--code" />
                        <fieldset class="choices">
                            <legend>{text::REGISTER_CATEGORY}</legend>
                            {CATEGORIES
                                .into_iter()
                                .map(|c| {
                                    view! {
                                        <label class="choice">
                                            <input
                                                type="radio"
                                                name="category"
                                                required
                                                prop:checked=move || category.get() == Some(c)
                                                on:change=move |_| category.set(Some(c))
                                            />
                                            <TriageBadge category=c />
                                        </label>
                                    }
                                })
                                .collect_view()}
                        </fieldset>
                        <label for="uhs">{text::REGISTER_UHS}</label>
                        <select id="uhs" required on:change=move |ev| uhs.set(event_target_value(&ev)) prop:value=move || uhs.get()>
                            <option value="">{text::UHS_CHOOSE}</option>
                            {list.get().into_iter().map(|u| view! { <option value=u.id.clone()>{u.name}</option> }).collect_view()}
                        </select>
                        <p role="alert" class="error">{move || error.get()}</p>
                        <button type="submit" class="button button--primary">{text::REGISTER_SUBMIT}</button>
                    </form>
                }
                .into_any()
            }}
            {move || {
                last.get().map(|tag| {
                    let route = record_route(&tag);
                    view! {
                        <p class="notice">
                            {text::patient_title(&tag)} " " {text::REGISTERED} " "
                            <a href=route.to_path() on:click=move |ev| {
                                ev.prevent_default();
                                nav.go(route.clone());
                                after_render(|| focus("record-title"));
                            }>{text::ACTION_OPEN_RECORD}</a>
                        </p>
                    }
                })
            }}
        </section>
    }
}

/// Display name from the identity form, if documented.
fn person_name(records: &Records, tag: &str) -> Option<String> {
    let record = records.get(tag)?;
    let value = |field: &str| match record.current("identity", field).map(|s| &s.value) {
        Some(Value::One(text)) => Some(text.clone()),
        _ => None,
    };
    match (value("family_name"), value("given_name")) {
        (Some(family), Some(given)) => Some(format!("{family}, {given}")),
        (Some(name), None) | (None, Some(name)) => Some(name),
        (None, None) => None,
    }
}

#[component]
fn PatientList(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
    selected: Signal<Option<String>>,
) -> impl IntoView {
    let cards = move || {
        revision.track();
        let names: HashMap<String, String> =
            uhs_list(core).into_iter().map(|u| (u.id, u.name)).collect();
        // Patients with changes that are still in the outbox.
        let pending: Vec<String> = core.with_value(|c| {
            c.events(med::COMPARTMENT)
                .iter()
                .filter(|e| e.seq.is_none())
                .filter_map(tag_of)
                .collect()
        });
        register.with(|r| {
            records.with(|records| {
                r.patients()
                    .map(|p| {
                        (
                            p.tag.clone(),
                            p.category(),
                            names.get(&p.uhs).cloned().unwrap_or_else(|| p.uhs.clone()),
                            p.history.len(),
                            person_name(records, &p.tag),
                            pending.contains(&p.tag),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        })
    };
    view! {
        <section aria-labelledby="board-title" class="card">
            <h2 id="board-title">{text::PATIENTS_TITLE}</h2>
            {move || {
                let cards = cards();
                if cards.is_empty() {
                    return view! { <p>{text::BOARD_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="patient-list">
                        {cards
                            .into_iter()
                            .map(|(tag, category, uhs, history, name, pending)| {
                                let route = record_route(&tag);
                                let current = {
                                    let tag = tag.clone();
                                    move || (selected.get().as_deref() == Some(tag.as_str())).then_some("page")
                                };
                                view! {
                                    <li>
                                        <a
                                            class="patient-card"
                                            data-tag=tag.clone()
                                            href=route.to_path()
                                            aria-current=current
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                nav.go(route.clone());
                                                after_render(|| focus("record-title"));
                                            }
                                        >
                                            <span class="patient-card__head">
                                                <span class="patient-card__tag code">{tag.clone()}</span>
                                                <TriageBadge category />
                                            </span>
                                            <span class="patient-card__name">{name.unwrap_or_else(|| text::UNKNOWN_PERSON.to_owned())}</span>
                                            <span class="patient-card__meta">
                                                {uhs} " · " {text::triage_count(history)} " · "
                                                <span class="patient-card__state">{if pending { text::NOT_SENT } else { text::SENT }}</span>
                                            </span>
                                        </a>
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

/// The triage tag number of a patient event (all patient event types carry `tag`).
fn tag_of(event: &ordo_sync::AppliedEvent) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Tag {
        tag: String,
    }
    ordo_core::codec::from_cbor::<Tag>(&event.payload)
        .ok()
        .map(|t| t.tag)
}

// [impl->dsn~patient-care-record~1]
/// The care record of one patient: photo, triage, and the forms of the profile.
#[component]
fn PatientRecord(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    tag: String,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
) -> impl IntoView {
    let exists = {
        let tag = tag.clone();
        move || register.with(|r| r.patient(&tag).is_some())
    };
    let back = Route::view(WORKSPACE, VIEW);
    let back_link = view! {
        <a class="back-link" href=back.to_path() on:click=move |ev| {
            ev.prevent_default();
            nav.go(back.clone());
            after_render(|| focus("board-title"));
        }>
            <span aria-hidden="true">"←"</span>{text::ALL_PATIENTS}
        </a>
    };
    if !exists() {
        return view! {
            <section class="card record">
                {back_link}
                <p class="notice">{text::NOT_FOUND}</p>
            </section>
        }
        .into_any();
    }
    let sections: Vec<(String, String)> = [
        ("section-photo".to_owned(), text::PHOTO_TITLE.to_owned()),
        ("section-triage".to_owned(), text::TRIAGE_TITLE.to_owned()),
    ]
    .into_iter()
    .chain(
        FORMS
            .forms
            .iter()
            .map(|f| (format!("section-{}", f.id), f.label.clone())),
    )
    .collect();
    view! {
        <article class="card record" aria-labelledby="record-title">
            {back_link}
            <h2 id="record-title" tabindex="-1">{text::patient_title(&tag)}</h2>
            <nav class="record__index" aria-label=text::SECTIONS>
                {sections
                    .into_iter()
                    .map(|(id, label)| {
                        view! {
                            <button type="button" class="chip" on:click=move |_| focus(&id)>{label}</button>
                        }
                    })
                    .collect_view()}
            </nav>
            <PhotoSection core revision tag=tag.clone() records />
            <TriageSection core revision tag=tag.clone() register />
            {FORMS
                .forms
                .iter()
                .map(|form| view! { <FormSection core revision tag=tag.clone() form records /> })
                .collect_view()}
        </article>
    }
    .into_any()
}

// [impl->dsn~patient-photo~1]
#[component]
fn PhotoSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    records: Memo<Records>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<&'static str>);
    // The data URL is rebuilt only when another photo becomes the latest.
    let source = {
        let tag = tag.clone();
        Memo::new(move |previous: Option<&Option<(String, String)>>| {
            let latest = records.with(|r| {
                r.get(&tag)
                    .and_then(|r| r.photo())
                    .map(|p| (p.id.clone(), p.media_type.clone(), p.data.clone()))
            });
            match (previous, latest) {
                (Some(Some(prev)), Some((id, _, _))) if prev.0 == id => Some(prev.clone()),
                (_, Some((id, media_type, data))) => Some((
                    id,
                    format!("data:{media_type};base64,{}", BASE64.encode(data)),
                )),
                (_, None) => None,
            }
        })
    };
    let on_file = {
        let tag = tag.clone();
        move |ev: leptos::ev::Event| {
            let input: web_sys::HtmlInputElement = event_target(&ev);
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return;
            };
            input.set_value("");
            busy.set(true);
            error.set(None);
            let tag = tag.clone();
            spawn_local(async move {
                match downscale(&file).await {
                    Ok(data) => {
                        let photo = PhotoAdded {
                            tag,
                            media_type: "image/jpeg".to_owned(),
                            data,
                        };
                        if core::add_photo(core, revision, &photo).is_err() {
                            error.set(Some(text::PHOTO_TOO_LARGE));
                        }
                    }
                    Err(message) => error.set(Some(message)),
                }
                busy.set(false);
            });
        }
    };
    let alt = text::photo_alt(&tag);
    view! {
        <section class="record__section" aria-labelledby="section-photo">
            <h3 id="section-photo" tabindex="-1">{text::PHOTO_TITLE}</h3>
            {move || match source.get() {
                Some((_, src)) => view! { <img class="photo" src=src alt=alt.clone() /> }.into_any(),
                None => view! { <p>{text::PHOTO_NONE}</p> }.into_any(),
            }}
            <p class="hint" id="photo-purpose">{text::PHOTO_PURPOSE}</p>
            <div class="row">
                <input id="photo-camera" class="file-input visually-hidden" type="file" accept="image/*" capture="environment"
                    aria-describedby="photo-purpose" on:change=on_file.clone() />
                <label for="photo-camera" class="button">{text::PHOTO_TAKE}</label>
                <input id="photo-upload" class="file-input visually-hidden" type="file" accept="image/*"
                    aria-describedby="photo-purpose" on:change=on_file />
                <label for="photo-upload" class="button">{text::PHOTO_UPLOAD}</label>
            </div>
            <p role="status" class="hint">{move || busy.get().then_some(text::PHOTO_BUSY)}</p>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}

/// Downscales a photo on the device and re-encodes it as JPEG: small enough to travel in
/// a sealed event, and without camera or location metadata (ADR-0028).
async fn downscale(file: &web_sys::File) -> Result<Vec<u8>, &'static str> {
    let window = web_sys::window().ok_or(text::PHOTO_FAILED)?;
    let promise = window
        .create_image_bitmap_with_blob(file)
        .map_err(|_| text::PHOTO_FAILED)?;
    let bitmap: web_sys::ImageBitmap = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| text::PHOTO_FAILED)?
        .dyn_into()
        .map_err(|_| text::PHOTO_FAILED)?;
    let (width, height) = (f64::from(bitmap.width()), f64::from(bitmap.height()));
    let document = window.document().ok_or(text::PHOTO_FAILED)?;
    let result = (|| {
        for (edge, quality) in [(800.0, 0.75), (640.0, 0.7), (480.0, 0.6), (360.0, 0.5)] {
            let scale = (edge / width.max(height)).min(1.0);
            let (w, h) = ((width * scale).round(), (height * scale).round());
            let canvas: web_sys::HtmlCanvasElement =
                document.create_element("canvas").ok()?.dyn_into().ok()?;
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
            let context: web_sys::CanvasRenderingContext2d =
                canvas.get_context("2d").ok()??.dyn_into().ok()?;
            context
                .draw_image_with_image_bitmap_and_dw_and_dh(&bitmap, 0.0, 0.0, w, h)
                .ok()?;
            let url = canvas
                .to_data_url_with_type_and_encoder_options(
                    "image/jpeg",
                    &wasm_bindgen::JsValue::from_f64(quality),
                )
                .ok()?;
            let bytes = BASE64.decode(url.split_once(',')?.1).ok()?;
            if bytes.len() <= record::MAX_PHOTO_BYTES {
                return Some(Ok(bytes));
            }
        }
        Some(Err(text::PHOTO_TOO_LARGE))
    })();
    bitmap.close();
    result.unwrap_or(Err(text::PHOTO_FAILED))
}

#[component]
fn TriageSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        move || {
            register.with(|r| {
                r.patient(&tag).map(|p| {
                    (
                        p.category(),
                        p.history
                            .iter()
                            .rev()
                            .map(|h| (h.category, core::clock(h.at.wall_ms), h.by.to_string()))
                            .collect::<Vec<_>>(),
                    )
                })
            })
        }
    };
    let current = {
        let state = state.clone();
        move || state().map(|(c, _)| c)
    };
    view! {
        <section class="record__section" aria-labelledby="section-triage">
            <h3 id="section-triage" tabindex="-1">{text::TRIAGE_TITLE}</h3>
            {
                let current = current.clone();
                move || current().map(|category| view! { <p><TriageBadge category /></p> })
            }
            <fieldset class="choices">
                <legend>{text::RETRIAGE_TO}</legend>
                {CATEGORIES
                    .into_iter()
                    .map(|c| {
                        let tag = tag.clone();
                        let current = current.clone();
                        view! {
                            <button
                                type="button"
                                class="button button--small"
                                aria-label=text::retriage_label(&tag, c)
                                hidden=move || current() == Some(c)
                                on:click=move |_| {
                                    let _ = core::retriage(core, revision, &tag, c);
                                }
                            >
                                {text::label(c)}
                            </button>
                        }
                    })
                    .collect_view()}
            </fieldset>
            <h4>{text::TRIAGE_HISTORY}</h4>
            <ol class="history">
                {move || {
                    state()
                        .map(|(_, history)| {
                            history
                                .into_iter()
                                .map(|(category, time, by)| {
                                    view! { <li><TriageBadge category /> " " {text::by_at(&by, &time)}</li> }
                                })
                                .collect_view()
                        })
                }}
            </ol>
        </section>
    }
}

/// One form of the record: its documented values and, on demand, an editor in place.
#[component]
fn FormSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    form: &'static Form,
    records: Memo<Records>,
) -> impl IntoView {
    let drafts = use_context::<Drafts>().unwrap_or_default();
    let key = draft_key(&tag, &form.id);
    let editing = RwSignal::new(drafts.0.with_value(|d| d.get(&key).map(|(m, _)| m.clone())));
    let heading = format!("section-{}", form.id);
    let opener = match form.kind {
        FormKind::Snapshot => format!("edit-{}", form.id),
        FormKind::Series => format!("new-{}", form.id),
    };
    let content = {
        let tag = tag.clone();
        move || {
            records.with(|records| {
                let record = records.get(&tag);
                match form.kind {
                    FormKind::Snapshot => {
                        let facts: Vec<(String, String)> = form
                            .fields
                            .iter()
                            .filter_map(|f| {
                                let value = record?.current(&form.id, &f.id)?;
                                Some((f.label.clone(), f.display(&value.value)))
                            })
                            .collect();
                        if facts.is_empty() {
                            return view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any();
                        }
                        view! { <Facts facts /> }.into_any()
                    }
                    FormKind::Series => {
                        let entries: Vec<record::Entry> = record
                            .map(|r| r.entries.get(&form.id).cloned().unwrap_or_default())
                            .unwrap_or_default();
                        if entries.is_empty() {
                            return view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any();
                        }
                        let corrected = entries.iter().filter(|e| e.replaced_by.is_some()).count();
                        view! {
                            <ol class="entries" reversed>
                                {entries
                                    .iter()
                                    .rev()
                                    .filter(|e| e.replaced_by.is_none())
                                    .map(|e| view! { <EntryView form entry=e.clone() editing /> })
                                    .collect_view()}
                            </ol>
                            {(corrected > 0).then(|| view! {
                                <details class="corrected">
                                    <summary>{text::corrected_entries(corrected)}</summary>
                                    <ol class="entries">
                                        {entries
                                            .iter()
                                            .filter(|e| e.replaced_by.is_some())
                                            .map(|e| view! { <EntryView form entry=e.clone() editing=RwSignal::new(None) /> })
                                            .collect_view()}
                                    </ol>
                                </details>
                            })}
                        }
                        .into_any()
                    }
                }
            })
        }
    };
    let open_label = match form.kind {
        FormKind::Snapshot => text::edit_label(&form.label),
        FormKind::Series => text::new_entry_label(&form.label),
    };
    let open_text = match form.kind {
        FormKind::Snapshot => text::EDIT,
        FormKind::Series => text::NEW_ENTRY,
    };
    let opener_id = opener.clone();
    view! {
        <section class="record__section" aria-labelledby=heading.clone()>
            <div class="record__section-head">
                <h3 id=heading.clone() tabindex="-1">{form.label.clone()}</h3>
                <button
                    id=opener.clone()
                    type="button"
                    class="button button--small"
                    aria-label=open_label
                    hidden=move || editing.get().is_some()
                    on:click={
                        let heading = heading.clone();
                        move |_| {
                            editing.set(Some(match form.kind {
                                FormKind::Snapshot => Mode::Edit,
                                FormKind::Series => Mode::New,
                            }));
                            let heading = heading.clone();
                            after_render(move || reveal_editor(&heading));
                        }
                    }
                >
                    {open_text}
                </button>
            </div>
            {move || {
                editing.get().map(|mode| {
                    let opener_id = opener_id.clone();
                    view! {
                        <EntryEditor core revision tag=tag.clone() form mode records
                            on_close=Callback::new(move |_| {
                                editing.set(None);
                                focus(&opener_id);
                            }) />
                    }
                })
            }}
            {content}
        </section>
    }
}

/// Brings an opened editor into view and focuses its first field, also when it was
/// opened from the command palette (UX-10).
fn reveal_editor(heading: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if let Some(section_head) = document.get_element_by_id(heading) {
        let options = web_sys::ScrollIntoViewOptions::new();
        options.set_block(web_sys::ScrollLogicalPosition::Start);
        section_head.scroll_into_view_with_scroll_into_view_options(&options);
    }
    let selector = format!(
        "[aria-labelledby='{heading}'] .editor :is(input, select, textarea):not([type=hidden])"
    );
    if let Some(field) = document
        .query_selector(&selector)
        .ok()
        .flatten()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = field.focus();
    }
}

#[component]
fn Facts(facts: Vec<(String, String)>) -> impl IntoView {
    view! {
        <dl class="facts">
            {facts
                .into_iter()
                .map(|(label, value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> })
                .collect_view()}
        </dl>
    }
}

#[component]
fn EntryView(
    form: &'static Form,
    entry: record::Entry,
    editing: RwSignal<Option<Mode>>,
) -> impl IntoView {
    let time = match entry.values.get("time") {
        Some(Value::One(t)) if !t.is_empty() => t.clone(),
        _ => core::clock(entry.at.wall_ms),
    };
    let mut facts: Vec<(String, String)> = form
        .fields
        .iter()
        .filter(|f| f.id != "time")
        .filter_map(|f| {
            let value = entry.values.get(&f.id).filter(|v| !v.is_empty())?;
            Some((f.label.clone(), f.display(value)))
        })
        .collect();
    facts.extend(
        form.sums(&entry.values)
            .into_iter()
            .map(|(sum, total)| (sum.label.clone(), total)),
    );
    let corrected = entry.replaced_by.is_some();
    let id = entry.id.clone();
    view! {
        <li class="entry" class:entry--corrected=corrected>
            <p class="entry__head">
                <strong>{time.clone()}</strong> " · " {entry.by.to_string()}
                {corrected.then(|| view! { " · " <em>{text::CORRECTED}</em> })}
            </p>
            <Facts facts />
            {(!corrected).then(|| view! {
                <button
                    type="button"
                    class="button button--small"
                    aria-label=format!("{} {} {}", form.label, time, text::CORRECT)
                    on:click=move |_| editing.set(Some(Mode::Correct(id.clone())))
                >
                    {text::CORRECT}
                </button>
            })}
        </li>
    }
}

/// Initial editor values: the draft, else the current values (edit), the corrected
/// entry (correction), or empty with the current time (new entry).
fn initial_values(
    form: &Form,
    mode: &Mode,
    record: Option<&record::Record>,
) -> BTreeMap<String, Value> {
    let empty = |f: &Field| match f.field_type {
        FieldType::Choices => Value::Many(Vec::new()),
        _ => Value::One(String::new()),
    };
    form.fields
        .iter()
        .map(|f| {
            let value = match mode {
                Mode::Edit => record
                    .and_then(|r| r.current(&form.id, &f.id))
                    .map(|s| s.value.clone()),
                Mode::Correct(id) => record
                    .and_then(|r| r.entries.get(&form.id))
                    .and_then(|list| list.iter().find(|e| &e.id == id))
                    .and_then(|e| e.values.get(&f.id).cloned()),
                Mode::New if f.field_type == FieldType::Time => Some(Value::One(core::now_clock())),
                Mode::New => None,
            };
            (f.id.clone(), value.unwrap_or_else(|| empty(f)))
        })
        .collect()
}

/// Editor for one form, in place on the record page.
#[component]
fn EntryEditor(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    form: &'static Form,
    mode: Mode,
    records: Memo<Records>,
    on_close: Callback<()>,
) -> impl IntoView {
    let drafts = use_context::<Drafts>().unwrap_or_default();
    let key = draft_key(&tag, &form.id);
    let start = drafts
        .0
        .with_value(|d| {
            d.get(&key)
                .filter(|(m, _)| *m == mode)
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_else(|| records.with_untracked(|r| initial_values(form, &mode, r.get(&tag))));
    let values: Vec<(&'static Field, RwSignal<Value>)> = form
        .fields
        .iter()
        .map(|f| {
            (
                f,
                RwSignal::new(
                    start
                        .get(&f.id)
                        .cloned()
                        .unwrap_or(Value::One(String::new())),
                ),
            )
        })
        .collect();
    let errors = RwSignal::new(BTreeMap::<String, String>::new());
    let message = RwSignal::new(None::<&'static str>);
    {
        // Keep unsaved input while the app runs.
        let values = values.clone();
        let (key, mode) = (key.clone(), mode.clone());
        Effect::new(move |_| {
            let snapshot: BTreeMap<String, Value> = values
                .iter()
                .map(|(f, v)| (f.id.clone(), v.get()))
                .collect();
            drafts.0.update_value(|d| {
                d.insert(key.clone(), (mode.clone(), snapshot));
            });
        });
    }
    let close = {
        let key = key.clone();
        move || {
            drafts.0.update_value(|d| {
                d.remove(&key);
            });
            on_close.run(());
        }
    };
    let submit = {
        let values = values.clone();
        let close = close.clone();
        let mode = mode.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let current =
                records.with_untracked(|r| initial_values(form, &Mode::Edit, r.get(&tag)));
            let chosen: BTreeMap<String, Value> = values
                .iter()
                .map(|(f, v)| (f.id.clone(), v.get_untracked()))
                .filter(|(id, v)| match mode {
                    // Snapshot: only what changed. Series: what was filled in.
                    Mode::Edit => {
                        let field = form.field(id).expect("own field");
                        field.normalise(v).ok() != field.normalise(&current[id]).ok()
                    }
                    Mode::New | Mode::Correct(_) => !v.is_empty(),
                })
                .collect();
            match form.normalise(&chosen) {
                Err(problems) => {
                    errors.set(
                        problems
                            .iter()
                            .map(|(id, e)| (id.clone(), text::value_error(e)))
                            .collect(),
                    );
                    message.set(Some(text::CHECK_INPUT));
                    if let Some((id, _)) = problems.first() {
                        focus(&format!("f-{}-{id}", form.id));
                    }
                }
                Ok(clean) if clean.is_empty() => {
                    errors.set(BTreeMap::new());
                    message.set(Some(match mode {
                        Mode::Edit => text::NO_CHANGES,
                        _ => text::EMPTY_ENTRY,
                    }));
                }
                Ok(clean) => {
                    let entry = EntryRecorded {
                        tag: tag.clone(),
                        form: form.id.clone(),
                        values: clean,
                        replaces: match &mode {
                            Mode::Correct(id) => Some(id.clone()),
                            _ => None,
                        },
                    };
                    match core::record_entry(
                        core,
                        revision,
                        &entry,
                        text::entry_saved(&form.label, &tag),
                    ) {
                        Ok(()) => close(),
                        Err(_) => message.set(Some(text::CHECK_INPUT)),
                    }
                }
            }
        }
    };
    let title_id = format!("editor-{}", form.id);
    let labelled_by = title_id.clone();
    view! {
        <form class="form editor" on:submit=submit aria-labelledby=labelled_by novalidate>
            <p id=title_id class="visually-hidden">{form.label.clone()}</p>
            {values
                .into_iter()
                .map(|(field, value)| {
                    let error = Signal::derive(move || errors.with(|e| e.get(&field.id).cloned()));
                    view! { <FieldInput form field value error /> }
                })
                .collect_view()}
            <p role="alert" class="error">{move || message.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">{text::SAVE}</button>
                <button type="button" class="button" on:click=move |_| close()>{text::CANCEL}</button>
            </div>
        </form>
    }
}

/// One field of an editor, by type. Labels, hints and errors are linked for assistive
/// technology; numbers keep the decimal keyboard on phones.
#[component]
fn FieldInput(
    form: &'static Form,
    field: &'static Field,
    value: RwSignal<Value>,
    error: Signal<Option<String>>,
) -> impl IntoView {
    let id = format!("f-{}-{}", form.id, field.id);
    let hint_id = format!("{id}-hint");
    let error_id = format!("{id}-error");
    let described = {
        let mut ids = Vec::new();
        if field.hint.is_some() || field.unit.is_some() {
            ids.push(hint_id.clone());
        }
        ids.push(error_id.clone());
        ids.join(" ")
    };
    let hint = match (&field.unit, &field.hint) {
        (Some(unit), Some(hint)) => Some(format!("{unit} · {hint}")),
        (Some(unit), None) => Some(unit.clone()),
        (None, hint) => hint.clone(),
    };
    let text_value = move || match value.get() {
        Value::One(t) => t,
        Value::Many(_) => String::new(),
    };
    let set_text = move |ev: leptos::ev::Event| value.set(Value::One(event_target_value(&ev)));
    let invalid = move || error.get().is_some().then_some("true");
    let input = match field.field_type {
        FieldType::Text | FieldType::Number | FieldType::Date | FieldType::Time => {
            let (input_type, mode) = match field.field_type {
                FieldType::Number => ("text", Some("decimal")),
                FieldType::Date => ("date", None),
                FieldType::Time => ("time", None),
                _ => ("text", None),
            };
            let now = (field.field_type == FieldType::Time).then(|| {
                view! {
                    <button type="button" class="button button--small" on:click=move |_| value.set(Value::One(core::now_clock()))>
                        {text::NOW}
                    </button>
                }
            });
            view! {
                <label for=id.clone()>{field.label.clone()}</label>
                <div class="row row--tight">
                    <input
                        id=id.clone()
                        type=input_type
                        inputmode=mode
                        autocomplete="off"
                        aria-describedby=described.clone()
                        aria-invalid=invalid
                        prop:value=text_value
                        on:input=set_text
                    />
                    {now}
                </div>
            }
            .into_any()
        }
        FieldType::Longtext => view! {
            <label for=id.clone()>{field.label.clone()}</label>
            <textarea
                id=id.clone()
                rows="3"
                aria-describedby=described.clone()
                aria-invalid=invalid
                prop:value=text_value
                on:input=set_text
            ></textarea>
        }
        .into_any(),
        FieldType::Choice => {
            let options = std::iter::once((String::new(), text::NO_ANSWER.to_owned()))
                .chain(field.options.iter().map(|o| (o.id.clone(), o.label.clone())));
            view! {
                <fieldset class="choices" id=id.clone() tabindex="-1" aria-describedby=described.clone()>
                    <legend>{field.label.clone()}</legend>
                    {options
                        .map(|(option, label)| {
                            let checked = {
                                let option = option.clone();
                                move || value.get() == Value::One(option.clone())
                            };
                            view! {
                                <label class="choice chip">
                                    <input
                                        type="radio"
                                        name=id.clone()
                                        prop:checked=checked
                                        on:change=move |_| value.set(Value::One(option.clone()))
                                    />
                                    {label}
                                </label>
                            }
                        })
                        .collect_view()}
                </fieldset>
            }
            .into_any()
        }
        FieldType::Choices => view! {
            <fieldset class="choices" id=id.clone() tabindex="-1" aria-describedby=described.clone()>
                <legend>{field.label.clone()}</legend>
                {field
                    .options
                    .iter()
                    .map(|o| {
                        let option = o.id.clone();
                        let checked = {
                            let option = option.clone();
                            move || matches!(value.get(), Value::Many(list) if list.contains(&option))
                        };
                        view! {
                            <label class="choice chip">
                                <input
                                    type="checkbox"
                                    prop:checked=checked
                                    on:change=move |ev| {
                                        let on = event_target_checked(&ev);
                                        value.update(|v| {
                                            let mut list = match v {
                                                Value::Many(list) => std::mem::take(list),
                                                Value::One(_) => Vec::new(),
                                            };
                                            list.retain(|x| x != &option);
                                            if on {
                                                list.push(option.clone());
                                            }
                                            *v = Value::Many(list);
                                        });
                                    }
                                />
                                {o.label.clone()}
                            </label>
                        }
                    })
                    .collect_view()}
            </fieldset>
        }
        .into_any(),
    };
    view! {
        <div class="field-block" class:field-block--invalid=move || error.get().is_some()>
            {input}
            {hint.map(|h| view! { <p id=hint_id class="hint">{h}</p> })}
            <p id=error_id class="field-error">{move || error.get()}</p>
        </div>
    }
}
