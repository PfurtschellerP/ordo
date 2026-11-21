//! A card number registered twice is changed by whoever registered it later (PAT-12,
//! ADR-0092): `patients.retagged` names that registration and the new number. Events are
//! never changed; they are read again: the registration, and everything the same device
//! recorded about the old number from the registration until the change, are read as
//! about the new number.

use std::borrow::Cow;

use ciborium::Value as Cbor;
use ordo_core::{DeviceId, Hlc, codec};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

use crate::{COMPARTMENT, MODULE, REGISTERED, RETAGGED};

/// Payload of `patients.retagged` (clinical compartment, schema version 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retagged {
    /// The card number registered twice.
    pub tag: String,
    /// The later registration's event ID: the patient that gets the new number.
    pub registration: String,
    /// The number of the new card.
    pub new_tag: String,
}

impl Retagged {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, codec::CodecError> {
        codec::to_cbor(self)
    }
}

/// What moves to a new number: events of `device` about `old`, from `from` to `until`.
struct Rule {
    old: String,
    new: String,
    device: DeviceId,
    from: Hlc,
    until: Hlc,
}

/// The `tag` of a payload changed to `new`, if it has one at its top.
fn with_tag(payload: &[u8], new: &str) -> Option<Vec<u8>> {
    let mut value: Cbor = codec::from_cbor(payload).ok()?;
    let Cbor::Map(entries) = &mut value else {
        return None;
    };
    let entry = entries
        .iter_mut()
        .find(|(k, _)| k.as_text() == Some("tag"))?;
    entry.1 = Cbor::Text(new.to_owned());
    codec::to_cbor(&value).ok()
}

/// The `tag` at the top of a payload, if any.
fn tag_of(payload: &[u8]) -> Option<String> {
    #[derive(Deserialize)]
    struct Tagged {
        tag: Option<String>,
    }
    codec::from_cbor::<Tagged>(payload).ok()?.tag
}

// [impl->dsn~unique-triage-tag~2]
/// The clinical events as read after the number changes they contain: unchanged (and not
/// copied) when there are none.
pub fn read(events: &[AppliedEvent]) -> Cow<'_, [AppliedEvent]> {
    let is = |e: &AppliedEvent, kind: &str| {
        e.body.module.as_str() == MODULE && e.body.event_type.as_str() == kind
    };
    let rules: Vec<Rule> = events
        .iter()
        .filter(|e| is(e, RETAGGED))
        .filter_map(|e| {
            let r: Retagged = codec::from_cbor(&e.payload).ok()?;
            let registration = events
                .iter()
                .find(|x| is(x, REGISTERED) && x.body.id.to_string() == r.registration)?;
            Some(Rule {
                old: r.tag,
                new: r.new_tag,
                device: registration.body.author.device,
                from: registration.body.hlc,
                until: e.body.hlc,
            })
        })
        .collect();
    if rules.is_empty() {
        return Cow::Borrowed(events);
    }
    Cow::Owned(
        events
            .iter()
            .map(|e| {
                let moved = (e.body.compartment.as_str() == COMPARTMENT && !is(e, RETAGGED))
                    .then(|| {
                        let tag = tag_of(&e.payload)?;
                        rules.iter().find(|r| {
                            r.old == tag
                                && r.device == e.body.author.device
                                && r.from <= e.body.hlc
                                && e.body.hlc <= r.until
                        })
                    })
                    .flatten();
                match moved.and_then(|r| with_tag(&e.payload, &r.new)) {
                    Some(payload) => AppliedEvent {
                        payload,
                        ..e.clone()
                    },
                    None => e.clone(),
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ordo_contracts::TriageCategory;
    use ordo_core::{DeviceId, Ulid};

    use super::*;
    use crate::record::{self, EntryRecorded, Records, Value};
    use crate::{PatientRegister, Registered, testing};

    fn on(device: u64, mut e: AppliedEvent) -> AppliedEvent {
        e.body.author.device = DeviceId(Ulid::from_parts(device, 1).expect("valid"));
        e
    }

    fn registered(n: u64, user: &str, uhs: &str, category: TriageCategory) -> AppliedEvent {
        let r = Registered {
            report: None,
            unit: None,
            task: None,
            tag: "DE-0042".into(),
            uhs: uhs.into(),
            category,
            algorithm: None,
        };
        testing::event(
            n,
            user,
            MODULE,
            COMPARTMENT,
            REGISTERED,
            r.encode().unwrap(),
        )
    }

    fn pulse(n: u64, user: &str, value: &str) -> AppliedEvent {
        let e = EntryRecorded {
            tag: "DE-0042".into(),
            form: "vitals".into(),
            values: BTreeMap::from([("heart_rate".to_owned(), Value::One(value.into()))]),
            replaces: None,
            order: None,
        };
        testing::event(
            n,
            user,
            record::MODULE,
            COMPARTMENT,
            record::ENTRY_RECORDED,
            e.encode().unwrap(),
        )
    }

    // [utest->dsn~unique-triage-tag~2]
    #[test]
    fn the_later_registration_gets_the_new_number_with_what_its_device_documented() {
        // Two devices offline register the same card; each documents a pulse.
        let anna = on(
            1,
            registered(1, "anna", "eingang", TriageCategory::Immediate),
        );
        let ben = on(2, registered(2, "ben", "nord", TriageCategory::Minor));
        let mut events = vec![
            anna,
            ben.clone(),
            on(1, pulse(3, "anna", "120")),
            on(2, pulse(4, "ben", "80")),
        ];
        let register = PatientRegister::from_events(&events);
        let again = &register.patient("DE-0042").unwrap().registered_again;
        assert_eq!(again.len(), 1, "a duplicate");
        assert_eq!(again[0].event, ben.body.id.to_string());

        // Ben gives his patient a new card.
        let change = Retagged {
            tag: "DE-0042".into(),
            registration: again[0].event.clone(),
            new_tag: "DE-0099".into(),
        };
        events.push(on(
            2,
            testing::event(
                5,
                "ben",
                MODULE,
                COMPARTMENT,
                RETAGGED,
                change.encode().unwrap(),
            ),
        ));
        // Anna documents her patient afterwards, from her device.
        events.push(on(1, pulse(6, "anna", "110")));
        let register = PatientRegister::from_events(&events);
        let first = register.patient("DE-0042").unwrap();
        assert!(first.registered_again.is_empty(), "no duplicate any more");
        assert_eq!(first.category(), TriageCategory::Immediate);
        assert_eq!(first.uhs, "eingang");
        let second = register.patient("DE-0099").expect("the new number");
        assert_eq!(second.category(), TriageCategory::Minor);
        assert_eq!(second.uhs, "nord");

        let records = Records::from_events(&events);
        let pulses = |tag: &str| -> Vec<String> {
            records
                .get(tag)
                .map(|r| {
                    r.current_entries("vitals")
                        .filter_map(|e| match e.values.get("heart_rate") {
                            Some(Value::One(v)) => Some(v.clone()),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        assert_eq!(pulses("DE-0042"), ["120", "110"], "Anna's stay");
        assert_eq!(pulses("DE-0099"), ["80"], "Ben's moved");
    }
}
