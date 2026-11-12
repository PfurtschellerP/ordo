//! Medical orders (ORDER-1…4): given on the patient record by physicians and paramedics,
//! carried out by the crew, and listed across every UHS in the view *Anordnungen*. Orders
//! given by voice, radio or phone are recorded by the crew and confirmed later by their
//! giver (ORDER-6). ORDO records orders; it never proposes, checks or calculates them
//! (EPCR-9).

use std::collections::HashMap;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_med_patients::orders::{
    Channel, Order, OrderBook, OrderCatalogue, OrderGiven, OrderRecorded, OrderStatus,
    OrderUpdated, Ordered,
};
use ordo_med_patients::record::FormCatalogue;
use ordo_med_patients::{self as med, PatientRegister};

use crate::core::{self, Handle};
use crate::patients::{FORMS, open_prefilled, record_route};
use crate::text;
use crate::ui::{Action, Actions, History, IncidentPeople, RadioChips, Suggestions};
use crate::views::{Nav, after_render, focus};

/// Routes of administration from the profile (checked at build time, see `build.rs`).
pub static CATALOGUE: LazyLock<OrderCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/orders.json")))
        .expect("order settings were checked at build time")
});

/// Who carries out each open order, as typed (by order ID), kept for the workspace so a
/// re-render keeps the input (provided by the workspace).
#[derive(Clone, Copy)]
pub struct Performers(StoredValue<HashMap<String, String>, LocalStorage>);

impl Default for Performers {
    fn default() -> Self {
        Self(StoredValue::new_local(HashMap::new()))
    }
}

/// What was ordered, as one line: "Paracetamol 1 g i.v." or the measure.
fn ordered_line(o: &OrderGiven) -> String {
    let main = match &o.ordered {
        Ordered::Medication {
            substance,
            dose,
            route,
        } => format!("{substance} {dose} {}", CATALOGUE.route_label(route)),
        Ordered::Measure { text } => text.clone(),
    };
    match &o.note {
        Some(note) => format!("{main} ({note})"),
        None => main,
    }
}

/// The ID of a channel in its payload, e.g. `radio`.
fn channel_id(channel: Channel) -> String {
    serde_json::to_value(channel)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

// [impl->dsn~order-documentation~3]
/// Whether `field` of `form` is the one that names who ordered a medication: filled only
/// from an order, never typed (ORDER-7).
pub fn is_ordered_by(form: &str, field: &str) -> bool {
    CATALOGUE
        .record
        .as_ref()
        .is_some_and(|r| r.form == form && r.ordered_by == field)
}

/// Whether this person's roles in the incident allow giving orders (the server decides
/// again for every order).
fn may_give(core: Handle) -> bool {
    core.with_value(|c| {
        c.incident
            .as_ref()
            .is_some_and(|i| c.replica.may(i, "give_orders"))
    })
}

/// One order: what, status, who ordered it and when, history, and the next steps.
#[component]
fn OrderItem(
    core: Handle,
    revision: RwSignal<u64>,
    order: Order,
    #[prop(optional)] patient: Option<(String, String)>,
    #[prop(optional)] nav: Option<Nav>,
) -> impl IntoView {
    let what = ordered_line(&order.given);
    let id = order.given.order.clone();
    let by = {
        let (at, user) = (core::clock(order.at().wall_ms), order.by().clone());
        let spoken = order.spoken.is_some();
        crate::profile::naming(move || {
            let name = crate::profile::display_name(core, &user);
            if spoken {
                text::order_recorded_by(&at, &name)
            } else {
                text::order_by(&at, &name)
            }
        })
    };
    // [impl->dsn~recorded-orders~1]
    // A spoken order: who gave it, how, and whether its giver confirmed it.
    let spoken = order.spoken.clone().map(|s| {
        let giver = s.ordered_by.clone();
        let line = crate::profile::naming(move || {
            text::order_spoken(
                &crate::profile::display_text(core, &giver),
                text::order_channel(s.channel),
                s.read_back,
            )
        });
        let confirmed = s.confirmed.clone();
        view! {
            <p class="order-card__spoken">{line}</p>
            {match confirmed {
                Some(c) => {
                    let at = core::clock(c.at.wall_ms);
                    view! {
                        <p class="order-card__confirmed">
                            {crate::profile::naming(move || text::order_confirmed(&at, &crate::profile::display_name(core, &c.by)))}
                        </p>
                    }
                    .into_any()
                }
                None => view! { <p class="order-card__unconfirmed"><strong>{text::ORDER_UNCONFIRMED}</strong></p> }.into_any(),
            }}
        }
    });
    // [impl->dsn~order-documentation~3]
    // On the record, a done medication order can be taken into the documentation.
    let drafts = use_context::<crate::patients::Drafts>();
    let me = core.with_value(|c| c.replica.user().to_string());
    let performers = use_context::<Performers>().unwrap_or_default();
    // While it can still be reported done: who carries it out, the reporting person
    // unless another is named (ORDER-2).
    let performer_field = order.status.next().contains(&OrderStatus::Done).then(|| {
        let field = format!("performer-{id}");
        let list = format!("{field}-people");
        let people = use_context::<IncidentPeople>().map_or_else(|| Signal::stored(Vec::new()), |p| p.0);
        let (key, key_in) = (id.clone(), id.clone());
        let me_value = me.clone();
        view! {
            <div class="field order-card__performer">
                <label for=field.clone()>{text::ORDER_PERFORMED_BY}</label>
                <input id=field type="text" autocomplete="off" list=list.clone()
                    prop:value=move || performers.0.with_value(|p| p.get(&key).cloned()).unwrap_or_else(|| me_value.clone())
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        performers.0.update_value(|p| { p.insert(key_in.clone(), value); });
                    } />
                <Suggestions id=list values=people core />
            </div>
        }
    });
    // The person named confirms it, if their roles give orders (the server decides again).
    // The permissions may arrive after the orders, so this follows `revision`.
    let confirm = (order.unconfirmed() && order.giver() == me).then(|| {
        let (id, label) = (id.clone(), text::order_confirm_label(&what));
        move || {
            revision.track();
            let (id, label) = (id.clone(), label.clone());
            may_give(core).then(|| view! {
                <div class="row">
                    <button type="button" class="button button--small button--primary" aria-label=label
                        on:click=move |_| { let _ = core::confirm_order(core, revision, &id); }>
                        {text::ORDER_CONFIRM}
                    </button>
                </div>
            })
        }
    });
    let done = order.done().cloned();
    let documentable = patient.is_none()
        && done.is_some()
        && matches!(order.given.ordered, Ordered::Medication { .. })
        && CATALOGUE.record.is_some();
    let document = documentable.then(|| {
        // Who gave the order, not who wrote it into ORDO (ORDER-7).
        let (given, ordered_by, label) = (order.given.clone(), order.giver(), text::document_order_label(&what));
        // Who carried it out and when, not who documents it now (ORDER-5).
        let step = done.clone().expect("done");
        let (performer, at) = (step.performer(), core::clock(step.at.wall_ms));
        view! {
            <div class="row">
                <button type="button" class="button button--small" aria-label=label on:click=move |_| {
                    let forms: &'static FormCatalogue = &FORMS;
                    if let Some((form, values)) = CATALOGUE.documentation(&given, &ordered_by, &performer, forms)
                        && let (Some(form), Some(drafts)) = (forms.form(&form), drafts)
                    {
                        open_prefilled(drafts, &given.tag, form, values, Some(at.clone()), Some(given.order.clone()));
                    }
                }>
                    {text::DOCUMENT_ORDER}
                </button>
            </div>
        }
    });
    view! {
        <li class="order-card" data-order=id.clone() data-status=format!("{:?}", order.status).to_lowercase()>
            {patient.map(|(tag, uhs)| {
                let route = record_route(&tag);
                view! {
                    <p class="order-card__patient">
                        <a href=route.to_path() on:click=move |ev| {
                            if let Some(nav) = nav {
                                ev.prevent_default();
                                nav.go(route.clone());
                                after_render(|| focus("record-title"));
                            }
                        }>
                            <span class="code">{tag.clone()}</span>
                        </a>
                        " · " {uhs}
                    </p>
                }
            })}
            <p class="order-card__what"><strong>{what.clone()}</strong></p>
            <p class="order-card__status">{text::order_status(order.status)} " · " {by}</p>
            {spoken}
            {done.as_ref().and_then(|s| s.performed_by.clone()).map(|p| view! {
                <p class="order-card__performed">{crate::profile::naming(move || text::order_performed_by(&crate::profile::display_text(core, &p)))}</p>
            })}
            <History entries=crate::profile::naming({
                let history = order.history.clone();
                move || history
                .iter()
                .map(|s| {
                    let what = match &s.performed_by {
                        Some(p) => text::order_status_performed(
                            text::order_status(s.status),
                            &crate::profile::display_text(core, p),
                        ),
                        None => text::order_status(s.status).to_owned(),
                    };
                    (core::clock(s.at.wall_ms), what, crate::profile::display_name(core, &s.by))
                })
                .collect()
            }) />
            {performer_field}
            <Actions actions=order
                .status
                .next()
                .into_iter()
                .map(|status| {
                    let (id, me) = (id.clone(), me.clone());
                    let text = text::order_step(status);
                    Action {
                        text,
                        label: text::transport_step_label(text, &what),
                        quiet: status == OrderStatus::Withdrawn,
                        run: Callback::new(move |()| {
                            // [impl->dsn~medical-orders~2]
                            // Done: who carried it out, if another person than the one reporting.
                            let performed_by = (status == OrderStatus::Done)
                                .then(|| performers.0.with_value(|p| p.get(&id).cloned()))
                                .flatten()
                                .map(|p| p.trim().to_owned())
                                .filter(|p| !p.is_empty() && *p != me);
                            let update = OrderUpdated { order: id.clone(), status, performed_by };
                            let _ = core::update_order(core, revision, &update);
                        }),
                    }
                })
                .collect() />
            {confirm}
            {document}
        </li>
    }
}

// [impl->dsn~medical-orders~2]
// [impl->dsn~recorded-orders~1]
/// The orders of one patient on the record, and the form: those who may give orders give
/// one or record a spoken one, the rest of the crew records spoken ones.
#[component]
pub fn OrdersSection(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let list = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            core.with_value(|c| {
                OrderBook::from_events(&c.events(med::orders::COMPARTMENT))
                    .of_patient(&tag)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    };
    let can_give = Memo::new(move |_| {
        revision.track();
        may_give(core)
    });
    let kind = RwSignal::new("medication".to_owned());
    let medication = move || kind.get() == "medication";
    let substance = RwSignal::new(String::new());
    let dose = RwSignal::new(String::new());
    let route = RwSignal::new(String::new());
    let measure = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let origin = RwSignal::new("mine".to_owned());
    let spoken = move || !can_give.get() || origin.get() == "spoken";
    let giver = RwSignal::new(String::new());
    let channel = RwSignal::new(channel_id(Channel::InPerson));
    let read_back = RwSignal::new(false);
    let people =
        use_context::<IncidentPeople>().map_or_else(|| Signal::stored(Vec::new()), |p| p.0);
    let message = RwSignal::new(None::<&'static str>);
    let submit = {
        let tag = tag.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let spoken = !can_give.get_untracked() || origin.get_untracked() == "spoken";
            let ordered = if kind.get_untracked() == "medication" {
                Ordered::Medication {
                    substance: substance.get_untracked().trim().to_owned(),
                    dose: dose.get_untracked().trim().to_owned(),
                    route: route.get_untracked(),
                }
            } else {
                Ordered::Measure {
                    text: measure.get_untracked().trim().to_owned(),
                }
            };
            let note_text = note.get_untracked().trim().to_owned();
            let order = OrderGiven {
                order: String::new(),
                tag: tag.clone(),
                ordered,
                note: (!note_text.is_empty()).then_some(note_text),
            };
            let result = if spoken {
                let recorded = OrderRecorded {
                    order: order.order,
                    tag: order.tag,
                    ordered: order.ordered,
                    note: order.note,
                    ordered_by: giver.get_untracked().trim().to_owned(),
                    channel: Channel::ALL
                        .into_iter()
                        .find(|c| channel_id(*c) == channel.get_untracked())
                        .unwrap_or(Channel::InPerson),
                    read_back: read_back.get_untracked(),
                };
                if recorded.check(&CATALOGUE).is_err() {
                    message.set(Some(text::ORDER_INCOMPLETE));
                    return;
                }
                core::record_order(core, revision, recorded)
            } else {
                if order.check(&CATALOGUE).is_err() {
                    message.set(Some(text::ORDER_INCOMPLETE));
                    return;
                }
                core::give_order(core, revision, order)
            };
            if result.is_ok() {
                for s in [substance, dose, route, measure, note, giver] {
                    s.set(String::new());
                }
                read_back.set(false);
                message.set(None);
            }
        }
    };
    view! {
        <section class="record__section" aria-labelledby="section-orders">
            <h3 id="section-orders" tabindex="-1">{text::ORDERS}</h3>
            {move || {
                let orders = list.get();
                if orders.is_empty() {
                    return view! { <p class="hint">{text::ORDERS_NONE}</p> }.into_any();
                }
                view! {
                    <ul class="order-list" data-keys="list">
                        {orders
                            .into_iter()
                            .map(|order| view! { <OrderItem core revision order /> })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <form class="form order-form" novalidate on:submit=submit.clone()>
                <Show when=move || can_give.get()>
                    <RadioChips legend=text::ORDER_WHO name="order-origin" value=origin options=vec![
                        ("mine".to_owned(), text::ORDER_MINE.to_owned()),
                        ("spoken".to_owned(), text::ORDER_SPOKEN.to_owned()),
                    ] />
                </Show>
                <Show when=spoken>
                    <div class="field">
                        <label for="order-given-by">{text::ORDER_ORDERED_BY}</label>
                        <input id="order-given-by" type="text" autocomplete="off" list="order-given-by-people" bind:value=giver />
                        <Suggestions id="order-given-by-people" values=people core />
                    </div>
                    <RadioChips legend=text::ORDER_CHANNEL name="order-channel" value=channel options=Channel::ALL
                        .into_iter()
                        .map(|c| (channel_id(c), text::order_channel(c).to_owned()))
                        .collect() />
                    <div class="field">
                        <label class="choice chip">
                            <input id="order-read-back" type="checkbox" bind:checked=read_back />
                            {text::ORDER_READ_BACK}
                        </label>
                    </div>
                </Show>
                <RadioChips legend=text::ORDER_KIND name="order-kind" value=kind options=vec![
                    ("medication".to_owned(), text::ORDER_MEDICATION.to_owned()),
                    ("measure".to_owned(), text::ORDER_MEASURE.to_owned()),
                ] />
                <Show
                    when=medication
                    fallback=move || view! {
                        <div class="field">
                            <label for="order-measure">{text::ORDER_MEASURE_TEXT}</label>
                            <input id="order-measure" type="text" autocomplete="off" bind:value=measure />
                        </div>
                    }
                >
                    <div class="field">
                        <label for="order-substance">{text::ORDER_SUBSTANCE}</label>
                        <input id="order-substance" type="text" autocomplete="off" bind:value=substance />
                    </div>
                    <div class="field">
                        <label for="order-dose">{text::ORDER_DOSE}</label>
                        <input id="order-dose" type="text" autocomplete="off" bind:value=dose />
                    </div>
                    <div class="field">
                        <label for="order-route">{text::ORDER_ROUTE}</label>
                        <select id="order-route" on:change=move |ev| route.set(event_target_value(&ev))
                            prop:value=move || route.get()>
                            <option value="">{text::ORDER_ROUTE_CHOOSE}</option>
                            {CATALOGUE
                                .routes
                                .iter()
                                .map(|r| {
                                    let shown = match &r.description {
                                        Some(d) => format!("{} – {d}", r.label),
                                        None => r.label.clone(),
                                    };
                                    view! { <option value=r.id.clone()>{shown}</option> }
                                })
                                .collect_view()}
                        </select>
                    </div>
                </Show>
                <div class="field">
                    <label for="order-note">{text::ORDER_NOTE}</label>
                    <input id="order-note" type="text" autocomplete="off" bind:value=note />
                </div>
                <p class="hint">{text::ORDER_RECORDS_ONLY}</p>
                <Show when=spoken>
                    <p class="hint">{text::ORDER_SPOKEN_HINT}</p>
                </Show>
                <p class="error" role="alert">{move || message.get()}</p>
                <div class="row">
                    <button type="submit" class="button button--primary">
                        {move || if spoken() { text::ORDER_RECORD } else { text::ORDER_GIVE }}
                    </button>
                </div>
            </form>
        </section>
    }
}

// [impl->dsn~medical-orders~2]
/// The open orders of every UHS this person may read, oldest first (ORDER-4).
#[component]
pub fn OrdersView(core: Handle, revision: RwSignal<u64>, nav: Nav) -> impl IntoView {
    // Suggestions for who carried an order out.
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    provide_context(IncidentPeople(people.into(), core));
    let open = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            let events = c.events(med::orders::COMPARTMENT);
            let register = PatientRegister::from_events(&events);
            OrderBook::from_events(&events)
                .open()
                .into_iter()
                .map(|o| {
                    let uhs = register
                        .patient(&o.given.tag)
                        .map(|p| p.uhs.clone())
                        .unwrap_or_default();
                    (o.clone(), uhs)
                })
                .collect::<Vec<_>>()
        })
    });
    view! {
        <section aria-labelledby="orders-title" class="card">
            <h2 id="orders-title">{text::ORDERS_TITLE}</h2>
            {move || {
                let list = open.get();
                if list.is_empty() {
                    return view! { <p>{text::ORDERS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="order-list" data-keys="list">
                        {list
                            .into_iter()
                            .map(|(order, uhs)| {
                                let patient = (order.given.tag.clone(), core::uhs_name(core, &uhs));
                                view! { <OrderItem core revision order patient nav /> }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}
