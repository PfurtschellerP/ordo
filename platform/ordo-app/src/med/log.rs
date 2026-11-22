//! The Med package's part of the incident log (LOG-1, LOG-6): its events in words, and the
//! names of UHS, places, orders, transports and hospitals they refer to. Clinical entries
//! appear only on devices that read the clinical compartment.

use ordo_med_patients as med;
use ordo_sync::AppliedEvent;

use crate::med::text;
use crate::shared::log::{Describe, Line, LogPart, Names, decode};

// [impl->dsn~incident-log~1]
/// The Med package's part of the log.
pub(crate) const PART: LogPart = LogPart {
    learn,
    describe: DESCRIBE,
};

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

/// A patient's holder as the log names it: the UHS's name, or the unit's call sign.
fn holder(c: &Names, h: &med::Holder) -> String {
    match h {
        med::Holder::Uhs(id) => c.get("uhs", id).to_owned(),
        med::Holder::Unit(unit) => unit.clone(),
    }
}

const DESCRIBE: &[(&str, Describe)] = &[
    // [impl->dsn~patient-handover~1]
    (med::HANDED_OVER, |e, c, _| {
        let p = decode::<med::HandedOver>(e)?;
        let words = match (&p.to, &p.outside) {
            (_, Some(receiver)) => text::handed_outside(&p.tag, receiver),
            (Some(to), None) => text::handed_over(&p.tag, holder(c, to)),
            (None, None) => text::handed_over_quick(&p.tag),
        };
        Some(words.into())
    }),
    // [impl->dsn~unique-triage-tag~2]
    (med::RETAGGED, |e, _, _| {
        let p = decode::<med::retag::Retagged>(e)?;
        Some(text::retagged(&p.tag, &p.new_tag).into())
    }),
    (med::TAKEN, |e, c, _| {
        let p = decode::<med::Taken>(e)?;
        Some(text::taken(&p.tag, holder(c, &p.holder)).into())
    }),
    // [impl->dsn~uhs-staff-request~2]
    (med::staffing::REQUESTED, |e, c, _| {
        Some({
            let p = decode::<med::staffing::Requested>(e)?;
            text::staff_requested(c.get("uhs", &p.uhs), p.need).into()
        })
    }),
    (med::staffing::CLOSED, |e, _, name| {
        Some({
            let p = decode::<med::staffing::Closed>(e)?;
            match p.person {
                Some(person) => text::staff_request_met(&name(&person)).into(),
                None => text::STAFF_REQUEST_WITHDRAWN.to_owned().into(),
            }
        })
    }),
    (med::UHS_CREATED, |e, _, _| {
        Some(text::uhs_created(&decode::<med::UhsCreated>(e)?.name).into())
    }),
    (med::UHS_UPDATED, |e, c, _| {
        Some({
            let p = decode::<med::UhsUpdated>(e)?;
            match p.name {
                Some(name) => text::uhs_renamed(&name).into(),
                None => text::uhs_updated(c.get("uhs", &p.id)).into(),
            }
        })
    }),
    (med::UHS_CLOSED, |e, c, _| {
        Some(text::uhs_closed(c.get("uhs", &decode::<med::UhsClosed>(e)?.id)).into())
    }),
    (med::UHS_PLACES, |e, c, _| {
        Some({
            let p = decode::<med::PlacesSet>(e)?;
            text::log_places(c.get("uhs", &p.uhs), p.places.len()).into()
        })
    }),
    (med::REGISTERED, |e, c, _| {
        Some({
            let p = decode::<med::Registered>(e)?;
            text::registered(&p.tag, p.category, &holder(c, &p.holder())).into()
        })
    }),
    (med::RETRIAGED, |e, _, _| {
        Some({
            let p = decode::<med::Retriaged>(e)?;
            text::retriaged(&p.tag, p.category).into()
        })
    }),
    (med::MOVED, |e, c, _| {
        Some({
            let p = decode::<med::Moved>(e)?;
            text::patient_moved(&p.tag, c.get("uhs", &p.uhs)).into()
        })
    }),
    (med::DISCHARGED, |e, _, _| {
        Some({
            let p = decode::<med::Discharged>(e)?;
            text::discharged(&p.tag, p.outcome).into()
        })
    }),
    (med::READMITTED, |e, _, _| {
        Some(text::readmitted(&decode::<med::Readmitted>(e)?.tag).into())
    }),
    (med::PLACE_ASSIGNED, |e, c, _| {
        Some({
            let p = decode::<med::PlaceAssigned>(e)?;
            let place = p.place.as_deref().map(|id| c.get("place", id));
            text::place_assigned(&p.tag, place).into()
        })
    }),
    (med::TRANSPORT_PRIORITY, |e, _, _| {
        Some({
            let p = decode::<med::TransportPrioritySet>(e)?;
            text::priority_changed(&p.tag, p.level.is_some()).into()
        })
    }),
    (med::transport::LINKED, |e, _, _| {
        Some({
            text::transport_requested(&decode::<med::transport::TransportLinked>(e)?.tag).into()
        })
    }),
    (med::transport::REQUESTED, |e, c, _| {
        Some({
            let p = decode::<med::transport::TransportRequested>(e)?;
            text::log_transport_requested(&transport(c, &p.request)).into()
        })
    }),
    (med::transport::UPDATED, |e, c, _| {
        Some({
            let p = decode::<med::transport::TransportUpdated>(e)?;
            // [impl->dsn~transport-request~2]
            // A step by the unit's status shows as it (ADR-0098), a destination set later
            // as the destination.
            let what = match (&p.status_code, &p.destination, p.unit.is_some()) {
                (Some(code), _, _) => crate::text::status_label(code),
                (None, Some(d), false) => text::destination_set(d),
                _ => text::transport_status(p.status).to_owned(),
            };
            text::log_transport_updated(&transport(c, &p.request), &what, p.unit.as_deref()).into()
        })
    }),
    (med::orders::GIVEN, |e, _, _| {
        Some(text::order_given(&decode::<med::orders::OrderGiven>(e)?.tag).into())
    }),
    (med::orders::RECORDED, |e, _, _| {
        Some(text::order_recorded(&decode::<med::orders::OrderRecorded>(e)?.tag).into())
    }),
    (med::orders::CONFIRMED, |e, c, _| {
        Some({
            let p = decode::<med::orders::OrderConfirmed>(e)?;
            text::log_order_confirmed(c.get("order", &p.order)).into()
        })
    }),
    (med::orders::UPDATED, |e, c, _| {
        Some({
            let p = decode::<med::orders::OrderUpdated>(e)?;
            let tag = c.get("order", &p.order);
            let status = match &p.performed_by {
                Some(by) => text::order_status_performed(text::order_status(p.status), by),
                None => text::order_status(p.status).to_owned(),
            };
            text::log_order_updated(tag, &status).into()
        })
    }),
    (med::record::ENTRY_RECORDED, |e, _, _| {
        Some({
            let p = decode::<med::record::EntryRecorded>(e)?;
            let form = crate::med::patients::FORMS
                .form(&p.form)
                .map_or(p.form.as_str(), |f| f.label.as_str());
            Line {
                text: text::entry_saved(form, &p.tag),
                corrects: p.replaces,
            }
        })
    }),
    (med::record::PHOTO_ATTACHED, |e, _, _| {
        Some(text::photo_added(&decode::<med::record::PhotoAttached>(e)?.tag).into())
    }),
    (med::record::PHOTO_REMOVED, |e, _, _| {
        Some(text::photo_removed(&decode::<med::record::PhotoRemoved>(e)?.tag).into())
    }),
    (med::hospitals::SET, |e, _, _| {
        Some(text::hospital_saved(&decode::<med::hospitals::HospitalSet>(e)?.name).into())
    }),
    (med::hospitals::CAPACITY_SET, |e, c, _| {
        Some({
            let p = decode::<med::hospitals::CapacitySet>(e)?;
            text::capacity_saved(c.get("hospital", &p.hospital)).into()
        })
    }),
    (ordo_contracts::PatientCountsV1::EVENT_TYPE, |e, c, _| {
        Some({
            let p = decode::<ordo_contracts::PatientCountsV1>(e)?;
            let counts = crate::med::triage::CATEGORIES.map(|k| (text::label(k), p.get(k)));
            text::log_counts(c.get("uhs", &p.uhs), &counts, p.without_place, p.discharged).into()
        })
    }),
];
