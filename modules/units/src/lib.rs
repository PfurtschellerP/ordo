//! Shared module `units` (prototype: unit status only).
//!
//! Unit status uses *last writer wins* by hybrid logical clock. Each change records which
//! change it replaces, so ORDO can also tell when a change was made without knowing the
//! previous one — a lost update — and show it instead of hiding it (requirement OFF-4).

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "units";
/// Compartment of unit events.
pub const COMPARTMENT: &str = "incident";
/// Event type of a status change.
pub const STATUS_CHANGED: &str = "units.statusChanged";

/// Payload of `units.statusChanged`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusChanged {
    /// Call sign of the unit.
    pub unit: String,
    /// New status code (from the profile, e.g. "3").
    pub status: String,
    /// Timestamp of the status change this one replaces, as known to the author.
    pub replaces: Option<Hlc>,
}

impl StatusChanged {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Current status of a unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitStatus {
    /// Status code.
    pub status: String,
    /// When it was set.
    pub since: Hlc,
    /// Who set it.
    pub by: UserId,
}

/// A status change that was made without knowing the previous one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LostUpdate {
    /// Call sign.
    pub unit: String,
    /// The status that was overwritten without being seen.
    pub overwritten: String,
    /// The status that won.
    pub kept: String,
    /// Who made the winning change.
    pub by: UserId,
}

/// Status of all units, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusBoard {
    /// Current status per call sign.
    pub units: BTreeMap<String, UnitStatus>,
    /// Lost updates, to be shown in the log.
    pub lost_updates: Vec<LostUpdate>,
}

impl StatusBoard {
    // [impl->dsn~lost-update-detection~1]
    /// Computes the board. The result doesn't depend on the order of `events`.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: BTreeMap<String, Vec<(Hlc, UserId, StatusChanged)>> = BTreeMap::new();
        for event in events {
            if event.body.module.as_str() != MODULE
                || event.body.event_type.as_str() != STATUS_CHANGED
            {
                continue;
            }
            if let Ok(change) = codec::from_cbor::<StatusChanged>(&event.payload) {
                changes.entry(change.unit.clone()).or_default().push((
                    event.body.hlc,
                    event.body.author.user.clone(),
                    change,
                ));
            }
        }
        let mut board = Self::default();
        for (unit, mut list) in changes {
            list.sort_by_key(|(hlc, _, _)| *hlc);
            let mut previous: Option<(Hlc, String)> = None;
            for (hlc, user, change) in list {
                if let Some((prev_hlc, prev_status)) = &previous
                    && change.replaces != Some(*prev_hlc)
                {
                    board.lost_updates.push(LostUpdate {
                        unit: unit.clone(),
                        overwritten: prev_status.clone(),
                        kept: change.status.clone(),
                        by: user.clone(),
                    });
                }
                previous = Some((hlc, change.status.clone()));
                board.units.insert(
                    unit.clone(),
                    UnitStatus {
                        status: change.status,
                        since: hlc,
                        by: user,
                    },
                );
            }
        }
        board
    }
}
