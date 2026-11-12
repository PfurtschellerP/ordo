//! What can be drawn on the situation map (MAP-4, ADR-0064): the categories of the profile
//! (`map/drawings.toml`), each an area or a line with a line pattern.

use serde::{Deserialize, Serialize};

/// An area (closed) or a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shape {
    /// A closed area, at least three corners.
    Area,
    /// A line, at least two points.
    Line,
}

impl Shape {
    /// Points needed at least.
    pub fn min_points(self) -> usize {
        match self {
            Self::Area => 3,
            Self::Line => 2,
        }
    }
}

/// How the outline is drawn; categories differ by it, not by colour alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineStyle {
    /// Solid.
    Solid,
    /// Dashed.
    Dashed,
    /// Dotted.
    Dotted,
}

/// A category, e.g. "Einsatzraum".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    /// Stable ID, stored as the drawing's reference.
    pub id: String,
    /// Label.
    pub label: String,
    /// Area or line.
    pub shape: Shape,
    /// Line pattern.
    pub line: LineStyle,
}

/// The categories of the profile.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawingCatalogue {
    /// In the order they are offered.
    #[serde(rename = "category", default)]
    pub categories: Vec<Category>,
}

impl DrawingCatalogue {
    /// The category with this ID.
    pub fn get(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Checks that there is a category, IDs are unique and labels are given.
    ///
    /// # Errors
    ///
    /// Describes the first problem found.
    pub fn check(&self) -> Result<(), String> {
        if self.categories.is_empty() {
            return Err("at least one category".to_owned());
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.categories {
            if c.id.trim().is_empty() || c.label.trim().is_empty() {
                return Err(format!("category {:?}: ID and label are needed", c.id));
            }
            if !seen.insert(c.id.as_str()) {
                return Err(format!("category {} twice", c.id));
            }
        }
        Ok(())
    }
}

/// Closes a ring of corners for a GeoJSON polygon: the first corner again at the end.
pub fn closed_ring(corners: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut ring = corners.to_vec();
    if let (Some(first), Some(last)) = (corners.first(), corners.last())
        && first != last
    {
        ring.push(*first);
    }
    ring
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Geometry;

    // [utest->dsn~map-drawing~2]
    #[test]
    fn categories_are_checked_and_areas_are_closed() {
        let c = DrawingCatalogue {
            categories: vec![Category {
                id: "incident-area".into(),
                label: "Einsatzraum".into(),
                shape: Shape::Area,
                line: LineStyle::Solid,
            }],
        };
        assert_eq!(c.check(), Ok(()));
        assert_eq!(c.get("incident-area").map(|c| c.shape), Some(Shape::Area));
        let mut twice = c.clone();
        twice.categories.push(c.categories[0].clone());
        assert!(twice.check().is_err());
        assert!(DrawingCatalogue::default().check().is_err());

        // Three corners make a valid GeoJSON area once closed; two make a line.
        let corners = [[10.0, 48.0], [10.1, 48.0], [10.1, 48.1]];
        assert_eq!(closed_ring(&corners).len(), 4);
        assert!(Geometry::Polygon(vec![closed_ring(&corners)]).is_valid());
        assert!(
            !Geometry::Polygon(vec![corners.to_vec()]).is_valid(),
            "open rings are not"
        );
        assert!(Geometry::LineString(corners[..2].to_vec()).is_valid());
        assert_eq!(Shape::Area.min_points(), 3);
    }
}
