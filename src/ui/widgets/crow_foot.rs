use crate::app::TableId;
use crate::model::constraints::constraint::ForeignKey;
use crate::model::entities::table::Table;
use eframe::epaint::{Color32, Pos2, Stroke};
use egui::{Painter, Rect, Shape, Vec2};
use slotmap::SlotMap;

const RELATION_COLOR: Color32 = Color32::DARK_GRAY;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationshipKind {
    Identifying,
    NonIdentifying,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalityMin {
    Zero,
    One,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalityMax {
    One,
    Many,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cardinality {
    pub min: CardinalityMin,
    pub max: CardinalityMax,
}

impl Cardinality {
    pub const fn one() -> Self {
        Self {
            min: CardinalityMin::One,
            max: CardinalityMax::One,
        }
    }

    pub const fn zero_to_many() -> Self {
        Self {
            min: CardinalityMin::Zero,
            max: CardinalityMax::Many,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CrowFootEdge {
    pub from: Pos2,
    pub to: Pos2,
    pub from_cardinality: Cardinality,
    pub to_cardinality: Cardinality,
    pub color: Color32,
    pub kind: RelationshipKind,
}

impl CrowFootEdge {
    pub fn with_relation(
        from: Pos2,
        to: Pos2,
        from_cardinality: Cardinality,
        to_cardinality: Cardinality,
        kind: RelationshipKind,
    ) -> Self {
        Self {
            from,
            to,
            from_cardinality,
            to_cardinality,
            color: RELATION_COLOR,
            kind,
        }
    }
}

/// Draws a single IE crow's-foot relationship edge.
pub fn draw_crow_foot_edge(painter: &Painter, edge: &CrowFootEdge) {
    let stroke = Stroke::new(1.8, edge.color);
    match edge.kind {
        RelationshipKind::Identifying => {
            painter.line_segment([edge.from, edge.to], stroke);
        }
        RelationshipKind::NonIdentifying => {
            draw_dotted_line(painter, edge.from, edge.to, stroke);
        }
    }

    draw_endpoint(painter, edge.from, edge.to - edge.from, edge.from_cardinality, stroke);
    draw_endpoint(painter, edge.to, edge.from - edge.to, edge.to_cardinality, stroke);
}

/// Builds drawable crow's-foot edges from FK metadata and table window rectangles.
pub fn build_edges(
    tables: &SlotMap<TableId, Table>,
    rects: &std::collections::HashMap<TableId, Rect>,
) -> Vec<CrowFootEdge> {
    let mut out = Vec::new();

    for (from_id, from_table) in tables {
        let Some(from_rect) = rects.get(&from_id) else {
            continue;
        };

        for fk in from_table.fks.values() {
            let Some(to_id) = fk.references else {
                continue;
            };

            let Some(to_rect) = rects.get(&to_id) else {
                continue;
            };

            let (from_anchor, to_anchor) = anchors_between(*from_rect, *to_rect);

            let parent_per_child = parent_side_cardinality(from_table, fk);
            let children_per_parent = child_side_cardinality(from_table, fk);
            let relation_kind = relationship_kind(from_table, fk);

            out.push(CrowFootEdge::with_relation(
                from_anchor,
                to_anchor,
                children_per_parent,
                parent_per_child,
                relation_kind,
            ));
        }
    }

    out
}

fn parent_side_cardinality(from_table: &Table, fk: &ForeignKey) -> Cardinality {
    let mandatory = !fk.local_attrs.is_empty()
        && fk
        .local_attrs
        .iter()
        .all(|attr_id| from_table.attributes.get(*attr_id).map(|a| a.not_null).unwrap_or(false));

    Cardinality {
        min: if mandatory {
            CardinalityMin::One
        } else {
            CardinalityMin::Zero
        },
        max: CardinalityMax::One,
    }
}

fn relationship_kind(from_table: &Table, fk: &ForeignKey) -> RelationshipKind {
    let identifying = !fk.local_attrs.is_empty()
        && fk
            .local_attrs
            .iter()
            .all(|attr_id| from_table.pk.attributes.contains(attr_id));

    if identifying {
        RelationshipKind::Identifying
    } else {
        RelationshipKind::NonIdentifying
    }
}

fn child_side_cardinality(from_table: &Table, fk: &ForeignKey) -> Cardinality {
    let has_exact_unique = from_table
        .uniques
        .iter()
        .any(|u| !u.attributes.is_empty() && u.attributes == fk.local_attrs);
    let is_pk = !from_table.pk.attributes.is_empty() && from_table.pk.attributes == fk.local_attrs;
    let has_inline_unique_single_column = {
        let mut local_attrs = fk.local_attrs.iter();
        match (local_attrs.next(), local_attrs.next()) {
            (Some(attr_id), None) => from_table
                .attributes
                .get(*attr_id)
                .map(|attr| attr.unique)
                .unwrap_or(false),
            _ => false,
        }
    };

    Cardinality {
        min: CardinalityMin::Zero,
        max: if has_exact_unique || is_pk || has_inline_unique_single_column {
            CardinalityMax::One
        } else {
            CardinalityMax::Many
        },
    }
}

fn anchors_between(from: Rect, to: Rect) -> (Pos2, Pos2) {
    let delta = to.center() - from.center();

    if delta.x.abs() >= delta.y.abs() {
        if delta.x >= 0.0 {
            (
                Pos2::new(from.right(), from.center().y),
                Pos2::new(to.left(), to.center().y),
            )
        } else {
            (
                Pos2::new(from.left(), from.center().y),
                Pos2::new(to.right(), to.center().y),
            )
        }
    } else if delta.y >= 0.0 {
        (
            Pos2::new(from.center().x, from.bottom()),
            Pos2::new(to.center().x, to.top()),
        )
    } else {
        (
            Pos2::new(from.center().x, from.top()),
            Pos2::new(to.center().x, to.bottom()),
        )
    }
}

fn draw_endpoint(
    painter: &Painter,
    point: Pos2,
    mut direction_to_other: Vec2,
    cardinality: Cardinality,
    stroke: Stroke,
) {
    if direction_to_other == Vec2::ZERO {
        return;
    }

    direction_to_other = direction_to_other.normalized();
    let normal = Vec2::new(-direction_to_other.y, direction_to_other.x);

    let min_pos = point + direction_to_other * 7.0;
    let max_pos = point + direction_to_other * 15.0;

    match cardinality.min {
        CardinalityMin::Zero => {
            painter.circle_stroke(min_pos, 4.0, stroke);
        }
        CardinalityMin::One => {
            draw_bar(painter, min_pos, normal, stroke);
        }
    }

    match cardinality.max {
        CardinalityMax::One => {
            draw_bar(painter, max_pos, normal, stroke);
        }
        CardinalityMax::Many => {
            draw_crow_foot(painter, max_pos, direction_to_other, normal, stroke);
        }
    }
}

fn draw_bar(painter: &Painter, center: Pos2, normal: Vec2, stroke: Stroke) {
    let half = normal * 5.0;
    painter.line_segment([center - half, center + half], stroke);
}

fn draw_crow_foot(painter: &Painter, apex: Pos2, direction: Vec2, normal: Vec2, stroke: Stroke) {
    let root = apex - direction * 8.0;
    let left = root + normal * 6.0;
    let middle = root;
    let right = root - normal * 6.0;

    painter.add(Shape::line_segment([apex, left], stroke));
    painter.add(Shape::line_segment([apex, middle], stroke));
    painter.add(Shape::line_segment([apex, right], stroke));
}

fn draw_dotted_line(painter: &Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let delta = to - from;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }

    let direction = delta / length;
    let mut offset = 0.0;
    const DOT_LEN: f32 = 4.0;
    const GAP_LEN: f32 = 6.0;

    while offset < length {
        let start = from + direction * offset;
        let end = from + direction * (offset + DOT_LEN).min(length);
        painter.line_segment([start, end], stroke);
        offset += DOT_LEN + GAP_LEN;
    }
}



