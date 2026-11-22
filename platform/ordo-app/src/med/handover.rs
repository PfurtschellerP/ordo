//! Handing a patient over (ADR-0087, EPCR-16, EPCR-17): the *Übergabe* tab with the
//! patient's way and where they go next, the notice of an open handover with taking it or
//! back, the quick handover's QR code, taking a patient from that code, and the handovers
//! waiting for this person. Within ORDO the patient stays with the holder until the
//! receiver takes them; outside ORDO they leave at once.

use leptos::prelude::*;
use ordo_med_patients::transport::TransportStatus;
use ordo_med_patients::{self as med, HandedOver, Holder, Patient, PatientRegister, Taken};

use crate::core::{self, Handle};
use crate::med::patients::{
    PlaceSelect, assign_chosen_place, free_places, holder_key, holder_of_key, holder_options,
    may_write, propose_place, writable_holders,
};
use crate::med::text;
use crate::shell::ui::Ago;

/// Where the patient is handed over to, as chosen on the tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Uhs,
    Unit,
    Transport,
    Outside,
}

impl Kind {
    const ALL: [Self; 4] = [Self::Uhs, Self::Unit, Self::Transport, Self::Outside];

    fn id(self) -> &'static str {
        match self {
            Self::Uhs => "uhs",
            Self::Unit => "unit",
            Self::Transport => "transport",
            Self::Outside => "outside",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Uhs => text::HANDOVER_TO_UHS,
            Self::Unit => text::HANDOVER_TO_UNIT,
            Self::Transport => text::HANDOVER_TO_TRANSPORT,
            Self::Outside => text::HANDOVER_TO_OUTSIDE,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Uhs => text::HANDOVER_TO_UHS_HINT,
            Self::Unit => text::HANDOVER_TO_UNIT_HINT,
            Self::Transport => text::HANDOVER_TO_TRANSPORT_HINT,
            Self::Outside => text::HANDOVER_TO_OUTSIDE_HINT,
        }
    }
}

/// The address a quick handover's QR code carries: the patient's record in this app, with
/// the handover. Any camera opens it; ORDO's own scanner reads it too. It names no clinical
/// data and grants nothing by itself.
fn quick_link(tag: &str, handover: &str) -> String {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    let route = crate::shell::nav::Route::patient(tag);
    format!("{origin}{}?handover={handover}", route.to_path())
}

/// The triage tag and handover of a quick handover's address, if it is one.
pub(crate) fn parse_link(link: &str) -> Option<(String, String)> {
    let (path, query) = link.split_once('?')?;
    let handover = query
        .split('&')
        .find_map(|p| p.strip_prefix("handover="))
        .filter(|h| !h.is_empty())?;
    let path = path.split_once("://").map_or(path, |(_, rest)| rest);
    let path = path.find('/').map_or(path, |i| &path[i..]);
    let route = crate::shell::nav::Route::from_path(path)?;
    Some((route.record?, handover.to_owned()))
}

// [impl->dsn~patient-handover~1]
/// The *Übergabe* tab's main part: the patient's way, then, for the holder's people, where
/// the patient goes next, a note, what the receiver gets and what follows, and the quick
/// handover by QR code.
#[component]
pub(crate) fn HandoverPanel(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let patient = {
        let tag = tag.clone();
        Memo::new(move |_| register.with(|r| r.patient(&tag).cloned()))
    };
    // Handing over is offered to the holder's people while the patient is here.
    let offered = Memo::new(move |_| {
        revision.track();
        patient.with(|p| {
            p.as_ref()
                .is_some_and(|p| p.is_present() && may_write(core, &p.holder()))
        })
    });
    let open =
        Memo::new(move |_| patient.with(|p| p.as_ref().is_some_and(|p| p.handover.is_some())));
    let document_tag = tag.clone();
    view! {
        <section class="record__section handover" aria-labelledby="section-way">
            <h3 id="section-way" tabindex="-1">{text::HANDOVER_WAY}</h3>
            <crate::med::flow::PatientFlow core revision tag=tag.clone() register
                next=Signal::derive(move || offered.get() && !open.get()) />
        </section>
        <Show when=move || offered.get() && !open.get()>
            <HandoverForm core revision tag=tag.clone() patient register />
            <QuickHandover core revision tag=tag.clone() />
        </Show>
        <crate::med::document::HandoverDocument core tag=document_tag />
    }
}

/// A choice of where to hand over: its value and label, and a line about it (e.g. free
/// places).
type Target = (String, String, Option<String>);

/// The targets of one kind, without the patient's holder.
fn targets(core: Handle, kind: Kind, patient: &Patient, register: &PatientRegister) -> Vec<Target> {
    let holder = patient.holder();
    match kind {
        Kind::Uhs => {
            let held = |uhs: &str| {
                register
                    .patients()
                    .filter(|p| p.is_present() && p.uhs == uhs)
                    .count()
            };
            crate::med::uhs::uhs_list(core)
                .into_iter()
                .filter(|u| holder.uhs() != Some(u.id.as_str()))
                .map(|u| {
                    let free = (u.capacity as usize).saturating_sub(held(&u.id));
                    (
                        u.id,
                        u.name,
                        Some(text::places_free(free, u.capacity as usize)),
                    )
                })
                .collect()
        }
        Kind::Unit => core.with_value(|c| {
            ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .present()
                .filter(|u| holder.unit() != Some(u.unit.as_str()))
                .map(|u| (u.unit.clone(), u.unit.clone(), Some(u.unit_type.clone())))
                .collect()
        }),
        Kind::Transport => crate::med::transports::patient_transports(core, &patient.tag)
            .into_iter()
            .filter(|t| t.status != TransportStatus::Cancelled && !t.status.has_left())
            .filter_map(|t| {
                // Not the unit that holds the patient already.
                let unit = t
                    .unit
                    .clone()
                    .filter(|u| holder.unit() != Some(u.as_str()))?;
                Some((
                    t.request.request.clone(),
                    crate::med::transports::transport_line(&t),
                    Some(unit),
                ))
            })
            .collect(),
        Kind::Outside => Vec::new(),
    }
}

// [impl->dsn~patient-pathway~2]
// [impl->dsn~patient-handover~1]
/// Where the patient goes, who their record goes to, a note, and the button.
#[component]
fn HandoverForm(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    patient: Memo<Option<Patient>>,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let kind = RwSignal::new(Kind::Uhs);
    let target = RwSignal::new(String::new());
    let receiver = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let options = Memo::new(move |_| {
        revision.track();
        let chosen = kind.get();
        let Some(p) = patient.get() else {
            return Vec::new();
        };
        register.with(|r| targets(core, chosen, &p, r))
    });
    // The first target of the kind is chosen; the person changes it.
    Effect::new(move || {
        let list = options.get();
        if !list.iter().any(|(v, ..)| *v == target.get_untracked()) {
            target.set(list.first().map(|(v, ..)| v.clone()).unwrap_or_default());
        }
    });
    // Who gets the record, by name, for the button and the explanation.
    let to = Memo::new(move |_| match kind.get() {
        Kind::Outside => {
            let r = receiver.get();
            (!r.trim().is_empty()).then(|| r.trim().to_owned())
        }
        _ => options.with(|o| {
            o.iter()
                .find(|(v, ..)| *v == target.get())
                .map(|(v, label, unit)| match kind.get_untracked() {
                    Kind::Transport => unit.clone().unwrap_or_else(|| label.clone()),
                    Kind::Uhs => label.clone(),
                    _ => v.clone(),
                })
        }),
    });
    let submit = {
        let tag = tag.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let chosen = kind.get_untracked();
            let value = target.get_untracked();
            let (to, outside, transport) = match chosen {
                Kind::Outside => {
                    let r = receiver.get_untracked().trim().to_owned();
                    if r.is_empty() {
                        error.set(Some(text::HANDOVER_RECEIVER_MISSING));
                        return;
                    }
                    (None, Some(r), None)
                }
                Kind::Transport => {
                    let Some((_, _, Some(unit))) =
                        options.with_untracked(|o| o.iter().find(|(v, ..)| *v == value).cloned())
                    else {
                        error.set(Some(text::HANDOVER_TARGET_MISSING));
                        return;
                    };
                    (Some(Holder::Unit(unit)), None, Some(value))
                }
                Kind::Uhs | Kind::Unit if value.is_empty() => {
                    error.set(Some(text::HANDOVER_TARGET_MISSING));
                    return;
                }
                Kind::Uhs => (Some(Holder::Uhs(value)), None, None),
                Kind::Unit => (Some(Holder::Unit(value)), None, None),
            };
            let handover = HandedOver {
                tag: tag.clone(),
                handover: String::new(),
                to,
                outside,
                transport,
                note: Some(note.get_untracked().trim().to_owned()).filter(|n| !n.is_empty()),
            };
            match core::hand_over(core, revision, handover) {
                Ok(_) => {
                    error.set(None);
                    note.set(String::new());
                    receiver.set(String::new());
                    crate::shell::views::after_render(|| {
                        crate::shell::views::focus("handover-open")
                    });
                }
                Err(_) => error.set(Some(text::HANDOVER_FAILED)),
            }
        }
    };
    let reset = move |_| {
        kind.set(Kind::Uhs);
        note.set(String::new());
        receiver.set(String::new());
        error.set(None);
    };
    view! {
        <form class="record__section handover-form" novalidate on:submit=submit aria-labelledby="section-handover">
            <h3 id="section-handover" tabindex="-1">{text::HANDOVER_TITLE}</h3>
            <fieldset class="handover-choices">
                <legend>{text::HANDOVER_TO}</legend>
                {Kind::ALL.into_iter().map(|k| view! {
                    <label class="handover-choice" class:handover-choice--on=move || kind.get() == k>
                        <input type="radio" name="handover-kind" value=k.id()
                            prop:checked=move || kind.get() == k on:change=move |_| { kind.set(k); error.set(None); } />
                        <strong>{k.title()}</strong>
                        <span class="hint">{k.description()}</span>
                    </label>
                }).collect_view()}
            </fieldset>
            <div class="handover-form__details">
                {move || match kind.get() {
                    Kind::Outside => view! {
                        <div class="field">
                            <label for="handover-receiver">{text::HANDOVER_RECEIVER}</label>
                            <input id="handover-receiver" type="text" autocomplete="off" bind:value=receiver
                                aria-describedby="handover-receiver-hint" />
                            <p id="handover-receiver-hint" class="hint">{text::HANDOVER_RECEIVER_HINT}</p>
                        </div>
                    }.into_any(),
                    k => view! {
                        <div class="field">
                            <label for="handover-target">{text::handover_target_label(k.title())}</label>
                            {move || {
                                let list = options.get();
                                if list.is_empty() {
                                    return view! { <p class="hint" id="handover-target">{text::handover_no_target(k == Kind::Transport)}</p> }.into_any();
                                }
                                view! {
                                    <select id="handover-target" prop:value=move || target.get() on:change=move |ev| target.set(event_target_value(&ev))>
                                        {list.into_iter().map(|(value, label, line)| {
                                            let chosen = value.clone();
                                            let shown = match line { Some(l) if k != Kind::Transport => format!("{label} · {l}"), _ => label };
                                            view! { <option value=value selected=move || target.get() == chosen>{shown}</option> }
                                        }).collect_view()}
                                    </select>
                                }.into_any()
                            }}
                        </div>
                    }.into_any(),
                }}
                <div class="field">
                    <label for="handover-note">{text::HANDOVER_NOTE}</label>
                    <textarea id="handover-note" rows="2" bind:value=note placeholder=text::HANDOVER_NOTE_EXAMPLE></textarea>
                </div>
            </div>
            <div class="handover-form__info">
                <section aria-labelledby="handover-gets">
                    <h4 id="handover-gets">{move || text::handover_gets(&to.get().unwrap_or_else(|| text::HANDOVER_RECEIVER_ANY.to_owned()))}</h4>
                    <ul>
                        {move || if kind.get() == Kind::Outside {
                            view! { <li>{text::HANDOVER_GETS_OUTSIDE}</li> }.into_any()
                        } else {
                            view! {
                                <li>{text::HANDOVER_GETS_RECORD}</li>
                                <li>{text::HANDOVER_GETS_NOTE}</li>
                            }.into_any()
                        }}
                    </ul>
                </section>
                <section aria-labelledby="handover-after">
                    <h4 id="handover-after">{text::HANDOVER_AFTER}</h4>
                    <p>{move || if kind.get() == Kind::Outside { text::HANDOVER_AFTER_OUTSIDE } else { text::HANDOVER_AFTER_INSIDE }}</p>
                </section>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="actions">
                <button type="button" class="button button--quiet" on:click=reset>{text::cancel()}</button>
                <button type="submit" class="button button--primary button--with-icon">
                    <crate::shell::ui::Icon name="arrow-right-left" />
                    {move || match to.get() {
                        Some(to) => text::handover_button(&to),
                        None => text::HANDOVER_BUTTON.to_owned(),
                    }}
                </button>
            </div>
        </form>
    }
}

// [impl->dsn~quick-handover~1]
/// The quick handover: records a handover to nobody in particular; its notice then shows
/// the QR code.
#[component]
fn QuickHandover(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let error = RwSignal::new(None::<&'static str>);
    let start = move |_| {
        let handover = HandedOver {
            tag: tag.clone(),
            handover: String::new(),
            to: None,
            outside: None,
            transport: None,
            note: None,
        };
        match core::hand_over(core, revision, handover) {
            Ok(_) => {
                crate::shell::views::after_render(|| crate::shell::views::focus("handover-open"))
            }
            Err(_) => error.set(Some(text::HANDOVER_FAILED)),
        }
    };
    view! {
        <section class="record__section handover-quick" aria-labelledby="section-quick">
            <h3 id="section-quick">{text::QUICK_HANDOVER}</h3>
            <p class="hint" id="quick-hint">{text::QUICK_HANDOVER_HINT}</p>
            <div class="actions">
                <button type="button" class="button button--with-icon" aria-describedby="quick-hint" on:click=start>
                    <crate::shell::ui::Icon name="qr-code" />{text::QUICK_HANDOVER_START}
                </button>
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}

/// The QR code of an address, as an image the device makes itself (offline too).
#[component]
fn QuickCode(link: String) -> impl IntoView {
    use base64::Engine;
    let image = crate::shell::qr::svg(&link).map(|svg| {
        format!(
            "data:image/svg+xml;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(svg)
        )
    });
    view! {
        <div class="handover-code">
            {image.map(|src| view! { <img class="handover-code__image" src=src alt=text::QUICK_CODE_ALT width="224" height="224" /> })}
            <p class="hint">{text::QUICK_CODE_HINT}</p>
            <p class="hint">{text::guest_link()} " " <a class="handover-code__link" href=link.clone()>{link.clone()}</a></p>
        </div>
    }
}

// [impl->dsn~patient-handover~1]
// [impl->dsn~take-with-place~1]
/// Taking a patient: for whom, where several holders are offered, the treatment place of
/// a UHS with free places (the one kept for the patient's category proposed, EPCR-17), and
/// "Übernehmen". Every place of taking uses it: the record, the quick handover's dialog and
/// the list of handovers waiting.
#[component]
pub(crate) fn TakeControls(
    core: Handle,
    revision: RwSignal<u64>,
    register: Memo<PatientRegister>,
    /// Prefix of the controls' IDs.
    #[prop(into)]
    id: String,
    tag: String,
    handover: String,
    /// Where this person may take the patient; with one, it is not asked.
    #[prop(into)]
    holders: Signal<Vec<(Holder, String)>>,
    /// After the patient was taken.
    #[prop(optional)]
    on_taken: Option<Callback<String>>,
) -> impl IntoView {
    let chosen = RwSignal::new(String::new());
    Effect::new(move || {
        if let Some((h, _)) = holders.with(|m| m.first().cloned())
            && !holders.with(|m| {
                m.iter()
                    .any(|(o, _)| holder_key(o) == chosen.get_untracked())
            })
        {
            chosen.set(holder_key(&h));
        }
    });
    let category = {
        let tag = tag.clone();
        Memo::new(move |_| register.with(|r| r.patient(&tag).map(Patient::category)))
    };
    let free = Memo::new(move |_| {
        revision.track();
        let holder = holder_of_key(&chosen.get());
        register.with(|r| free_places(core, r, holder.as_ref()))
    });
    let place = RwSignal::new(String::new());
    Effect::new(move || {
        let proposed = category
            .get()
            .and_then(|c| free.with(|f| propose_place(f, c)));
        place.set(proposed.unwrap_or_default());
    });
    let error = RwSignal::new(None::<String>);
    let take = {
        let tag = tag.clone();
        move |_| {
            let Some(holder) = holder_of_key(&chosen.get_untracked()) else {
                error.set(Some(text::HANDOVER_TARGET_MISSING.to_owned()));
                return;
            };
            let taken = Taken {
                tag: tag.clone(),
                handover: handover.clone(),
                holder,
            };
            if core::take(core, revision, &taken).is_err() {
                error.set(Some(text::HANDOVER_FAILED.to_owned()));
                return;
            }
            let on = place.get_untracked();
            if let Err(e) =
                free.with_untracked(|f| assign_chosen_place(core, revision, &tag, f, &on))
            {
                error.set(Some(e));
                return;
            }
            if let Some(f) = on_taken {
                f.run(tag.clone());
            }
        }
    };
    let (to_id, place_id, error_id) = (
        format!("{id}-to"),
        format!("{id}-place"),
        format!("{id}-error"),
    );
    let words = move || match holders.with(|h| (h.len() == 1).then(|| h[0].1.clone())) {
        Some(name) => text::handover_take(&name),
        None => text::HANDOVER_TAKE.to_owned(),
    };
    let label = {
        let tag = tag.clone();
        move || text::handover_take_label(&words(), &tag)
    };
    view! {
        <div class="take">
            <Show when=move || holders.with(|h| h.len() > 1)>
                <div class="field">
                    <label for=to_id.clone()>{text::HANDOVER_TAKE_TO}</label>
                    <select id=to_id.clone() prop:value=move || chosen.get() on:change=move |ev| chosen.set(event_target_value(&ev))>
                        {move || holder_options(holders.get(), chosen)}
                    </select>
                </div>
            </Show>
            <PlaceSelect id=place_id free=free place hint=text::TAKE_PLACE_HINT />
            <div class="actions">
                <button type="button" class="button button--primary" aria-label=label aria-describedby=error_id.clone() on:click=take>
                    {words}
                </button>
            </div>
            <p id=error_id role="alert" class="error">{move || error.get()}</p>
        </div>
    }
}

// [impl->dsn~patient-handover~1]
/// The notice of an open handover, at the top of the record: to whom, since when, the
/// note; for the receiver "Übernehmen", for the holder "Zurücknehmen", and the QR code of a
/// quick handover.
#[component]
pub(crate) fn HandoverNotice(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            let p = register.with(|r| r.patient(&tag).cloned())?;
            let open = p.handover.clone()?;
            let holder = p.holder();
            let giver = may_write(core, &holder);
            let receiver = open.to.as_ref().filter(|to| may_write(core, to)).cloned();
            Some((p, open, holder, giver, receiver))
        })
    };
    let error = RwSignal::new(None::<&'static str>);
    let take_back = move |holder: Holder| {
        let Some((p, open, ..)) = state.get_untracked() else {
            return;
        };
        let taken = Taken {
            tag: p.tag.clone(),
            handover: open.id.clone(),
            holder,
        };
        if core::take(core, revision, &taken).is_err() {
            error.set(Some(text::HANDOVER_FAILED));
        }
    };
    // A quick handover is taken here for one of one's own holders.
    let mine = Memo::new(move |_| {
        revision.track();
        let holder = state.with(|s| s.as_ref().map(|s| s.2.clone()));
        writable_holders(core)
            .into_iter()
            .filter(|(h, _)| Some(h) != holder.as_ref())
            .collect::<Vec<_>>()
    });
    view! {
        {move || state.get().map(|(p, open, holder, giver, receiver)| {
            let to = open.to.as_ref().map(|t| core::holder_name(core, t));
            let since = open.at.wall_ms;
            let by = crate::shell::profile::display_name(core, &open.by);
            let quick = open.to.is_none();
            view! {
                <section id="handover-open" tabindex="-1" class="notice handover-open" aria-labelledby="handover-open-title">
                    <h3 id="handover-open-title" class="handover-open__title">
                        <crate::shell::ui::Icon name="arrow-right-left" />
                        {match &to {
                            Some(to) => text::handover_open_to(to),
                            None => text::HANDOVER_OPEN_QUICK.to_owned(),
                        }}
                    </h3>
                    <p>
                        {text::handover_open_since(&core::holder_name(core, &holder), &core::clock(since), &by)}
                        " " <Ago at=since />
                    </p>
                    {open.note.clone().map(|n| view! { <p><strong>{text::HANDOVER_NOTE_SHORT}</strong> " " {n}</p> })}
                    <p class="hint">{if quick { text::HANDOVER_OPEN_QUICK_HINT } else { text::HANDOVER_OPEN_HINT }}</p>
                    {(giver && quick).then(|| view! { <QuickCode link=quick_link(&p.tag, &open.id) /> })}
                    {receiver.clone().map(|r| {
                        let name = core::holder_name(core, &r);
                        view! {
                            <TakeControls core revision register id="handover-take" tag=p.tag.clone() handover=open.id.clone()
                                holders=Signal::derive(move || vec![(r.clone(), name.clone())]) />
                        }
                    })}
                    {(quick && !mine.with(Vec::is_empty)).then(|| view! {
                        <TakeControls core revision register id="handover-take" tag=p.tag.clone() handover=open.id.clone()
                            holders=mine />
                    })}
                    {giver.then(|| {
                        let back = holder.clone();
                        view! {
                            <div class="actions">
                                <button type="button" class="button button--quiet" on:click=move |_| take_back(back.clone())>
                                    {text::HANDOVER_TAKE_BACK}
                                </button>
                            </div>
                        }
                    })}
                    <p role="alert" class="error">{move || error.get()}</p>
                </section>
            }
        })}
    }
}

// [impl->dsn~quick-handover~1]
/// Taking a patient from a quick handover's code, opened by its address or scanned in
/// ORDO: the patient, and where this person takes them among the holders they write.
#[component]
pub(crate) fn TakeDialog(
    core: Handle,
    revision: RwSignal<u64>,
    nav: crate::shell::views::Nav,
    register: Memo<PatientRegister>,
    /// The triage tag and the handover, while the dialog is open.
    taking: RwSignal<Option<(String, String)>>,
) -> impl IntoView {
    let mine = Memo::new(move |_| {
        revision.track();
        writable_holders(core)
    });
    // What this device knows: the patient, and whether the handover is open, taken or not
    // here yet.
    let state = Memo::new(move |_| {
        let (tag, handover) = taking.get()?;
        let p = register.with(|r| r.patient(&tag).cloned());
        Some((tag, handover, p))
    });
    let close = Callback::new(move |()| {
        taking.set(None);
        core::clear_url_params();
    });
    let taken = Callback::new(move |tag: String| {
        close.run(());
        let route = crate::shell::nav::Route::patient(&tag);
        nav.go(route);
        crate::shell::views::after_render(|| crate::shell::views::focus("record-title"));
    });
    view! {
        <crate::shell::ui::EditDialog id="take" open=Signal::derive(move || taking.with(Option::is_some))
            on_close=close back=Signal::derive(|| "scan-handover-button".to_owned())
            title=Signal::derive(move || taking.with(|t| t.as_ref().map(|(tag, _)| text::take_title(tag)).unwrap_or_default()))>
            {move || state.get().map(|(_, handover, patient)| match patient {
                None => view! { <p class="notice">{text::TAKE_NOT_HERE}</p> }.into_any(),
                Some(p) if !p.is_present() => view! { <p class="notice">{text::TAKE_GONE}</p> }.into_any(),
                Some(p) if p.handover.as_ref().is_none_or(|h| h.id != handover) && p.handover_taken(&handover) => {
                    view! { <p class="notice">{text::TAKE_TAKEN}</p> }.into_any()
                }
                Some(p) => {
                    let known = p.handover.as_ref().is_some_and(|h| h.id == handover);
                    view! {
                        <p>{text::take_from(&core::holder_name(core, &p.holder()))}</p>
                        {(!known).then(|| view! { <p class="hint">{text::TAKE_NOT_ARRIVED}</p> })}
                        {if mine.with(Vec::is_empty) {
                            view! { <p class="notice">{text::TAKE_NOWHERE}</p> }.into_any()
                        } else {
                            view! {
                                <TakeControls core revision register id="take" tag=p.tag.clone() handover=handover.clone()
                                    holders=mine on_taken=taken />
                            }.into_any()
                        }}
                    }.into_any()
                }
            })}
        </crate::shell::ui::EditDialog>
    }
}

// [impl->dsn~quick-handover~1]
/// "Übergabe scannen": a photo of a quick handover's code, read on the device (ADR-0073).
#[component]
pub(crate) fn ScanHandover(taking: RwSignal<Option<(String, String)>>) -> impl IntoView {
    let error = RwSignal::new(None::<&'static str>);
    let busy = RwSignal::new(false);
    let on_file = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        let Some(file) = input.files().and_then(|f| f.get(0)) else {
            return;
        };
        input.set_value("");
        busy.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match crate::shell::images::read_qr(&file).await {
                Ok(texts) => match texts.iter().find_map(|t| parse_link(t)) {
                    Some(found) => taking.set(Some(found)),
                    None => error.set(Some(text::SCAN_NO_HANDOVER)),
                },
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <div class="scan-handover">
            <input id="scan-handover" class="file-input visually-hidden" type="file" accept="image/*" capture="environment"
                aria-describedby="scan-handover-hint" on:change=on_file />
            <label id="scan-handover-button" for="scan-handover" class="button button--with-icon" tabindex="0"
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        crate::shell::views::click("scan-handover");
                    }
                }>
                <crate::shell::ui::Icon name="scan-line" />{text::SCAN_HANDOVER}
            </label>
            <p id="scan-handover-hint" class="hint">{move || if busy.get() { text::SCAN_READING } else { text::SCAN_HANDOVER_HINT }}</p>
            <p role="alert" class="error">{move || error.get()}</p>
        </div>
    }
}

/// The open handovers this person may take: to a holder they write, and not from one.
pub(crate) fn incoming(core: Handle, register: &PatientRegister) -> Vec<(Patient, Holder)> {
    register
        .patients()
        .filter(|p| p.is_present())
        .filter_map(|p| {
            let to = p.handover.as_ref()?.to.clone()?;
            (may_write(core, &to) && !may_write(core, &p.holder())).then(|| (p.clone(), to))
        })
        .collect()
}

// [impl->dsn~patient-handover~1]
/// *Übergaben an Sie*: the patients handed over to a holder this person writes, each with
/// its record and "Übernehmen".
#[component]
pub(crate) fn IncomingHandovers(
    core: Handle,
    revision: RwSignal<u64>,
    nav: crate::shell::views::Nav,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let list = Memo::new(move |_| {
        revision.track();
        register.with(|r| incoming(core, r))
    });
    view! {
        <Show when=move || list.with(|l| !l.is_empty())>
            <section class="card incoming" aria-labelledby="incoming-title">
                <h2 id="incoming-title">{text::INCOMING_TITLE}</h2>
                <ul class="incoming__list">
                    {move || list.get().into_iter().map(|(p, to)| {
                        let open = p.handover.clone().expect("open");
                        let route = crate::shell::nav::Route::patient(&p.tag);
                        let (tag, from, at) = (p.tag.clone(), core::holder_name(core, &p.holder()), open.at.wall_ms);
                        let to_name = core::holder_name(core, &to);
                        view! {
                            <li class="incoming__item" data-tag=tag.clone()>
                                <span class="incoming__what">
                                    <strong class="code">{tag.clone()}</strong> " "
                                    <crate::med::triage::TriageBadge category=p.category() />
                                    " " {text::incoming_line(&from, &to_name)} " · " {core::clock(at)} " " <Ago at />
                                </span>
                                {open.note.map(|n| view! { <span class="hint">{n}</span> })}
                                <a class="button button--quiet incoming__open" href=route.to_path() on:click=move |ev| {
                                    ev.prevent_default();
                                    nav.go(route.clone());
                                }>{text::RECORD_OPEN}</a>
                                <TakeControls core revision register id=format!("incoming-{tag}") tag=tag.clone() handover=open.id.clone()
                                    holders=Signal::derive(move || vec![(to.clone(), to_name.clone())]) />
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </section>
        </Show>
    }
}

/// The bell's items of handovers waiting for this person.
pub(crate) fn bell(core: Handle) -> Vec<(String, crate::shell::nav::Route)> {
    let register = core.with_value(|c| PatientRegister::from_events(&c.events(med::COMPARTMENT)));
    incoming(core, &register)
        .into_iter()
        .map(|(p, to)| {
            (
                text::handover_bell(&p.tag, &core::holder_name(core, &to)),
                crate::shell::nav::Route::patient(&p.tag),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~quick-handover~1]
    #[test]
    fn a_quick_handover_link_names_the_patient_and_the_handover() {
        assert_eq!(
            parse_link("https://ordo.example.org/med.uhs/patients/DE-0044?handover=01JD"),
            Some(("DE-0044".to_owned(), "01JD".to_owned()))
        );
        assert_eq!(
            parse_link("https://ordo.example.org/med.uhs/patients/DE-0044"),
            None
        );
        assert_eq!(
            parse_link("https://ordo.example.org/patient/DE-0044?handover=01JD"),
            Some(("DE-0044".to_owned(), "01JD".to_owned())),
            "the patient's own address (ADR-0093)"
        );
        assert_eq!(parse_link("ordo:pair?code=x"), None);
    }
}
