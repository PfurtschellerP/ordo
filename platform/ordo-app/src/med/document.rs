//! The handover document (ADR-0089, EPCR-20): the ORDO server makes it from the record and
//! the profile's template; the app asks for a ticket over the sync connection, fetches the
//! PDF with it and saves it. It needs a connection, as the product owner decided on
//! 2026-11-21.

use std::cell::RefCell;

use leptos::prelude::*;
use ordo_sync::message::{ClientMessage, DocumentRequest, DocumentTicket};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::core::Handle;
use crate::med::text;

/// How far the document is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Idle,
    Asking,
    Saved,
    Offline,
    Refused,
    Failed,
}

thread_local! {
    /// The state of the document asked for, by triage tag (one at a time).
    static WAITING: RefCell<Option<(String, RwSignal<State>)>> = const { RefCell::new(None) };
}

// [impl->dsn~handover-document~1]
/// Asks the server for the document of the patient `tag`.
fn ask(core: Handle, tag: &str, state: RwSignal<State>) {
    let Some(incident) = core.with_value(|c| c.incident.clone()) else {
        return;
    };
    if core.with_value(|c| c.connection) != crate::core::Connection::Online {
        state.set(State::Offline);
        return;
    }
    // The device's offset from UTC, for the times in the document.
    let offset = -(js_sys::Date::new_0().get_timezone_offset() as i32);
    WAITING.with_borrow_mut(|w| *w = Some((tag.to_owned(), state)));
    state.set(State::Asking);
    core.with_value(|c| {
        c.send(&ClientMessage::DocumentTicket(DocumentRequest {
            incident,
            tag: tag.to_owned(),
            utc_offset_minutes: offset,
        }));
    });
}

/// The server's answer: fetch and save the PDF with the ticket.
pub(crate) fn on_ticket(answer: DocumentTicket) {
    let Some((tag, state)) = WAITING.with_borrow_mut(|w| w.take_if(|(t, _)| *t == answer.tag))
    else {
        return;
    };
    let Some(ticket) = answer.ticket else {
        state.set(State::Refused);
        return;
    };
    leptos::task::spawn_local(async move {
        state.set(if save(&tag, &ticket).await.is_some() {
            State::Saved
        } else {
            State::Failed
        });
    });
}

async fn save(tag: &str, ticket: &str) -> Option<()> {
    let headers = crate::core::api_headers()?;
    headers
        .set("Authorization", &format!("Bearer {ticket}"))
        .ok()?;
    let init = web_sys::RequestInit::new();
    init.set_headers(&headers);
    let window = web_sys::window()?;
    let promise = window.fetch_with_str_and_init("/documents/handover", &init);
    let response: web_sys::Response = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let blob: web_sys::Blob = JsFuture::from(response.blob().ok()?)
        .await
        .ok()?
        .dyn_into()
        .ok()?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok()?;
    let a: web_sys::HtmlAnchorElement = window
        .document()?
        .create_element("a")
        .ok()?
        .dyn_into()
        .ok()?;
    a.set_href(&url);
    a.set_download(&text::document_file(tag));
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Some(())
}

// [impl->dsn~handover-document~1]
/// *Übergabedokument*: the PDF for the receiver, made on the server.
#[component]
pub(crate) fn HandoverDocument(core: Handle, tag: String) -> impl IntoView {
    let state = RwSignal::new(State::Idle);
    let label = text::document_label(&tag);
    view! {
        <section class="record__section document" aria-labelledby="section-document">
            <h3 id="section-document">{text::DOCUMENT_TITLE}</h3>
            <p class="hint" id="document-hint">{text::DOCUMENT_HINT}</p>
            <div class="actions">
                <button id="document-button" type="button" class="button button--with-icon" aria-describedby="document-hint document-state"
                    aria-label=label disabled=move || state.get() == State::Asking
                    on:click=move |_| ask(core, &tag, state)>
                    <crate::shell::ui::Icon name="file-down" />{text::DOCUMENT_BUTTON}
                </button>
            </div>
            <p id="document-state" role="status" class:error=move || matches!(state.get(), State::Offline | State::Refused | State::Failed)>
                {move || match state.get() {
                    State::Idle => "",
                    State::Asking => text::DOCUMENT_ASKING,
                    State::Saved => text::DOCUMENT_SAVED,
                    State::Offline => text::DOCUMENT_OFFLINE,
                    State::Refused => text::DOCUMENT_REFUSED,
                    State::Failed => text::DOCUMENT_FAILED,
                }}
            </p>
        </section>
    }
}
