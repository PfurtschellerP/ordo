//! The handover document (ADR-0089, EPCR-20): a device asks for a ticket over its sync
//! connection (`documentTicket`, protocol 1.5), then fetches `GET /documents/handover` with
//! it (HTTP API 1.2). The server decrypts the incident's clinical events, logs that it did
//! (ADR-0014), and makes the PDF from the profile's Typst template.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use ordo_core::{CompartmentId, IncidentId, UserId};
use ordo_med_handover::{Events, Naming, Profile};
use ordo_sync::message::DocumentRequest;

use crate::app::{App, now_ms};

/// How long a ticket is good for; it is used once.
const TICKET_TTL: Duration = Duration::from_secs(120);
/// At most this many tickets wait at once; the oldest goes first.
const MAX_TICKETS: usize = 1000;

struct Ticket {
    user: UserId,
    request: DocumentRequest,
    expires: Instant,
}

/// The tickets given out, and the profile the documents are made from.
#[derive(Default)]
pub(crate) struct Documents {
    tickets: Mutex<HashMap<String, Ticket>>,
    profile: OnceLock<Option<Arc<Profile>>>,
}

impl Documents {
    // [impl->dsn~handover-document~1]
    /// A ticket for one fetch of the document `request` names, by `user`.
    pub(crate) fn issue(&self, user: UserId, request: DocumentRequest) -> String {
        let mut token = [0u8; 32];
        getrandom::fill(&mut token).expect("the operating system provides randomness");
        let token = crate::blobs::hex(&token);
        let now = Instant::now();
        let mut tickets = self.tickets.lock().expect("ticket lock");
        tickets.retain(|_, t| t.expires > now);
        if tickets.len() >= MAX_TICKETS
            && let Some(oldest) = tickets
                .iter()
                .min_by_key(|(_, t)| t.expires)
                .map(|(k, _)| k.clone())
        {
            tickets.remove(&oldest);
        }
        tickets.insert(
            token.clone(),
            Ticket {
                user,
                request,
                expires: now + TICKET_TTL,
            },
        );
        token
    }

    /// The ticket named in `headers`, used up.
    fn take(&self, headers: &HeaderMap) -> Option<Ticket> {
        let token = headers
            .get(header::AUTHORIZATION)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;
        let ticket = self.tickets.lock().expect("ticket lock").remove(token)?;
        (ticket.expires > Instant::now()).then_some(ticket)
    }

    /// The profile the server was built with, read once.
    fn profile(&self) -> Option<Arc<Profile>> {
        self.profile
            .get_or_init(|| match Profile::embedded() {
                Ok(p) => Some(Arc::new(p)),
                Err(error) => {
                    tracing::error!(%error, "the handover profile cannot be used");
                    None
                }
            })
            .clone()
    }
}

/// The clinical compartment and the incident's shared one.
fn compartments() -> (CompartmentId, CompartmentId) {
    (
        ordo_med_patients::COMPARTMENT
            .parse()
            .expect("valid compartment"),
        ordo_med_patients::UHS_COMPARTMENT
            .parse()
            .expect("valid compartment"),
    )
}

/// Whether `user` may have documents of `incident`: they read its clinical compartment.
pub(crate) async fn may(app: &App, user: &UserId, incident: &IncidentId) -> bool {
    let (clinical, _) = compartments();
    app.hub
        .lock()
        .await
        .can_access(user, incident, &clinical, false)
}

// [impl->dsn~handover-document~1]
/// `GET /documents/handover`: the PDF the ticket names.
pub(crate) async fn handover(State(app): State<Arc<App>>, headers: HeaderMap) -> Response {
    let Some(ticket) = app.documents.take(&headers) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(profile) = app.documents.profile() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let (clinical, shared) = compartments();
    let request = ticket.request;
    let (clinical_events, incident_events) = {
        let hub = app.hub.lock().await;
        // Checked again: roles may have changed since the ticket.
        if !hub.can_access(&ticket.user, &request.incident, &clinical, false) {
            return StatusCode::FORBIDDEN.into_response();
        }
        (
            hub.opened_events(&request.incident, &clinical),
            hub.opened_events(&request.incident, &shared),
        )
    };
    // ADR-0014: every decryption of clinical data on the server is logged.
    tracing::info!(user = %ticket.user, incident = %request.incident, tag = %request.tag, "clinical data decrypted for a handover document");
    // The names of the people who documented, as they chose them (ORG-7).
    let people: HashSet<UserId> = clinical_events
        .iter()
        .map(|e| e.body.author.user.clone())
        .collect();
    let mut names = HashMap::new();
    for user in people {
        if let Ok(Some(name)) = app.store.display_name(&user).await {
            names.insert(user, name);
        }
    }
    let made = tokio::task::spawn_blocking(move || {
        let person = |u: &UserId| names.get(u).cloned().unwrap_or_else(|| u.to_string());
        let naming = Naming {
            offset_minutes: request.utc_offset_minutes.clamp(-14 * 60, 14 * 60),
            person: &person,
        };
        let events = Events {
            clinical: &clinical_events,
            incident: &incident_events,
        };
        ordo_med_handover::make(&profile, &events, &request.tag, now_ms(), &naming)
    })
    .await;
    match made {
        Ok(Ok(document)) => (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/pdf"),
                ),
                (
                    header::CONTENT_DISPOSITION,
                    HeaderValue::from_static("attachment"),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            ],
            document.pdf,
        )
            .into_response(),
        Ok(Err(ordo_med_handover::Error::UnknownPatient)) => StatusCode::NOT_FOUND.into_response(),
        Ok(Err(error)) => {
            tracing::error!(%error, "making a handover document failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(error) => {
            tracing::error!(%error, "making a handover document failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
