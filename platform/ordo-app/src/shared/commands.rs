//! The shared modules' entries in the command palette (UX-10, ADR-0030): adding a unit,
//! every unit, and a unit's status codes.

use leptos::prelude::*;
use ordo_units::{self as units, UnitRegister};

use crate::core::{self, Handle};
use crate::shared::text;
use crate::shell::nav::{self, Route};
use crate::shell::palette::{Action, Command, Item, PalettePart, Record, RecordPage, item};
use crate::shell::views::access;

/// The kind of record a unit is in the palette.
const UNIT: &str = "unit";

// [impl->dsn~command-palette~1]
/// The shared modules' part of the palette.
pub(crate) const PALETTE: PalettePart = PalettePart {
    actions,
    records,
    record,
};

fn command_access(core: Handle) -> Result<(), &'static str> {
    nav::workspace("command.prepare").map_or(Err(text::reason_no_access()), |w| access(core, w))
}

fn actions(core: Handle) -> Vec<Item> {
    vec![Item {
        disabled: command_access(core).err(),
        suggested: true,
        ..item(
            text::group_actions(),
            text::ACTION_ADD_UNIT.to_owned(),
            Action::GoFocus(
                Route::view("command.prepare", "units"),
                "unit-call-sign".to_owned(),
            ),
        )
        .named("add unit")
    }]
}

fn records(core: Handle) -> Vec<Item> {
    if command_access(core).is_err() {
        return Vec::new();
    }
    let register = core.with_value(|c| UnitRegister::from_events(&c.events(units::COMPARTMENT)));
    register
        .units
        .into_values()
        .map(|u| {
            let status = u.status.as_ref().map_or_else(
                || text::NO_STATUS.to_owned(),
                |s| text::status_label(&s.status),
            );
            Item {
                detail: format!("{} · {} · {status}", u.unit_type, u.organisation),
                ..item(
                    text::GROUP_UNITS,
                    u.unit.clone(),
                    Action::Open(Record {
                        kind: UNIT,
                        id: u.unit.clone(),
                    }),
                )
            }
        })
        .collect()
}

/// Sets the status of the unit `args[0]` to the code `args[1]`.
fn set_status(core: Handle, revision: RwSignal<u64>, args: &[String]) {
    if let [unit, status] = args {
        let _ = core::set_status(core, revision, unit, status);
    }
}

fn record(core: Handle, record: &Record) -> RecordPage {
    if record.kind != UNIT {
        return None;
    }
    let unit = &record.id;
    let can_edit = core.with_value(|c| c.may(core::permission::EDIT_UNITS));
    let list = crate::shared::units::STATUS
        .codes
        .iter()
        .map(|status| {
            let code = status.code.as_str();
            let name = format!("status {code} {}", status.name);
            let action = Action::Run(Command {
                id: "units.status",
                run: set_status,
                args: vec![unit.clone(), code.to_owned()],
            });
            Item {
                detail: unit.clone(),
                suggested: true,
                disabled: (!can_edit).then_some(text::reason_no_access()),
                ..item(text::GROUP_UNITS, text::action_status(code), action).named(name)
            }
        })
        .collect();
    Some((unit.clone(), list))
}
