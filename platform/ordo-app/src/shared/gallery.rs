//! The shared modules' building blocks in the component gallery (DS-1, ADR-0094), with
//! sample units only.

use leptos::prelude::*;
use ordo_units::Strength;

use crate::shared::summary::TileHead;
use crate::shared::units::StrengthText;
use crate::shell::gallery::{Example, Part};

/// The shared modules' part of the gallery.
pub(crate) const PARTS: &[Part] = &[Part {
    id: "shared",
    title: "Einsatz",
    examples: &[
        Example {
            id: "tiles",
            name: "Tiles",
            code: "shared::summary::TileHead, .tile",
            usage: "One subject of an overview page with its icon, heading, a link to its page and its figures. Tiles sit in a .bento grid.",
            access: "A section labelled by its heading; the link says where it goes, not \"mehr\".",
            show: tile,
        },
        Example {
            id: "strength",
            name: "Strength",
            code: "shared::units::StrengthText",
            usage: "A unit's crew in tactical notation (Führer / Unterführer / Mannschaft / gesamt), the total underlined; the detailed notation where the incident asks for it.",
            access: "Plain text and numbers, read as written.",
            show: strength,
        },
    ],
}];

fn tile() -> AnyView {
    view! {
        <section class="tile" aria-labelledby="gallery-tile">
            <TileHead id="gallery-tile" icon="clipboard-list" title=crate::shared::text::DISPATCH
                route=crate::shell::nav::Route::view("command.lead", "dispatch") link=crate::shared::text::OPEN_DISPATCH />
            <p class="tile__figure"><span class="tile__number">"3"</span>" offen"<span class="tag tag--warning">"1 fällig"</span></p>
        </section>
    }
    .into_any()
}

fn strength() -> AnyView {
    let strength = Strength {
        unit_leaders: 0,
        leaders: 1,
        sub_leaders: 2,
        crew: 6,
    };
    view! {
        <p>"RTW 83/1 · Stärke " <StrengthText strength /></p>
        <p>"Zug 1 · Stärke " <StrengthText strength detailed=true /></p>
    }
    .into_any()
}
