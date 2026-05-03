//! SVG export scene model and renderer for ER diagrams.

use crate::AppStella;
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::datatype::DATA_TYPES;
use crate::model::entities::domain::Domain;
use crate::ui::widgets::crow_foot::{
    Cardinality, CardinalityMax, CardinalityMin, CrowFootEdge, RelationshipKind, build_edges,
};
use egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::fs;
use svg::Document;
use svg::node::element::{Circle, Group, Line, Rectangle, Text as SvgText};

/// Rectangle side used when placing relationship anchors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// Chooses how table cards are positioned in the exported SVG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgLayoutMode {
    /// Places tables in the automatic grid layout.
    Automatic,
    /// Reuses the current workbench window positions.
    Workbench,
}

/// Selects the color theme used for the exported SVG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgThemeChoice {
    /// Follows the application's current theme.
    Default,
    /// Forces a light SVG theme.
    Light,
    /// Forces a dark SVG theme.
    Dark,
}

/// Options controlling SVG export layout and theme.
#[derive(Clone, Copy, Debug)]
pub struct SvgExportOptions {
    /// Layout mode for table placement.
    pub layout: SvgLayoutMode,
    /// Theme choice used when rendering the SVG.
    pub theme: SvgThemeChoice,
}

impl Default for SvgExportOptions {
    fn default() -> Self {
        Self {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Default,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SvgTheme {
    table_fill: &'static str,
    table_stroke: &'static str,
    title_text: &'static str,
    body_text: &'static str,
    section_title_text: &'static str,
    constraint_text: &'static str,
}

impl SvgTheme {
    fn dark() -> Self {
        Self {
            table_fill: "#2b2b2b",
            table_stroke: "#505050",
            title_text: "#f0f0f0",
            body_text: "#d8d8d8",
            section_title_text: "#bbbbbb",
            constraint_text: "#b8b8b8",
        }
    }

    fn light() -> Self {
        Self {
            table_fill: "#f7f7f7",
            table_stroke: "#9b9b9b",
            title_text: "#181818",
            body_text: "#2a2a2a",
            section_title_text: "#444444",
            constraint_text: "#4f4f4f",
        }
    }
}

fn resolve_theme(choice: SvgThemeChoice, default_dark_mode: bool) -> SvgTheme {
    match choice {
        SvgThemeChoice::Default => {
            if default_dark_mode {
                SvgTheme::dark()
            } else {
                SvgTheme::light()
            }
        }
        SvgThemeChoice::Light => SvgTheme::light(),
        SvgThemeChoice::Dark => SvgTheme::dark(),
    }
}

/// Complete SVG scene with canvas dimensions, tables, and relations.
#[derive(Clone, Debug)]
pub struct SvgScene {
    /// Canvas width in pixels.
    pub width: f32,
    /// Canvas height in pixels.
    pub height: f32,
    /// All table cards in the diagram.
    pub tables: Vec<SvgTableNode>,
    /// All relationship edges in the diagram.
    pub relations: Vec<SvgRelationEdge>,
}

/// A table card rendered into the SVG scene.
#[derive(Clone, Debug)]
pub struct SvgTableNode {
    /// Unique identifier for the table.
    pub id: TableId,
    /// Table name/title.
    pub title: String,
    /// All attribute rows in the table.
    pub attributes: Vec<SvgAttributeRow>,
    /// All table-level constraints.
    pub table_constraints: Vec<SvgTableConstraintRow>,
    /// Position and dimensions of the table card on the canvas.
    pub rect: Rect,
}

/// A single attribute row shown in a table card.
#[derive(Clone, Debug)]
pub struct SvgAttributeRow {
    /// Column/attribute name.
    pub name: String,
    /// Data type as a displayable string.
    pub datatype: String,
    /// Inline constraint flags.
    pub constraints: String,
}

/// A formatted table-level constraint row.
#[derive(Clone, Debug)]
pub struct SvgTableConstraintRow {
    /// Formatted constraint description.
    pub text: String,
}

/// A relationship edge rendered with crow's-foot notation.
#[derive(Clone, Debug)]
pub struct SvgRelationEdge {
    /// Starting point of the edge.
    pub from: Pos2,
    /// Ending point of the edge.
    pub to: Pos2,
    /// Routing path waypoints between the endpoints.
    pub route: Vec<Pos2>,
    /// Cardinality at the source end.
    pub from_cardinality: Cardinality,
    /// Cardinality at the target end.
    pub to_cardinality: Cardinality,
    /// Relationship type.
    pub kind: RelationshipKind,
    /// Color of the edge.
    pub color: Color32,
}

impl AppStella {
    /// Writes the current model to an SVG file using the default export settings.
    pub fn to_svg(&self, path: &str) {
        self.to_svg_with_options(path, SvgExportOptions::default(), None, true);
    }

    /// Writes the current model to an SVG file using explicit export options.
    pub fn to_svg_with_options(
        &self,
        path: &str,
        options: SvgExportOptions,
        workbench_rects: Option<&HashMap<TableId, Rect>>,
        default_dark_mode: bool,
    ) {
        let svg = self.svg_string_with_options(options, workbench_rects, default_dark_mode);
        if let Err(err) = fs::write(path, svg) {
            eprintln!("Error exporting SVG: {err}");
        }
    }

    /// Returns the SVG document as a string for the given export options.
    pub fn svg_string_with_options(
        &self,
        options: SvgExportOptions,
        workbench_rects: Option<&HashMap<TableId, Rect>>,
        default_dark_mode: bool,
    ) -> String {
        let scene = model_to_svg_scene_with_layout(self, options.layout, workbench_rects);
        let theme = resolve_theme(options.theme, default_dark_mode);
        render_svg_scene_with_theme(&scene, theme)
    }
}

/// Builds an SVG scene from the current application state.
pub fn model_to_svg_scene(app: &AppStella) -> SvgScene {
    model_to_svg_scene_with_layout(app, SvgLayoutMode::Automatic, None)
}

/// Builds an SVG scene using the requested layout and optional workbench rects.
pub fn model_to_svg_scene_with_layout(
    app: &AppStella,
    layout: SvgLayoutMode,
    workbench_rects: Option<&HashMap<TableId, Rect>>,
) -> SvgScene {
    let tables = match (layout, workbench_rects) {
        (SvgLayoutMode::Workbench, Some(rects)) => {
            map_tables_to_nodes_with_rects(app.tables(), app.domains(), rects)
        }
        _ => map_tables_to_nodes(app.tables(), app.domains()),
    };

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

/// Converts all tables and domains from the model into SVG table nodes.
///
/// Lays out tables in a multi-column grid (3 columns per row) with automatic
/// height calculation based on attributes and constraints. Extracts and formats
/// attributes and table-level constraints.
///
/// # Arguments
/// * `tables` - All tables in the model
/// * `domains` - All domains in the model (used for attribute type resolution)
///
/// # Returns
/// A vector of `SvgTableNode` with positioned rectangles and formatted content.
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
        let attributes = if table.attr_order.is_empty() {
            table
                .attributes
                .values()
                .map(|a| format_attribute_row(a, domains))
                .collect::<Vec<_>>()
        } else {
            table
                .attr_order
                .iter()
                .filter_map(|&id| table.attributes.get(id))
                .map(|a| format_attribute_row(a, domains))
                .collect::<Vec<_>>()
        };
        let table_constraints = format_table_constraints(table, tables);

        let attr_h = (attributes.len() as f32 * 24.0).max(24.0);
        let constraints_h = if table_constraints.is_empty() {
            0.0
        } else {
            28.0 + table_constraints.len() as f32 * 18.0
        };
        let h = 66.0 + attr_h + constraints_h;
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
            table_constraints,
            rect,
        });

        x += 380.0;
        col += 1;
    }

    out
}

fn map_tables_to_nodes_with_rects(
    tables: &SlotMap<TableId, crate::model::entities::table::Table>,
    domains: &SlotMap<DomainId, Domain>,
    table_rects: &HashMap<TableId, Rect>,
) -> Vec<SvgTableNode> {
    let mut nodes = map_tables_to_nodes(tables, domains);
    for node in &mut nodes {
        if let Some(rect) = table_rects.get(&node.id) {
            node.rect = *rect;
        }
    }
    nodes
}

/// Formats a single attribute into an SVG row representation.
///
/// Extracts the attribute name, data type, and inline constraints (PK, NN, U).
/// Data types are resolved from domains or shown as logical type names with parameters.
///
/// # Arguments
/// * `attr` - The attribute to format
/// * `domains` - Domain collection for type resolution
///
/// # Returns
/// A formatted `SvgAttributeRow` ready for rendering.
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

/// Renders an SVG scene to XML string format.
///
/// Produces valid SVG markup that can be written to a file or displayed in a viewer.
/// Includes proper XML declaration, SVG header with viewBox, and grouped elements
/// for relations and tables.
///
/// # Arguments
/// * `scene` - The SVG scene to render
///
/// # Returns
/// A complete SVG document as a string.
pub fn render_svg_scene(scene: &SvgScene) -> String {
    render_svg_scene_with_theme(scene, SvgTheme::dark())
}

fn render_svg_scene_with_theme(scene: &SvgScene, theme: SvgTheme) -> String {
    let relations = scene
        .relations
        .iter()
        .fold(Group::new().set("class", "relations"), |group, rel| {
            group.add(render_relation(rel))
        });
    let tables = scene
        .tables
        .iter()
        .fold(Group::new().set("class", "tables"), |group, table| {
            group.add(render_table(table, theme))
        });

    let document = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("width", format!("{:.0}", scene.width))
        .set("height", format!("{:.0}", scene.height))
        .set(
            "viewBox",
            format!("0 0 {:.0} {:.0}", scene.width, scene.height),
        )
        .add(relations)
        .add(tables);

    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}\n", document)
}

/// Renders a single table card as SVG elements.
///
/// Produces a rounded-rectangle background with a title header, attribute rows
/// with name/type/constraints columns, and a table-constraints section if present.
/// Uses dark theme colors for accessibility.
///
/// # Arguments
/// * `table` - The table node to render
///
/// # Returns
/// SVG XML string containing the table group and all its content.
fn render_table(table: &SvgTableNode, theme: SvgTheme) -> Group {
    let r = table.rect;
    let mut group = Group::new()
        .set("class", "table")
        .set("data-id", format!("{:?}", table.id));
    group = group.add(
        Rectangle::new()
            .set("x", r.left())
            .set("y", r.top())
            .set("width", r.width())
            .set("height", r.height())
            .set("rx", 8)
            .set("fill", theme.table_fill)
            .set("stroke", theme.table_stroke),
    );

    let title_y = r.top() + 26.0;
    group = group.add(
        SvgText::new(table.title.clone())
            .set("x", (r.left() + 12.0) / 0.45)
            .set("y", title_y / 0.45)
            .set("fill", theme.title_text)
            .set("font-family", "sans-serif")
            .set("font-size", 30)
            .set("font-weight", 700)
            .set("transform", "scale(0.45)"),
    );

    let name_x = r.left() + 12.0;
    let datatype_x = r.left() + r.width() * 0.50;
    let constraints_x = r.right() - 12.0;
    let mut y = r.top() + 54.0;
    for attr in &table.attributes {
        group = group.add(svg_text(
            name_x,
            y,
            theme.body_text,
            12,
            "start",
            Some("middle"),
            None,
            &attr.name,
        ));
        group = group.add(svg_text(
            datatype_x,
            y,
            theme.body_text,
            12,
            "middle",
            Some("middle"),
            None,
            &attr.datatype,
        ));
        if !attr.constraints.is_empty() {
            group = group.add(svg_text(
                constraints_x,
                y,
                theme.body_text,
                12,
                "end",
                Some("middle"),
                None,
                &attr.constraints,
            ));
        }
        y += 18.0;
    }

    if !table.table_constraints.is_empty() {
        let sep_y = y + 4.0;
        group = group.add(
            Line::new()
                .set("x1", r.left() + 8.0)
                .set("y1", sep_y)
                .set("x2", r.right() - 8.0)
                .set("y2", sep_y)
                .set("stroke", theme.table_stroke)
                .set("stroke-width", 1),
        );

        y = sep_y + 14.0;
        group = group.add(svg_text(
            name_x,
            y,
            theme.section_title_text,
            11,
            "start",
            Some("middle"),
            Some("600"),
            "Table constraints",
        ));

        y += 16.0;
        for constraint in &table.table_constraints {
            group = group.add(svg_text(
                name_x,
                y,
                theme.constraint_text,
                11,
                "start",
                Some("middle"),
                None,
                &constraint.text,
            ));
            y += 16.0;
        }
    }

    group
}

/// Extracts and formats table-level constraints for SVG display.
///
/// Collects multi-column PRIMARY KEYs, UNIQUE constraints, NOT NULL constraints,
/// FOREIGN KEYs, and CHECK constraints. Single-column constraints that are already
/// shown inline with attributes are excluded.
///
/// # Arguments
/// * `table` - The table to extract constraints from
/// * `tables` - All tables (used for FK reference resolution)
///
/// # Returns
/// A vector of formatted constraint rows.
fn format_table_constraints(
    table: &crate::model::entities::table::Table,
    tables: &SlotMap<TableId, crate::model::entities::table::Table>,
) -> Vec<SvgTableConstraintRow> {
    let mut rows = Vec::new();

    let pk_attrs = sorted_attr_names(table, &table.pk.attributes);
    if pk_attrs.len() > 1 {
        rows.push(SvgTableConstraintRow {
            text: format!("PK ({})", pk_attrs.join(", ")),
        });
    }

    for unique in &table.uniques {
        let attrs = sorted_attr_names(table, &unique.attributes);
        if attrs.len() > 1 {
            rows.push(SvgTableConstraintRow {
                text: if unique.name.trim().is_empty() {
                    format!("UQ ({})", attrs.join(", "))
                } else {
                    format!("UQ {} ({})", unique.name, attrs.join(", "))
                },
            });
        }
    }

    for not_null in &table.not_nulls {
        let attrs = sorted_attr_names(table, &not_null.attributes);
        if attrs.len() > 1 {
            rows.push(SvgTableConstraintRow {
                text: if not_null.name.trim().is_empty() {
                    format!("NN ({})", attrs.join(", "))
                } else {
                    format!("NN {} ({})", not_null.name, attrs.join(", "))
                },
            });
        }
    }

    for fk in table.fks.values() {
        let local_attrs = sorted_attr_names(table, &fk.local_attrs);
        let target = fk
            .references
            .and_then(|id| tables.get(id))
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "?".to_string());
        rows.push(SvgTableConstraintRow {
            text: if fk.name.trim().is_empty() {
                format!("FK ({}) -> {}", local_attrs.join(", "), target)
            } else {
                format!("FK {} ({}) -> {}", fk.name, local_attrs.join(", "), target)
            },
        });
    }

    for check in &table.checks {
        rows.push(SvgTableConstraintRow {
            text: if check.name.trim().is_empty() {
                format!("CHECK ({})", check.condition)
            } else {
                format!("CHECK {} ({})", check.name, check.condition)
            },
        });
    }

    rows
}

/// Extracts and sorts attribute names from a set of attribute IDs.
///
/// Performs a lookup of attribute names from the table
/// sorted by user
///
/// # Arguments
/// * `table` - The table containing the attributes
/// * `ids` - Set of attribute IDs to resolve
///
/// # Returns
/// A sorted vector of attribute names.
fn sorted_attr_names(
    table: &crate::model::entities::table::Table,
    ids: &HashSet<AttrId>,
) -> Vec<String> {
    if table.attr_order.is_empty() {
        let mut names = ids
            .iter()
            .filter_map(|id| table.attributes.get(*id).map(|attr| attr.name.clone()))
            .collect::<Vec<_>>();
        names.sort();
        names
    } else {
        table.attr_order
            .iter()
            .filter(|id| ids.contains(id))
            .filter_map(|id| table.attributes.get(*id).map(|attr| attr.name.clone()))
            .collect()
    }
}

/// Renders a single relationship edge with crow's foot notation.
///
/// Produces SVG elements for the line, endpoints with cardinality symbols
/// (circles, bars, crow's feet), and the appropriate styling (solid for identifying,
/// dashed for non-identifying).
///
/// # Arguments
/// * `rel` - The relationship edge to render
///
/// # Returns
/// SVG XML string for the complete relationship line with endpoints.
fn render_relation(rel: &SvgRelationEdge) -> Group {
    let color = color_hex(rel.color);
    let mut group = Group::new()
        .set("class", "relation")
        .set("data-kind", relation_kind_label(rel.kind))
        .set(
            "data-from-cardinality",
            cardinality_label(rel.from_cardinality),
        )
        .set("data-to-cardinality", cardinality_label(rel.to_cardinality));

    group = group.add(render_route(
        &rel.route,
        &color,
        rel.kind == RelationshipKind::NonIdentifying,
    ));

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

    group = group.add(render_endpoint(
        rel.from,
        from_dir,
        rel.from_cardinality,
        &color,
    ));
    group = group.add(render_endpoint(rel.to, to_dir, rel.to_cardinality, &color));

    group
}

/// Renders the visual endpoint (cardinality symbol) of a relationship.
///
/// Produces the appropriate SVG geometry for the cardinality constraint:
/// - Circles for "zero" cardinality min
/// - Bars for "one" cardinality min
/// - Crow's feet for "many" cardinality max
/// - Bars for "one" cardinality max
///
/// # Arguments
/// * `point` - Endpoint position
/// * `direction` - Direction vector along the relationship line
/// * `card` - Cardinality constraint (min and max)
/// * `color` - Color for the endpoint
///
/// # Returns
/// SVG XML string for the endpoint symbols.
fn render_endpoint(point: Pos2, mut direction: Vec2, card: Cardinality, color: &str) -> Group {
    if direction == Vec2::ZERO {
        return Group::new();
    }

    direction = direction.normalized();
    let normal = vec2(-direction.y, direction.x);
    let min_pos = point + direction * 7.0;
    let max_pos = point + direction * 15.0;

    let mut group = Group::new();

    match card.min {
        CardinalityMin::Zero => {
            group = group.add(
                Circle::new()
                    .set("cx", min_pos.x)
                    .set("cy", min_pos.y)
                    .set("r", 4)
                    .set("fill", "none")
                    .set("stroke", color)
                    .set("stroke-width", 1.8),
            );
        }
        CardinalityMin::One => {
            group = group.add(render_bar(min_pos, normal, color));
        }
    }

    match card.max {
        CardinalityMax::One => {
            group = group.add(render_bar(max_pos, normal, color));
        }
        CardinalityMax::Many => {
            group = group.add(render_crow_foot(max_pos, direction, normal, color));
        }
    }

    group
}

/// Renders a polyline route connecting multiple points.
///
/// Produces SVG line segments connecting the waypoints in the route,
/// with optional dashing for non-identifying relationships.
///
/// # Arguments
/// * `points` - Waypoints along the route
/// * `color` - Line color
/// * `dash` - Optional stroke-dasharray attribute (empty or " stroke-dasharray=\"4 6\"")
///
/// # Returns
/// SVG XML string for the line segments.
fn render_route(points: &[Pos2], color: &str, dashed: bool) -> Group {
    let mut group = Group::new();
    for win in points.windows(2) {
        let mut line = Line::new()
            .set("x1", win[0].x)
            .set("y1", win[0].y)
            .set("x2", win[1].x)
            .set("y2", win[1].y)
            .set("stroke", color)
            .set("stroke-width", 1.8);
        if dashed {
            line = line.set("stroke-dasharray", "4 6");
        }
        group = group.add(line);
    }
    group
}

fn render_bar(center: Pos2, normal: Vec2, color: &str) -> Line {
    let half = normal * 5.0;
    let a = center - half;
    let b = center + half;
    Line::new()
        .set("x1", a.x)
        .set("y1", a.y)
        .set("x2", b.x)
        .set("y2", b.y)
        .set("stroke", color)
        .set("stroke-width", 1.8)
}

fn render_crow_foot(apex: Pos2, direction: Vec2, normal: Vec2, color: &str) -> Group {
    let root = apex - direction * 8.0;
    let left = root + normal * 6.0;
    let mid = root;
    let right = root - normal * 6.0;

    Group::new()
        .add(render_line(apex, left, color))
        .add(render_line(apex, mid, color))
        .add(render_line(apex, right, color))
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

fn render_line(a: Pos2, b: Pos2, color: &str) -> Line {
    Line::new()
        .set("x1", a.x)
        .set("y1", a.y)
        .set("x2", b.x)
        .set("y2", b.y)
        .set("stroke", color)
        .set("stroke-width", 1.8)
}

fn svg_text(
    x: f32,
    y: f32,
    fill: &str,
    font_size: u32,
    anchor: &str,
    dominant_baseline: Option<&str>,
    font_weight: Option<&str>,
    content: &str,
) -> SvgText {
    let mut text = SvgText::new(content.to_owned())
        .set("x", x)
        .set("y", y)
        .set("fill", fill)
        .set("font-family", "sans-serif")
        .set("font-size", font_size)
        .set("text-anchor", anchor);

    if let Some(dominant_baseline) = dominant_baseline {
        text = text.set("dominant-baseline", dominant_baseline);
    }
    if let Some(font_weight) = font_weight {
        text = text.set("font-weight", font_weight);
    }

    text
}

fn distribute_edge_anchors(edges: &mut [CrowFootEdge], rects: &HashMap<TableId, Rect>) {
    let mut groups: HashMap<(TableId, Side), Vec<(usize, bool)>> = HashMap::new();

    for (idx, edge) in edges.iter().enumerate() {
        if let Some((table_id, side)) = endpoint_table_side(edge.from, rects) {
            groups
                .entry((table_id, side))
                .or_default()
                .push((idx, true));
        }
        if let Some((table_id, side)) = endpoint_table_side(edge.to, rects) {
            groups
                .entry((table_id, side))
                .or_default()
                .push((idx, false));
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
    let mut candidates = vec![
        vec![from, pos2(to.x, from.y), to],
        vec![from, pos2(from.x, to.y), to],
    ];

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
