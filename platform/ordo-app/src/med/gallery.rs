//! The Med package's building blocks in the component gallery (DS-1, ADR-0094), with
//! sample patients only.

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_med_patients::overview::{Occupancy, PlaceGroup};

use crate::med::flow::{Edge, FlowChart, Kind, State, Station};
use crate::med::places::{Level, PlaceRow, TreatmentPlaces};
use crate::med::triage::{CATEGORIES, TriageBadge};
use crate::shell::gallery::{Example, Part};

/// The package's part of the gallery.
pub(crate) const PARTS: &[Part] = &[Part {
    id: "med",
    title: "Sanität",
    examples: &[
        Example {
            id: "triage-badges",
            name: "Triage badges",
            code: "med::triage::TriageBadge",
            usage: "Wherever a patient's triage category shows: lists, cards, counts. Labels and colours come from the profile's catalogue.",
            access: "Colour, label and a shape per category, never colour alone; the category's full name for screen readers. Safety colours: no theme changes them.",
            show: triage,
        },
        Example {
            id: "patient-cards",
            name: "Patient cards",
            code: "med::patients::PatientLink",
            usage: "One patient in a list or on a place: card number, category, name, where; then what needs attention (transport priority, handover, a card registered twice).",
            access: "The whole card is one link to the record; the open record's card has aria-current. j and k move through the list.",
            show: crate::med::patients::gallery_cards,
        },
        Example {
            id: "treatment-places",
            name: "Treatment places",
            code: "med::places::TreatmentPlaces",
            usage: "Free and taken places per UHS and category, \"2 / 4 frei\": on Führung (overview tile and Lage, with staff, transports and requests there), in the UHS module and for crews. Numbers only, never who lies where.",
            access: "Each strip has a text alternative (\"SK I: 1 von 2 frei\"); a full category is hatched, outlined and adds the tag \"voll\". The badge stays on one line.",
            show: places,
        },
        Example {
            id: "patient-flow",
            name: "Patient's way",
            code: "med::flow::FlowChart",
            usage: "Where the patient was and is, with how and when they got there; the dashed box is a handover not taken yet.",
            access: "An ordered list read in order; the arrows are drawn, not read. \"Jetzt\" marks the current station in words.",
            show: flow,
        },
    ],
}];

fn triage() -> AnyView {
    view! {
        <div class="row">
            {CATEGORIES.into_iter().map(|category| view! { <TriageBadge category /> }).collect_view()}
        </div>
    }
    .into_any()
}

fn group(category: Option<TriageCategory>, total: usize, taken: usize) -> PlaceGroup {
    PlaceGroup {
        category,
        total,
        taken,
    }
}

fn places() -> AnyView {
    let rows = vec![
        PlaceRow {
            id: "gallery-nord".into(),
            name: "UHS Bühne Nord".into(),
            occupancy: Some(Occupancy {
                groups: vec![
                    group(Some(TriageCategory::Immediate), 2, 2),
                    group(Some(TriageCategory::Urgent), 3, 1),
                    group(Some(TriageCategory::Minor), 5, 2),
                ],
                without_place: 1,
            }),
            load: (6, 10),
            more: Some("1 Transport offen".into()),
            lage: None,
        },
        PlaceRow {
            id: "gallery-eingang".into(),
            name: "UHS Eingang".into(),
            occupancy: Some(Occupancy {
                groups: vec![group(None, 4, 1)],
                without_place: 0,
            }),
            load: (1, 4),
            more: None,
            lage: None,
        },
    ];
    view! { <TreatmentPlaces rows=Signal::stored(rows) level=Level::H4 /> }.into_any()
}

fn flow() -> AnyView {
    let edge = |what: &str, time: &str| {
        Some(Edge {
            what: what.to_owned(),
            time: time.to_owned(),
        })
    };
    let list = vec![
        Station {
            kind: Kind::Unit,
            title: "RTW 83/1".into(),
            lines: vec!["Aufgenommen 14:02".into()],
            edge: None,
            state: State::Past,
        },
        Station {
            kind: Kind::Uhs,
            title: "UHS Bühne Nord".into(),
            lines: vec!["Platz 2".into()],
            edge: edge("Übergeben", "14:12"),
            state: State::Now,
        },
        Station {
            kind: Kind::Uhs,
            title: "UHS Eingang".into(),
            lines: Vec::new(),
            edge: edge("Übergabe offen", "14:30"),
            state: State::Open,
        },
    ];
    view! { <FlowChart list=Signal::stored(list) next=Signal::stored(false) /> }.into_any()
}
