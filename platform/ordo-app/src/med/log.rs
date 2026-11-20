//! The Med package's part of the incident log (LOG-1, LOG-6): its events in words, and the
//! names of UHS, places, orders, transports and hospitals they refer to. Clinical entries
//! appear only on devices that read the clinical compartment.

use ordo_med_patients as med;
use ordo_sync::AppliedEvent;

use crate::shared::log::{Line, LogPart, Names, Naming, decode};
use crate::text;

// [impl->dsn~incident-log~1]
/// The Med package's part of the log.
pub(crate) const PART: LogPart = LogPart { learn, describe };

fn learn(e: &AppliedEvent, c: &mut Names) {
    match e.body.event_type.as_str() {
        med::UHS_CREATED => {
            if let Some(p) = decode::<med::UhsCreated>(e) {
                c.set("uhs", p.id, p.name);
            }
        }
        med::UHS_UPDATED => {
            if let Some(med::UhsUpdated {
                id,
                name: Some(name),
                ..
            }) = decode(e)
            {
                c.set("uhs", id, name);
            }
        }
        med::UHS_PLACES => {
            if let Some(p) = decode::<med::PlacesSet>(e) {
                for place in p.places {
                    c.set("place", place.id, place.label);
                }
            }
        }
        med::orders::GIVEN => {
            if let Some(p) = decode::<med::orders::OrderGiven>(e) {
                c.set("order", p.order, p.tag);
            }
        }
        med::orders::RECORDED => {
            if let Some(p) = decode::<med::orders::OrderRecorded>(e) {
                c.set("order", p.order, p.tag);
            }
        }
        med::transport::REQUESTED => {
            if let Some(p) = decode::<med::transport::TransportRequested>(e) {
                c.set("transport.type", p.request.clone(), p.transport_type);
                c.set("transport.uhs", p.request, p.uhs);
            }
        }
        med::hospitals::SET => {
            if let Some(p) = decode::<med::hospitals::HospitalSet>(e) {
                c.set("hospital", p.hospital, p.name);
            }
        }
        _ => {}
    }
}

/// A transport as the log names it: its type and its UHS.
fn transport(c: &Names, id: &str) -> String {
    let kind = c.get("transport.type", id);
    if kind == id {
        return id.to_owned();
    }
    text::log_transport(kind, c.get("uhs", c.get("transport.uhs", id)))
}

fn describe(e: &AppliedEvent, c: &Names, name: Naming) -> Option<Line> {
    Some(match e.body.event_type.as_str() {
        // [impl->dsn~uhs-staff-request~2]
        med::staffing::REQUESTED => {
            let p = decode::<med::staffing::Requested>(e)?;
            text::staff_requested(c.get("uhs", &p.uhs), p.need).into()
        }
        med::staffing::CLOSED => {
            let p = decode::<med::staffing::Closed>(e)?;
            match p.person {
                Some(person) => text::staff_request_met(&name(&person)).into(),
                None => text::STAFF_REQUEST_WITHDRAWN.to_owned().into(),
            }
        }
        med::UHS_CREATED => text::uhs_created(&decode::<med::UhsCreated>(e)?.name).into(),
        med::UHS_UPDATED => {
            let p = decode::<med::UhsUpdated>(e)?;
            match p.name {
                Some(name) => text::uhs_renamed(&name).into(),
                None => text::uhs_updated(c.get("uhs", &p.id)).into(),
            }
        }
        med::UHS_CLOSED => text::uhs_closed(c.get("uhs", &decode::<med::UhsClosed>(e)?.id)).into(),
        med::UHS_PLACES => {
            let p = decode::<med::PlacesSet>(e)?;
            text::log_places(c.get("uhs", &p.uhs), p.places.len()).into()
        }
        med::REGISTERED => {
            let p = decode::<med::Registered>(e)?;
            text::registered(&p.tag, p.category, c.get("uhs", &p.uhs)).into()
        }
        med::RETRIAGED => {
            let p = decode::<med::Retriaged>(e)?;
            text::retriaged(&p.tag, p.category).into()
        }
        med::MOVED => {
            let p = decode::<med::Moved>(e)?;
            text::patient_moved(&p.tag, c.get("uhs", &p.uhs)).into()
        }
        med::DISCHARGED => {
            let p = decode::<med::Discharged>(e)?;
            text::discharged(&p.tag, p.outcome).into()
        }
        med::READMITTED => text::readmitted(&decode::<med::Readmitted>(e)?.tag).into(),
        med::PLACE_ASSIGNED => {
            let p = decode::<med::PlaceAssigned>(e)?;
            let place = p.place.as_deref().map(|id| c.get("place", id));
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
            text::log_transport_requested(&transport(c, &p.request)).into()
        }
        med::transport::UPDATED => {
            let p = decode::<med::transport::TransportUpdated>(e)?;
            text::log_transport_updated(
                &transport(c, &p.request),
                text::transport_status(p.status),
                p.unit.as_deref(),
            )
            .into()
        }
        med::orders::GIVEN => text::order_given(&decode::<med::orders::OrderGiven>(e)?.tag).into(),
        med::orders::RECORDED => {
            text::order_recorded(&decode::<med::orders::OrderRecorded>(e)?.tag).into()
        }
        med::orders::CONFIRMED => {
            let p = decode::<med::orders::OrderConfirmed>(e)?;
            text::log_order_confirmed(c.get("order", &p.order)).into()
        }
        med::orders::UPDATED => {
            let p = decode::<med::orders::OrderUpdated>(e)?;
            let tag = c.get("order", &p.order);
            let status = match &p.performed_by {
                Some(by) => text::order_status_performed(text::order_status(p.status), by),
                None => text::order_status(p.status).to_owned(),
            };
            text::log_order_updated(tag, &status).into()
        }
        med::record::ENTRY_RECORDED => {
            let p = decode::<med::record::EntryRecorded>(e)?;
            let form = crate::med::patients::FORMS
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
        med::record::PHOTO_REMOVED => {
            text::photo_removed(&decode::<med::record::PhotoRemoved>(e)?.tag).into()
        }
        med::hospitals::SET => {
            text::hospital_saved(&decode::<med::hospitals::HospitalSet>(e)?.name).into()
        }
        med::hospitals::CAPACITY_SET => {
            let p = decode::<med::hospitals::CapacitySet>(e)?;
            text::capacity_saved(c.get("hospital", &p.hospital)).into()
        }
        ordo_contracts::PatientCountsV1::EVENT_TYPE => {
            let p = decode::<ordo_contracts::PatientCountsV1>(e)?;
            let counts = crate::med::triage::CATEGORIES.map(|k| (text::label(k), p.get(k)));
            text::log_counts(c.get("uhs", &p.uhs), &counts, p.without_place, p.discharged).into()
        }
        _ => return None,
    })
}
