//! ORDO design tokens.
//!
//! Reads design tokens in the W3C Design Tokens format (DTCG 2025.10), layers them into
//! themes, checks every theme against the WCAG contrast pairs in `design/contrast.toml`,
//! and generates CSS custom properties.
//!
//! The same checks are used at runtime to validate organisation themes (branding), so
//! that no theme can break accessibility or change safety-relevant colours
//! (docs/architecture/08-concepts.md, section 8.6.3).

pub mod color;
pub mod css;
pub mod design;
pub mod dtcg;
mod error;
pub mod report;
pub mod value;

pub use design::{CheckReport, ContrastPair, DesignSystem, PairResult, ThemeReport};
pub use error::Error;
