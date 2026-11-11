//! Medical orders (ORDER-1…4): given on the patient record by physicians and paramedics,
//! carried out by the crew, and listed across every UHS in the view *Anordnungen*.
//! ORDO records orders; it never proposes, checks or calculates them (EPCR-9).

use std::collections::HashMap;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_med_patients::orders::{
    Order, OrderBook, OrderCatalogue, OrderGiven, OrderStatus, OrderUpdated, Ordered,
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
        crate::profile::naming(move || {
            text::order_by(&at, &crate::profile::display_name(core, &user))
        })
    };
    // [impl->dsn~order-documentation~2]
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
    let done = order.done().cloned();
    let documentable = patient.is_none()
        && done.is_some()
        && matches!(order.given.ordered, Ordered::Medication { .. })
        && CATALOGUE.record.is_some();
    let document = documentable.then(|| {
        let (given, ordered_by, label) = (order.given.clone(), order.by().to_string(), text::document_order_label(&what));
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
                        open_prefilled(drafts, &given.tag, form, values, Some(at.clone()));
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
            {document}
        </li>
    }
}

// [impl->dsn~medical-orders~2]
/// The orders of one patient on the record; the form for those who may give orders.
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
    let message = RwSignal::new(None::<&'static str>);
    let submit = {
        let tag = tag.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
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
            if order.check(&CATALOGUE).is_err() {
                message.set(Some(text::ORDER_INCOMPLETE));
                return;
            }
            if core::give_order(core, revision, order).is_ok() {
                for s in [substance, dose, route, measure, note] {
                    s.set(String::new());
                }
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
            <Show when=move || can_give.get()>
                <form class="form order-form" novalidate on:submit=submit.clone()>
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
                    <p class="error" role="alert">{move || message.get()}</p>
                    <div class="row">
                        <button type="submit" class="button button--primary">{text::ORDER_GIVE}</button>
                    </div>
                </form>
            </Show>
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
