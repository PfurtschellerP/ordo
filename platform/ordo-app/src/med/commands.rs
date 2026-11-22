//! The Med package's entries in the command palette (UX-10, ADR-0030): registering a
//! patient, setting up a UHS, every patient by triage tag number, and a patient's commands.
//! Need to know: patients are found by triage tag number only (PAT-7).

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_med_patients::record::FormKind;
use ordo_med_patients::{self as med, PatientRegister};

use crate::core::{self, Handle};
use crate::med::patients::{FORMS, record_route};
use crate::med::text;
use crate::med::triage::CATEGORIES;
use crate::med::uhs::uhs_list;
use crate::shell::nav::{self, Route};
use crate::shell::palette::{Action, Command, Item, PalettePart, Record, RecordPage, item};
use crate::shell::views::access;

/// The kind of record a patient is in the palette.
const PATIENT: &str = "patient";

// [impl->dsn~command-palette~1]
/// The Med package's part of the palette.
pub(crate) const PALETTE: PalettePart = PalettePart {
    actions,
    records,
    record,
};

fn workspace_access(core: Handle, id: &str) -> Result<(), &'static str> {
    nav::workspace(id).map_or(Err(text::reason_no_access()), |w| access(core, w))
}

fn actions(core: Handle) -> Vec<Item> {
    [
        (
            text::action_register(),
            "register patient",
            Route::view("med.uhs", "patients"),
            "tag",
            workspace_access(core, "med.uhs"),
        ),
        (
            text::action_create_uhs(),
            "create uhs",
            // Set up by the incident command (ADR-0086).
            Route::view("command.prepare", "units"),
            "uhs-name",
            workspace_access(core, "command.prepare"),
        ),
    ]
    .into_iter()
    .map(|(label, name, route, target, allowed)| Item {
        disabled: allowed.err(),
        suggested: true,
        ..item(
            text::group_actions(),
            label.to_owned(),
            Action::GoFocus(route, target.to_owned()),
        )
        .named(name)
    })
    .collect()
}

fn records(core: Handle) -> Vec<Item> {
    if workspace_access(core, "med.uhs").is_err() {
        return Vec::new();
    }
    let names: std::collections::HashMap<String, String> =
        uhs_list(core).into_iter().map(|u| (u.id, u.name)).collect();
    let register = core.with_value(|c| PatientRegister::from_events(&c.events(med::COMPARTMENT)));
    register
        .patients()
        .map(|p| {
            let uhs = names.get(&p.uhs).unwrap_or(&p.uhs);
            Item {
                detail: format!("{} · {uhs}", text::label(p.category())),
                ..item(
                    text::group_patients(),
                    p.tag.clone(),
                    Action::Open(Record {
                        kind: PATIENT,
                        id: p.tag.clone(),
                    }),
                )
            }
        })
        .collect()
}

/// English name of a triage category (its core meaning), for typed commands (ADR-0036).
fn category_name(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => "immediate",
        TriageCategory::Urgent => "urgent",
        TriageCategory::Minor => "minor",
        TriageCategory::Expectant => "expectant",
        TriageCategory::Deceased => "deceased",
        TriageCategory::Uninjured => "uninjured",
    }
}

/// Re-triages the patient `args[0]` to the category named `args[1]`.
fn retriage(core: Handle, revision: RwSignal<u64>, args: &[String]) {
    let [tag, name] = args else { return };
    if let Some(category) = CATEGORIES.into_iter().find(|c| category_name(*c) == name) {
        let _ = core::retriage(core, revision, tag, category);
    }
}

fn record(core: Handle, record: &Record) -> RecordPage {
    if record.kind != PATIENT {
        return None;
    }
    let tag = &record.id;
    let route = record_route(tag);
    let patient = core.with_value(|c| {
        PatientRegister::from_events(&c.events(med::COMPARTMENT))
            .patient(tag)
            .map(|p| (p.category(), p.uhs.clone()))
    });
    let current = patient.as_ref().map(|(c, _)| *c);
    let group = text::group_patients();
    let mut list = vec![
        item(
            group,
            text::action_open_record().to_owned(),
            Action::Go(route.clone()),
        )
        .named("open record"),
    ];
    // [impl->dsn~uhs-staff~3]
    // Where the person may only read the patient (UHS-10), only opening is offered.
    let writable =
        patient.is_some_and(|(_, uhs)| crate::med::patients::writable_uhs(core).contains(&uhs));
    if writable {
        list.push(
            item(
                group,
                text::action_take_photo().to_owned(),
                Action::GoFocus(route.clone(), "photo-camera".to_owned()),
            )
            .named("photo"),
        );
        for f in FORMS.forms.iter().filter(|f| f.kind == FormKind::Series) {
            let go = Action::GoClick(route.clone(), format!("new-{}", f.id));
            list.push(item(group, text::action_entry(&f.label), go).named(format!("add {}", f.id)));
        }
        for c in CATEGORIES.into_iter().filter(|c| Some(*c) != current) {
            let action = Action::Run(Command {
                id: "med.retriage",
                run: retriage,
                args: vec![tag.clone(), category_name(c).to_owned()],
            });
            list.push(
                item(group, text::action_retriage(c), action)
                    .named(format!("retriage {}", category_name(c))),
            );
        }
    }
    for i in &mut list {
        i.detail = text::patient_title(tag);
        i.suggested = true;
    }
    Some((text::patient_title(tag), list))
}
