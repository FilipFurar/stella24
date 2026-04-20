use crate::app::{DomainId, TableId};
use crate::model::attribute::{Attribute, AttributeType};
use crate::model::datatype::DATA_TYPES;
use crate::model::entities::domain::Domain;
use crate::ui::widgets::crow_foot::{
    Cardinality, CardinalityMax, CardinalityMin, CrowFootEdge, RelationshipKind, build_edges,
};
use crate::AppStella;
use egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug)]
pub struct SvgScene {
    pub width: f32,
    pub height: f32,
    pub tables: Vec<SvgTableNode>,
    pub relations: Vec<SvgRelationEdge>,
}

#[derive(Clone, Debug)]
pub struct SvgTableNode {
    pub id: TableId,
    pub title: String,
    pub attributes: Vec<SvgAttributeRow>,
    pub rect: Rect,
}

#[derive(Clone, Debug)]
pub struct SvgAttributeRow {
    pub name: String,
    pub datatype: String,
    pub constraints: String,
}

#[derive(Clone, Debug)]
pub struct SvgRelationEdge {
    pub from: Pos2,
    pub to: Pos2,
    pub route: Vec<Pos2>,
    pub from_cardinality: Cardinality,
    pub to_cardinality: Cardinality,
    pub kind: RelationshipKind,
    pub color: Color32,
}

impl AppStella {
    pub fn to_svg(&self, path: &str) {
        let scene = model_to_svg_scene(self);
        let svg = render_svg_scene(&scene);
        if let Err(err) = fs::write(path, svg) {
            eprintln!("Error exporting SVG: {err}");
        }
    }
}

pub fn model_to_svg_scene(app: &AppStella) -> SvgScene {
    let tables = map_tables_to_nodes(app.tables(), app.domains());

    let mut rects = HashMap::new();
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;

    for node in &tables {
        rects.insert(node.id, node.rect);
        max_x = max_x.max(node.rect.right());
        max_y = max_y.max(node.rect.bottom());
    }

    let mut edges = build_edges(app.tables(), &rects);
    distribute_edge_anchors(&mut edges, &rects);

    let relations = edges
        .into_iter()
        .map(|e| SvgRelationEdge {
            from: e.from,
            to: e.to,
            route: vec![e.from, e.to],
            from_cardinality: e.from_cardinality,
            to_cardinality: e.to_cardinality,
            kind: e.kind,
            color: e.color,
        })
        .collect();

    let mut relations = deduplicate_relations(relations);
    for rel in &mut relations {
        rel.route = route_relation(rel.from, rel.to, &rects);
    }

    SvgScene {
        width: (max_x + 60.0).max(840.0),
        height: (max_y + 60.0).max(520.0),
        tables,
        relations,
    }
}

fn map_tables_to_nodes(
    tables: &SlotMap<TableId, crate::model::entities::table::Table>,
    domains: &SlotMap<DomainId, Domain>,
) -> Vec<SvgTableNode> {
    let mut out = Vec::new();
    let mut x = 40.0;
    let mut y = 40.0;
    let mut col = 0;
    let mut row_max: f32 = 0.0;

    for (id, table) in tables {
        let attributes = table
            .attributes
            .values()
            .map(|a| format_attribute_row(a, domains))
            .collect::<Vec<_>>();

        let h = 66.0 + (attributes.len() as f32 * 24.0).max(24.0);
        if col == 3 {
            col = 0;
            x = 40.0;
            y += row_max + 80.0;
            row_max = 0.0;
        }

        let rect = Rect::from_min_size(pos2(x, y), vec2(300.0, h));
        row_max = row_max.max(h);

        out.push(SvgTableNode {
            id,
            title: table.title.clone(),
            attributes,
            rect,
        });

        x += 380.0;
        col += 1;
    }

    out
}

fn format_attribute_row(attr: &Attribute, domains: &SlotMap<DomainId, Domain>) -> SvgAttributeRow {
    let mut flags = Vec::new();
    if attr.pk {
        flags.push("PK");
    }
    if attr.not_null {
        flags.push("NN");
    }
    if attr.unique {
        flags.push("U");
    }

    let ty = match &attr.attribute_type {
        AttributeType::Logical(dt) => {
            let name = DATA_TYPES.get(dt.base).map(|d| d.name).unwrap_or("UNKNOWN");
            if dt.params.is_empty() {
                name.to_string()
            } else {
                format!(
                    "{}({})",
                    name,
                    dt.params
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        AttributeType::Domain(did) => domains
            .get(*did)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Invalid domain".to_string()),
        AttributeType::ForeignKeyAttribute(_) => "FK".to_string(),
    };

    SvgAttributeRow {
        name: attr.name.clone(),
        datatype: ty,
        constraints: if flags.is_empty() {
            String::new()
        } else {
            flags.join(", ")
        },
    }
}

pub fn render_svg_scene(scene: &SvgScene) -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\">\n",
        scene.width, scene.height, scene.width, scene.height
    ));
    // Keep exported SVG transparent; viewers can style background themselves.

    s.push_str("<g class=\"relations\">\n");
    for rel in &scene.relations {
        s.push_str(&render_relation(rel));
    }
    s.push_str("</g>\n");

    s.push_str("<g class=\"tables\">\n");
    for table in &scene.tables {
        s.push_str(&render_table(table));
    }
    s.push_str("</g>\n");

    s.push_str("</svg>\n");
    s
}

fn render_table(table: &SvgTableNode) -> String {
    let mut s = String::new();
    let r = table.rect;
    s.push_str(&format!(
        "<g class=\"table\" data-id=\"{:?}\">\n",
        table.id
    ));
    s.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"8\" fill=\"#2b2b2b\" stroke=\"#505050\"/>\n",
        r.left(),
        r.top(),
        r.width(),
        r.height()
    ));

    let title_y = r.top() + 26.0;
    s.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#f0f0f0\" font-family=\"Inter, Segoe UI, Arial, sans-serif\" font-size=\"30\" font-weight=\"700\" transform=\"scale(0.45)\">{}</text>\n",
        (r.left() + 12.0) / 0.45,
        title_y / 0.45,
        escape_xml(&table.title)
    ));

    let name_x = r.left() + 12.0;
    let datatype_x = r.left() + r.width() * 0.50;
    let constraints_x = r.right() - 12.0;
    let mut y = r.top() + 54.0;
    for attr in &table.attributes {
        s.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#d8d8d8\" font-family=\"Inter, Segoe UI, Arial, sans-serif\" font-size=\"12\" text-anchor=\"start\" dominant-baseline=\"middle\">{}</text>\n",
            name_x,
            y,
            escape_xml(&attr.name)
        ));
        s.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#d8d8d8\" font-family=\"Inter, Segoe UI, Arial, sans-serif\" font-size=\"12\" text-anchor=\"middle\" dominant-baseline=\"middle\">{}</text>\n",
            datatype_x,
            y,
            escape_xml(&attr.datatype)
        ));
        if !attr.constraints.is_empty() {
            s.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#d8d8d8\" font-family=\"Inter, Segoe UI, Arial, sans-serif\" font-size=\"12\" text-anchor=\"end\" dominant-baseline=\"middle\">{}</text>\n",
                constraints_x,
                y,
                escape_xml(&attr.constraints)
            ));
        }
        y += 18.0;
    }

    s.push_str("</g>\n");
    s
}

fn render_relation(rel: &SvgRelationEdge) -> String {
    let color = color_hex(rel.color);
    let mut s = String::new();
    let dash = if rel.kind == RelationshipKind::NonIdentifying {
        " stroke-dasharray=\"4 6\""
    } else {
        ""
    };

    s.push_str(&format!(
        "<g class=\"relation\" data-kind=\"{}\" data-from-cardinality=\"{}\" data-to-cardinality=\"{}\">\n",
        if rel.kind == RelationshipKind::Identifying {
            "identifying"
        } else {
            "non-identifying"
        },
        cardinality_label(rel.from_cardinality),
        cardinality_label(rel.to_cardinality)
    ));

    s.push_str(&render_route(&rel.route, &color, dash));

    let from_dir = if rel.route.len() >= 2 {
        rel.route[1] - rel.route[0]
    } else {
        rel.to - rel.from
    };
    let to_dir = if rel.route.len() >= 2 {
        rel.route[rel.route.len() - 2] - rel.route[rel.route.len() - 1]
    } else {
        rel.from - rel.to
    };

    s.push_str(&render_endpoint(
        rel.from,
        from_dir,
        rel.from_cardinality,
        &color,
    ));
    s.push_str(&render_endpoint(
        rel.to,
        to_dir,
        rel.to_cardinality,
        &color,
    ));

    s.push_str("</g>\n");
    s
}

fn render_endpoint(point: Pos2, mut direction: Vec2, card: Cardinality, color: &str) -> String {
    if direction == Vec2::ZERO {
        return String::new();
    }

    direction = direction.normalized();
    let normal = vec2(-direction.y, direction.x);
    let min_pos = point + direction * 7.0;
    let max_pos = point + direction * 15.0;

    let mut s = String::new();

    match card.min {
        CardinalityMin::Zero => {
            s.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.8\"/>\n",
                min_pos.x, min_pos.y, color
            ));
        }
        CardinalityMin::One => {
            s.push_str(&render_bar(min_pos, normal, color));
        }
    }

    match card.max {
        CardinalityMax::One => {
            s.push_str(&render_bar(max_pos, normal, color));
        }
        CardinalityMax::Many => {
            s.push_str(&render_crow_foot(max_pos, direction, normal, color));
        }
    }

    s
}

fn render_route(points: &[Pos2], color: &str, dash: &str) -> String {
    let mut s = String::new();
    for win in points.windows(2) {
        s.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.8\"{}/>\n",
            win[0].x, win[0].y, win[1].x, win[1].y, color, dash
        ));
    }
    s
}

fn render_bar(center: Pos2, normal: Vec2, color: &str) -> String {
    let half = normal * 5.0;
    let a = center - half;
    let b = center + half;
    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.8\"/>\n",
        a.x, a.y, b.x, b.y, color
    )
}

fn render_crow_foot(apex: Pos2, direction: Vec2, normal: Vec2, color: &str) -> String {
    let root = apex - direction * 8.0;
    let left = root + normal * 6.0;
    let mid = root;
    let right = root - normal * 6.0;

    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.8\"/>\n\
         <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.8\"/>\n\
         <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.8\"/>\n",
        apex.x, apex.y, left.x, left.y, color,
        apex.x, apex.y, mid.x, mid.y, color,
        apex.x, apex.y, right.x, right.y, color
    )
}

fn color_hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

fn cardinality_label(c: Cardinality) -> &'static str {
    match (c.min, c.max) {
        (CardinalityMin::Zero, CardinalityMax::One) => "0..1",
        (CardinalityMin::Zero, CardinalityMax::Many) => "0..N",
        (CardinalityMin::One, CardinalityMax::One) => "1..1",
        (CardinalityMin::One, CardinalityMax::Many) => "1..N",
    }
}

fn relation_kind_label(kind: RelationshipKind) -> &'static str {
    if kind == RelationshipKind::Identifying {
        "identifying"
    } else {
        "non-identifying"
    }
}

fn deduplicate_relations(relations: Vec<SvgRelationEdge>) -> Vec<SvgRelationEdge> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for rel in relations {
        let key = (
            quant(rel.from.x),
            quant(rel.from.y),
            quant(rel.to.x),
            quant(rel.to.y),
            relation_kind_label(rel.kind),
            cardinality_label(rel.from_cardinality),
            cardinality_label(rel.to_cardinality),
            color_hex(rel.color),
        );
        if seen.insert(key) {
            out.push(rel);
        }
    }

    out
}

fn quant(v: f32) -> i32 {
    (v * 10.0).round() as i32
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn distribute_edge_anchors(edges: &mut [CrowFootEdge], rects: &HashMap<TableId, Rect>) {
    let mut groups: HashMap<(TableId, Side), Vec<(usize, bool)>> = HashMap::new();

    for (idx, edge) in edges.iter().enumerate() {
        if let Some((table_id, side)) = endpoint_table_side(edge.from, rects) {
            groups.entry((table_id, side)).or_default().push((idx, true));
        }
        if let Some((table_id, side)) = endpoint_table_side(edge.to, rects) {
            groups.entry((table_id, side)).or_default().push((idx, false));
        }
    }

    for ((table_id, side), mut endpoints) in groups {
        let Some(rect) = rects.get(&table_id).copied() else {
            continue;
        };

        endpoints.sort_by(|(ia, from_a), (ib, from_b)| {
            let oa = opposite_coord(&edges[*ia], *from_a, side);
            let ob = opposite_coord(&edges[*ib], *from_b, side);
            oa.partial_cmp(&ob).unwrap_or(std::cmp::Ordering::Equal)
        });

        let count = endpoints.len();
        for (slot, (edge_idx, is_from)) in endpoints.into_iter().enumerate() {
            let anchor = slotted_anchor(rect, side, slot, count);
            if is_from {
                edges[edge_idx].from = anchor;
            } else {
                edges[edge_idx].to = anchor;
            }
        }
    }
}

fn endpoint_table_side(point: Pos2, rects: &HashMap<TableId, Rect>) -> Option<(TableId, Side)> {
    for (id, rect) in rects {
        let eps = 0.5;
        let on_left = (point.x - rect.left()).abs() <= eps
            && point.y >= rect.top() - eps
            && point.y <= rect.bottom() + eps;
        let on_right = (point.x - rect.right()).abs() <= eps
            && point.y >= rect.top() - eps
            && point.y <= rect.bottom() + eps;
        let on_top = (point.y - rect.top()).abs() <= eps
            && point.x >= rect.left() - eps
            && point.x <= rect.right() + eps;
        let on_bottom = (point.y - rect.bottom()).abs() <= eps
            && point.x >= rect.left() - eps
            && point.x <= rect.right() + eps;

        if on_left {
            return Some((*id, Side::Left));
        }
        if on_right {
            return Some((*id, Side::Right));
        }
        if on_top {
            return Some((*id, Side::Top));
        }
        if on_bottom {
            return Some((*id, Side::Bottom));
        }
    }
    None
}

fn opposite_coord(edge: &CrowFootEdge, is_from: bool, side: Side) -> f32 {
    let other = if is_from { edge.to } else { edge.from };
    match side {
        Side::Left | Side::Right => other.y,
        Side::Top | Side::Bottom => other.x,
    }
}

fn slotted_anchor(rect: Rect, side: Side, slot: usize, count: usize) -> Pos2 {
    let t = (slot as f32 + 1.0) / (count as f32 + 1.0);
    let pad = 14.0;
    match side {
        Side::Left => Pos2::new(
            rect.left(),
            rect.top() + pad + (rect.height() - 2.0 * pad) * t,
        ),
        Side::Right => Pos2::new(
            rect.right(),
            rect.top() + pad + (rect.height() - 2.0 * pad) * t,
        ),
        Side::Top => Pos2::new(
            rect.left() + pad + (rect.width() - 2.0 * pad) * t,
            rect.top(),
        ),
        Side::Bottom => Pos2::new(
            rect.left() + pad + (rect.width() - 2.0 * pad) * t,
            rect.bottom(),
        ),
    }
}

fn side_outward(side: Side) -> Vec2 {
    match side {
        Side::Left => vec2(-1.0, 0.0),
        Side::Right => vec2(1.0, 0.0),
        Side::Top => vec2(0.0, -1.0),
        Side::Bottom => vec2(0.0, 1.0),
    }
}

fn route_relation(from: Pos2, to: Pos2, rects: &HashMap<TableId, Rect>) -> Vec<Pos2> {
    let from_side = endpoint_table_side(from, rects);
    let to_side = endpoint_table_side(to, rects);
    let from_id = from_side.map(|(id, _)| id);
    let to_id = to_side.map(|(id, _)| id);

    let from_stub = from_side
        .map(|(_, side)| from + side_outward(side) * 22.0)
        .unwrap_or(from);
    let to_stub = to_side
        .map(|(_, side)| to + side_outward(side) * 22.0)
        .unwrap_or(to);

    let mut route = route_between_stubs(from_stub, to_stub, rects, from_id, to_id);
    route.insert(0, from);
    route.push(to);
    simplify_route(route)
}

fn route_between_stubs(
    from: Pos2,
    to: Pos2,
    rects: &HashMap<TableId, Rect>,
    from_id: Option<TableId>,
    to_id: Option<TableId>,
) -> Vec<Pos2> {
    let mut candidates = vec![vec![from, pos2(to.x, from.y), to], vec![from, pos2(from.x, to.y), to]];

    let mut x_lanes = vec![from.x - 40.0, from.x + 40.0, to.x - 40.0, to.x + 40.0];
    let mut y_lanes = vec![from.y - 40.0, from.y + 40.0, to.y - 40.0, to.y + 40.0];

    for (id, rect) in rects {
        if Some(*id) == from_id || Some(*id) == to_id {
            continue;
        }
        x_lanes.push(rect.left() - 28.0);
        x_lanes.push(rect.right() + 28.0);
        y_lanes.push(rect.top() - 28.0);
        y_lanes.push(rect.bottom() + 28.0);
    }

    for x in x_lanes {
        candidates.push(vec![from, pos2(x, from.y), pos2(x, to.y), to]);
    }
    for y in y_lanes {
        candidates.push(vec![from, pos2(from.x, y), pos2(to.x, y), to]);
    }

    let mut best = vec![from, to];
    let mut best_score = f32::INFINITY;
    for c in candidates {
        let score = route_score(&c, rects, from_id, to_id);
        if score < best_score {
            best_score = score;
            best = c;
        }
    }

    simplify_route(best)
}

fn route_score(
    points: &[Pos2],
    rects: &HashMap<TableId, Rect>,
    from_id: Option<TableId>,
    to_id: Option<TableId>,
) -> f32 {
    let mut length = 0.0;
    for w in points.windows(2) {
        length += (w[1] - w[0]).length();
    }

    let mut hits = 0;
    for w in points.windows(2) {
        for (id, rect) in rects {
            if Some(*id) == from_id || Some(*id) == to_id {
                continue;
            }
            if segment_hits_rect_axis(w[0], w[1], *rect, 8.0) {
                hits += 1;
            }
        }
    }

    let bends = points.len().saturating_sub(2) as f32;
    (hits as f32) * 1_000_000.0 + length + bends * 12.0
}

fn segment_hits_rect_axis(a: Pos2, b: Pos2, rect: Rect, margin: f32) -> bool {
    let r = Rect::from_min_max(
        pos2(rect.left() - margin, rect.top() - margin),
        pos2(rect.right() + margin, rect.bottom() + margin),
    );

    if (a.x - b.x).abs() <= f32::EPSILON {
        if a.x < r.left() || a.x > r.right() {
            return false;
        }
        let y1 = a.y.min(b.y);
        let y2 = a.y.max(b.y);
        y2 >= r.top() && y1 <= r.bottom()
    } else if (a.y - b.y).abs() <= f32::EPSILON {
        if a.y < r.top() || a.y > r.bottom() {
            return false;
        }
        let x1 = a.x.min(b.x);
        let x2 = a.x.max(b.x);
        x2 >= r.left() && x1 <= r.right()
    } else {
        false
    }
}

fn simplify_route(mut points: Vec<Pos2>) -> Vec<Pos2> {
    points.dedup_by(|a, b| (*a - *b).length_sq() <= f32::EPSILON);
    if points.len() <= 2 {
        return points;
    }

    let mut out = Vec::with_capacity(points.len());
    out.push(points[0]);
    for i in 1..(points.len() - 1) {
        let prev = out[out.len() - 1];
        let curr = points[i];
        let next = points[i + 1];
        let collinear_x =
            (prev.x - curr.x).abs() <= f32::EPSILON && (curr.x - next.x).abs() <= f32::EPSILON;
        let collinear_y =
            (prev.y - curr.y).abs() <= f32::EPSILON && (curr.y - next.y).abs() <= f32::EPSILON;
        if !(collinear_x || collinear_y) {
            out.push(curr);
        }
    }
    out.push(*points.last().expect("route has endpoint"));
    out
}







