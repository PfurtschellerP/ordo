//! Hybrid logical clock (spec/sync/event.md §3).
//!
//! A hybrid logical clock gives every event a timestamp that is close to real time but
//! strictly increasing per device, even if the physical clock stands still or jumps
//! backwards, and that orders consistently with timestamps observed from other devices.

use serde::{Deserialize, Serialize};

use crate::ids::DeviceId;

/// A hybrid logical clock timestamp. Ordered by wall time, then counter, then node.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(from = "HlcRepr", into = "HlcRepr")]
pub struct Hlc {
    /// Milliseconds since the Unix epoch (UTC).
    pub wall_ms: u64,
    /// Counter for several timestamps within the same millisecond.
    pub counter: u16,
    /// The device that created the timestamp.
    pub node: DeviceId,
}

/// Wire form: `[wall, counter, node]`.
#[derive(Serialize, Deserialize)]
struct HlcRepr(u64, u16, DeviceId);

impl From<HlcRepr> for Hlc {
    fn from(HlcRepr(wall_ms, counter, node): HlcRepr) -> Self {
        Self {
            wall_ms,
            counter,
            node,
        }
    }
}

impl From<Hlc> for HlcRepr {
    fn from(hlc: Hlc) -> Self {
        Self(hlc.wall_ms, hlc.counter, hlc.node)
    }
}

/// The clock could not produce or accept a timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HlcError {
    /// A remote timestamp is further in the future than allowed.
    #[error("the other clock is {ahead_ms} ms ahead, more than the allowed {max_ms} ms")]
    TooFarAhead {
        /// How far the remote timestamp is ahead of the local physical clock.
        ahead_ms: u64,
        /// The configured limit.
        max_ms: u64,
    },
    /// More than 65 535 timestamps were requested within one millisecond.
    #[error("more than 65 535 timestamps within one millisecond")]
    CounterOverflow,
}

/// A hybrid logical clock for one device.
///
/// The clock does not read the time itself: callers pass the physical time, which keeps
/// this crate free of I/O and makes the clock easy to test. To stay monotonic across
/// restarts, call [`HlcClock::observe`] with the last timestamp the device stored.
#[derive(Debug, Clone)]
pub struct HlcClock {
    node: DeviceId,
    last_wall_ms: u64,
    last_counter: u16,
    max_ahead_ms: u64,
}

impl HlcClock {
    /// Default limit for how far a remote clock may be ahead: one minute.
    pub const DEFAULT_MAX_AHEAD_MS: u64 = 60_000;

    /// A new clock for `node` with the default limit.
    pub fn new(node: DeviceId) -> Self {
        Self::with_max_ahead(node, Self::DEFAULT_MAX_AHEAD_MS)
    }

    /// A new clock for `node` that rejects remote timestamps more than `max_ahead_ms`
    /// ahead of the local physical clock.
    pub fn with_max_ahead(node: DeviceId, max_ahead_ms: u64) -> Self {
        Self {
            node,
            last_wall_ms: 0,
            last_counter: 0,
            max_ahead_ms,
        }
    }

    /// A new timestamp, greater than every timestamp produced or observed before.
    pub fn now(&mut self, physical_ms: u64) -> Result<Hlc, HlcError> {
        if physical_ms > self.last_wall_ms {
            self.last_wall_ms = physical_ms;
            self.last_counter = 0;
        } else {
            self.last_counter = self
                .last_counter
                .checked_add(1)
                .ok_or(HlcError::CounterOverflow)?;
        }
        Ok(Hlc {
            wall_ms: self.last_wall_ms,
            counter: self.last_counter,
            node: self.node,
        })
    }

    /// Takes a timestamp from another device (or from storage) into account, so that
    /// all later timestamps of this clock are greater.
    pub fn observe(&mut self, remote: &Hlc, physical_ms: u64) -> Result<(), HlcError> {
        let ahead_ms = remote.wall_ms.saturating_sub(physical_ms);
        if ahead_ms > self.max_ahead_ms {
            return Err(HlcError::TooFarAhead {
                ahead_ms,
                max_ms: self.max_ahead_ms,
            });
        }
        if (remote.wall_ms, remote.counter) > (self.last_wall_ms, self.last_counter) {
            self.last_wall_ms = remote.wall_ms;
            self.last_counter = remote.counter;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ulid::Ulid;

    fn device(n: u128) -> DeviceId {
        DeviceId(Ulid::from_parts(0, n).unwrap())
    }

    #[test]
    fn follows_physical_time() {
        let mut clock = HlcClock::new(device(1));
        let a = clock.now(1_000).unwrap();
        let b = clock.now(2_000).unwrap();
        assert_eq!((a.wall_ms, a.counter), (1_000, 0));
        assert_eq!((b.wall_ms, b.counter), (2_000, 0));
    }

    #[test]
    fn stays_monotonic_when_time_stands_still_or_goes_back() {
        let mut clock = HlcClock::new(device(1));
        let a = clock.now(5_000).unwrap();
        let b = clock.now(5_000).unwrap();
        let c = clock.now(4_000).unwrap();
        assert!(a < b && b < c);
        assert_eq!((c.wall_ms, c.counter), (5_000, 2));
    }

    #[test]
    fn orders_after_observed_remote_time() {
        let mut local = HlcClock::new(device(1));
        let mut remote = HlcClock::new(device(2));
        let remote_ts = remote.now(10_500).unwrap();
        local.observe(&remote_ts, 10_000).unwrap();
        let next = local.now(10_000).unwrap();
        assert!(next > remote_ts);
    }

    #[test]
    fn rejects_clock_far_ahead() {
        let mut local = HlcClock::with_max_ahead(device(1), 1_000);
        let mut remote = HlcClock::new(device(2));
        let remote_ts = remote.now(20_000).unwrap();
        assert_eq!(
            local.observe(&remote_ts, 10_000),
            Err(HlcError::TooFarAhead {
                ahead_ms: 10_000,
                max_ms: 1_000
            })
        );
        // The rejected timestamp must not influence the clock.
        assert_eq!(local.now(10_000).unwrap().wall_ms, 10_000);
    }

    #[test]
    fn counter_overflow_is_an_error_and_keeps_state() {
        let mut clock = HlcClock::new(device(1));
        let remote = Hlc {
            wall_ms: 1_000,
            counter: u16::MAX,
            node: device(2),
        };
        clock.observe(&remote, 1_000).unwrap();
        assert_eq!(clock.now(1_000), Err(HlcError::CounterOverflow));
        // Once physical time moves on, the clock works again.
        assert_eq!(clock.now(1_001).unwrap().counter, 0);
    }

    #[test]
    fn node_breaks_ties() {
        let a = Hlc {
            wall_ms: 1,
            counter: 0,
            node: device(1),
        };
        let b = Hlc {
            node: device(2),
            ..a
        };
        assert!(a < b);
    }
}
