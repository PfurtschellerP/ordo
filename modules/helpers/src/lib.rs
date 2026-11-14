//! Shared module `helpers`: the helpers of an incident (HELP-1, HELP-2), kept in the
//! compartment `people`, which only the incident command reads (ADR-0071).
//!
//! A helper is registered with a name and, where known, date of birth, address (street,
//! postcode, town), phone, organisation, qualifications, diet, duty and ORDO account. The
//! profile's settings ([`register::RegisterSettings`]) check phone and postcode and offer
//! the diets. Changes carry only the changed
//! fields; the latest value per field counts by hybrid logical clock, so the register does
//! not depend on the order in which events arrive.

pub mod cards;
pub mod register;

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "helpers";
/// Compartment of the helpers' personal data (ADR-0071).
pub const COMPARTMENT: &str = "people";
/// Event type: a helper was registered.
pub const REGISTERED: &str = "helpers.registered";
/// Event type: fields of a helper changed.
pub const UPDATED: &str = "helpers.updated";
/// Schema version of both event types. Version 2 adds the diet, version 3 the address as
/// street, postcode and town (HELP-1); older versions decode without them.
pub const SCHEMA: u32 = 3;
/// Longest text of a field, in characters.
pub const TEXT_MAX: usize = 200;

/// What is known of a helper. In `helpers.updated` only the changed fields are set; an
/// empty text, an empty list or a time of 0 clears a field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fields {
    /// Name as the helper is called, e.g. "Anna Schmidt".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Date of birth, `YYYY-MM-DD`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    /// Postal address in one text, as schema versions 1 and 2 kept it; shown where street,
    /// postcode and town are not given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Street and house number (schema version 3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    /// Postcode (schema version 3), checked against the profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,
    /// Town (schema version 3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub town: Option<String>,
    /// Phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Organisation, e.g. "DRK Musterstadt".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organisation: Option<String>,
    /// Qualification IDs from the profile (`units/catalogue.toml`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualifications: Option<Vec<String>>,
    /// Diet, for catering on site, e.g. "vegetarisch" (schema version 2). Can say something
    /// about health; it stays in `people` like everything else here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diet: Option<String>,
    /// Start of duty (ms since the Unix epoch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_start: Option<u64>,
    /// End of duty (ms since the Unix epoch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_end: Option<u64>,
    /// The helper's ORDO account (HELP-2), as a user ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

/// Payload of `helpers.registered` and `helpers.updated`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperChange {
    /// Helper ID (ULID), chosen when the helper is registered.
    pub helper: String,
    /// The fields given or changed.
    pub fields: Fields,
}

/// Why a change is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldError {
    /// A registration without a name, or a change that clears it.
    NameMissing,
    /// A text longer than [`TEXT_MAX`].
    TooLong,
    /// A date of birth not in the form `YYYY-MM-DD`.
    NotADate,
    /// An account that is not a user ID.
    NotAnAccount,
}

fn is_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && (1..=12).contains(&text[5..7].parse::<u8>().unwrap_or(0))
        && (1..=31).contains(&text[8..10].parse::<u8>().unwrap_or(0))
}

impl HelperChange {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }

    /// Checks the change; a registration needs a name.
    pub fn check(&self, registration: bool) -> Result<(), FieldError> {
        let f = &self.fields;
        let name = f.name.as_deref().map(str::trim);
        if (registration && name.is_none_or(str::is_empty)) || name == Some("") {
            return Err(FieldError::NameMissing);
        }
        let texts = [
            &f.name,
            &f.birth_date,
            &f.address,
            &f.street,
            &f.postcode,
            &f.town,
            &f.phone,
            &f.organisation,
            &f.diet,
        ];
        let long = texts
            .iter()
            .filter_map(|t| t.as_deref())
            .chain(f.qualifications.iter().flatten().map(String::as_str))
            .any(|t| t.chars().count() > TEXT_MAX);
        if long {
            return Err(FieldError::TooLong);
        }
        if let Some(d) = f.birth_date.as_deref().map(str::trim)
            && !d.is_empty()
            && !is_date(d)
        {
            return Err(FieldError::NotADate);
        }
        if let Some(a) = f.account.as_deref().map(str::trim)
            && !a.is_empty()
            && a.parse::<UserId>().is_err()
        {
            return Err(FieldError::NotAnAccount);
        }
        Ok(())
    }
}

/// A helper as the register knows them now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Helper {
    /// Helper ID.
    pub id: String,
    /// The current fields; cleared ones are `None`.
    pub fields: Fields,
    /// When and by whom the helper was registered.
    pub registered: (Hlc, UserId),
}

impl Helper {
    /// The name (always set for a registered helper).
    pub fn name(&self) -> &str {
        self.fields.name.as_deref().unwrap_or_default()
    }

    /// The address in one line: street, postcode and town, else as older versions kept it.
    pub fn address(&self) -> Option<String> {
        let f = &self.fields;
        let place = [f.postcode.as_deref(), f.town.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let line = [
            f.street.as_deref(),
            (!place.is_empty()).then_some(place.as_str()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");
        if line.is_empty() {
            f.address.clone()
        } else {
            Some(line)
        }
    }

    /// Whether the helper is on duty at `now_ms`: duty started and not ended.
    pub fn on_duty(&self, now_ms: u64) -> bool {
        self.fields.duty_start.is_some_and(|s| s <= now_ms)
            && self.fields.duty_end.is_none_or(|e| e > now_ms)
    }
}

/// Sets `into` to `value` where it is given; empty clears.
fn take<T>(into: &mut Option<T>, value: Option<T>, empty: impl Fn(&T) -> bool) {
    if let Some(v) = value {
        *into = (!empty(&v)).then_some(v);
    }
}

fn apply(into: &mut Fields, change: Fields) {
    let text = |t: &String| t.trim().is_empty();
    let trimmed = |t: Option<String>| t.map(|t| t.trim().to_owned());
    take(&mut into.name, trimmed(change.name), text);
    take(&mut into.birth_date, trimmed(change.birth_date), text);
    take(&mut into.address, trimmed(change.address), text);
    take(&mut into.street, trimmed(change.street), text);
    take(&mut into.postcode, trimmed(change.postcode), text);
    take(&mut into.town, trimmed(change.town), text);
    take(&mut into.phone, trimmed(change.phone), text);
    take(&mut into.organisation, trimmed(change.organisation), text);
    take(&mut into.diet, trimmed(change.diet), text);
    take(
        &mut into.qualifications,
        change.qualifications,
        Vec::is_empty,
    );
    take(&mut into.duty_start, change.duty_start, |t| *t == 0);
    take(&mut into.duty_end, change.duty_end, |t| *t == 0);
    take(&mut into.account, trimmed(change.account), text);
}

/// The helpers of an incident, computed from the events of `people`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HelperRegister {
    /// Registered helpers by ID.
    pub helpers: BTreeMap<String, Helper>,
}

impl HelperRegister {
    // [impl->dsn~helper-register~3]
    /// Computes the register; the result does not depend on the order of `events`. A
    /// helper exists once registered; changes apply in clock order, also those made before
    /// the registration arrived.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, bool, UserId, HelperChange)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let registration = match e.body.event_type.as_str() {
                    REGISTERED => true,
                    UPDATED => false,
                    _ => return None,
                };
                let change = codec::from_cbor::<HelperChange>(&e.payload).ok()?;
                change.check(registration).ok()?;
                Some((e.body.hlc, registration, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(hlc, ..)| *hlc);
        let mut fields: BTreeMap<String, Fields> = BTreeMap::new();
        let mut registered: BTreeMap<String, (Hlc, UserId)> = BTreeMap::new();
        for (hlc, registration, by, change) in changes {
            if registration {
                registered.entry(change.helper.clone()).or_insert((hlc, by));
            }
            apply(fields.entry(change.helper).or_default(), change.fields);
        }
        let helpers = registered
            .into_iter()
            .filter_map(|(id, registered)| {
                let fields = fields.remove(&id)?;
                fields.name.is_some().then(|| {
                    (
                        id.clone(),
                        Helper {
                            id,
                            fields,
                            registered,
                        },
                    )
                })
            })
            .collect();
        Self { helpers }
    }

    /// The helpers sorted by name.
    pub fn by_name(&self) -> Vec<&Helper> {
        let mut list: Vec<&Helper> = self.helpers.values().collect();
        list.sort_by_key(|h| (h.name().to_lowercase(), h.id.clone()));
        list
    }

    /// The helper linked to the account `user`, if any (HELP-2).
    pub fn of_account(&self, user: &str) -> Option<&Helper> {
        self.helpers
            .values()
            .find(|h| h.fields.account.as_deref() == Some(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, IncidentId, ServerName, Ulid};

    fn event(n: u64, event_type: &str, change: &HelperChange) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().expect("valid server");
        let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        AppliedEvent {
            seq: Some(n),
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(n, u128::from(n)).expect("valid")),
                schema_version: SCHEMA,
                hlc: Hlc {
                    wall_ms: 1000 + n,
                    counter: 0,
                    node: device,
                },
                incident: IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server),
                module: MODULE.parse().expect("valid"),
                compartment: COMPARTMENT.parse().expect("valid"),
                event_type: event_type.parse().expect("valid"),
                author: Author {
                    user: "ben@ordo.example.org".parse().expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload: change.encode().expect("encodes"),
        }
    }

    fn change(helper: &str, fields: Fields) -> HelperChange {
        HelperChange {
            helper: helper.into(),
            fields,
        }
    }

    // [utest->dsn~helper-register~3]
    #[test]
    fn helpers_are_registered_and_changed_field_by_field_in_any_order() {
        let anna = change(
            "h1",
            Fields {
                name: Some(" Anna Schmidt ".into()),
                phone: Some("0170 123".into()),
                qualifications: Some(vec!["paramedic".into()]),
                diet: Some("vegetarisch".into()),
                duty_start: Some(5000),
                ..Fields::default()
            },
        );
        let moved = change(
            "h1",
            Fields {
                address: Some("Hauptstr. 1".into()),
                phone: Some(String::new()),
                account: Some("anna@ordo.example.org".into()),
                ..Fields::default()
            },
        );
        let ben = change(
            "h2",
            Fields {
                name: Some("Ben".into()),
                ..Fields::default()
            },
        );
        let events = vec![
            event(3, UPDATED, &moved),
            event(1, REGISTERED, &anna),
            event(2, REGISTERED, &ben),
        ];
        let register = HelperRegister::from_events(&events);
        let a = &register.helpers["h1"];
        assert_eq!(a.name(), "Anna Schmidt", "trimmed");
        assert_eq!(a.fields.phone, None, "an empty field clears it");
        assert_eq!(a.fields.address.as_deref(), Some("Hauptstr. 1"));
        assert_eq!(a.fields.qualifications, Some(vec!["paramedic".to_owned()]));
        assert_eq!(a.fields.diet.as_deref(), Some("vegetarisch"));
        assert!(a.on_duty(6000) && !a.on_duty(4000));
        assert_eq!(
            register
                .of_account("anna@ordo.example.org")
                .map(|h| h.id.as_str()),
            Some("h1")
        );
        let names: Vec<&str> = register.by_name().iter().map(|h| h.name()).collect();
        assert_eq!(names, ["Anna Schmidt", "Ben"]);
        let mut reversed = events.clone();
        reversed.reverse();
        assert_eq!(HelperRegister::from_events(&reversed), register);
        // Ended duty; a change for an unknown helper registers nobody.
        let ended = change(
            "h1",
            Fields {
                duty_end: Some(7000),
                ..Fields::default()
            },
        );
        let ghost = change(
            "h9",
            Fields {
                name: Some("Niemand".into()),
                ..Fields::default()
            },
        );
        let later = HelperRegister::from_events(
            &[
                events,
                vec![event(4, UPDATED, &ended), event(5, UPDATED, &ghost)],
            ]
            .concat(),
        );
        assert!(!later.helpers["h1"].on_duty(8000));
        assert!(!later.helpers.contains_key("h9"));
    }

    #[test]
    fn changes_are_checked() {
        let with = |fields: Fields| change("h1", fields);
        assert_eq!(
            with(Fields::default()).check(true),
            Err(FieldError::NameMissing)
        );
        assert_eq!(with(Fields::default()).check(false), Ok(()));
        let named = |f: Fields| {
            with(Fields {
                name: Some("Anna".into()),
                ..f
            })
        };
        assert_eq!(
            named(Fields {
                birth_date: Some("1990-02-30".into()),
                ..Fields::default()
            })
            .check(true),
            Ok(()),
            "the form, not the calendar"
        );
        assert_eq!(
            named(Fields {
                birth_date: Some("1.2.1990".into()),
                ..Fields::default()
            })
            .check(true),
            Err(FieldError::NotADate)
        );
        assert_eq!(
            named(Fields {
                address: Some("x".repeat(TEXT_MAX + 1)),
                ..Fields::default()
            })
            .check(true),
            Err(FieldError::TooLong)
        );
        assert_eq!(
            named(Fields {
                diet: Some("x".repeat(TEXT_MAX + 1)),
                ..Fields::default()
            })
            .check(true),
            Err(FieldError::TooLong)
        );
        assert_eq!(
            named(Fields {
                account: Some("not an account".into()),
                ..Fields::default()
            })
            .check(true),
            Err(FieldError::NotAnAccount)
        );
        assert_eq!(
            with(Fields {
                name: Some(" ".into()),
                ..Fields::default()
            })
            .check(false),
            Err(FieldError::NameMissing),
            "the name is never cleared"
        );
    }
}
