//! SVG export scene model and renderer for ER diagrams.
//!
//! This module converts the in-memory database schema into a scalable vector graphics (SVG)
//! representation suitable for documentation, printing, or sharing. It supports automatic
//! grid-based table layout, workbench position reuse, light/dark themes, and crow's-foot
//! relationship notation with orthogonal edge routing.

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
///
/// Determines which edge of a table card a relationship line attaches to.
/// Used by the anchor distribution algorithm to spread multiple connections
/// evenly along a side and avoid visual overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Side {
    /// Left edge of the table card.
    Left,
    /// Right edge of the table card.
    Right,
    /// Top edge of the table card.
    Top,
    /// Bottom edge of the table card.
    Bottom,
}

/// Chooses how table cards are positioned in the exported SVG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgLayoutMode {
    /// Places tables in a multi-column grid with automatic spacing.
    ///
    /// Tables are arranged left-to-right, top-to-bottom in rows of three.
    /// Row height is determined by the tallest table in that row.
    Automatic,
    /// Reuses the current workbench window positions from the live editor.
    ///
    /// Each table is placed at the same screen coordinates it occupied in the GUI.
    /// Falls back to automatic layout if workbench positions are unavailable.
    Workbench,
}

/// Selects the color theme used for the exported SVG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgThemeChoice {
    /// Follows the application's current dark/light mode setting.
    Default,
    /// Forces a light background with dark text.
    Light,
    /// Forces a dark background with light text.
    Dark,
}

/// Options controlling SVG export layout and theme.
///
/// Passed to [`AppStella::svg_string_with_options`] to customize the output.
#[derive(Clone, Copy, Debug)]
pub struct SvgExportOptions {
    /// Layout mode for table placement.
    pub layout: SvgLayoutMode,
    /// Theme choice used when rendering the SVG.
    pub theme: SvgThemeChoice,
}

impl Default for SvgExportOptions {
    /// Returns default options: automatic layout with the application's current theme.
    fn default() -> Self {
        Self {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Default,
        }
    }
}

/// Internal color palette for a specific theme variant.
///
/// Contains hex color strings for all visual elements in the diagram.
#[derive(Clone, Copy, Debug)]
struct SvgTheme {
    /// Fill color for table card backgrounds.
    table_fill: &'static str,
    /// Stroke color for table card borders.
    table_stroke: &'static str,
    /// Text color for table titles.
    title_text: &'static str,
    /// Text color for attribute names, types, and inline constraints.
    body_text: &'static str,
    /// Text color for the "Table constraints" section header.
    section_title_text: &'static str,
    /// Text color for individual table-level constraint descriptions.
    constraint_text: &'static str,
}

impl SvgTheme {
    /// Returns the dark theme palette.
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

    /// Returns the light theme palette.
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

/// Resolves a [`SvgThemeChoice`] into a concrete [`SvgTheme`].
///
/// If [`SvgThemeChoice::Default`] is selected, the theme is determined by
/// the application's current dark mode state.
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
///
/// This intermediate representation separates model-to-scene conversion
/// from actual SVG XML generation, enabling testing and alternative renderers.
#[derive(Clone, Debug)]
pub struct SvgScene {
    /// Canvas width in pixels.
    pub width: f32,
    /// Canvas height in pixels.
    pub height: f32,
    /// All table cards in the diagram, positioned on the canvas.
    pub tables: Vec<SvgTableNode>,
    /// All relationship edges in the diagram, with routing waypoints.
    pub relations: Vec<SvgRelationEdge>,
}

/// A table card rendered into the SVG scene.
///
/// Contains all visual information needed to draw one table, including
/// its title, attributes, constraints, and screen rectangle.
#[derive(Clone, Debug)]
pub struct SvgTableNode {
    /// Unique identifier for the table (matches the model's [`TableId`]).
    pub id: TableId,
    /// Table name displayed as the card header.
    pub title: String,
    /// All attribute rows shown in the body of the card.
    pub attributes: Vec<SvgAttributeRow>,
    /// All table-level constraints shown below the attribute list.
    pub table_constraints: Vec<SvgTableConstraintRow>,
    /// Position and dimensions of the table card on the canvas.
    pub rect: Rect,
}

/// A single attribute row shown in a table card.
///
/// Formatted for direct SVG text rendering with three columns:
/// name (left), data type (center), inline constraints (right).
#[derive(Clone, Debug)]
pub struct SvgAttributeRow {
    /// Column/attribute name.
    pub name: String,
    /// Data type as a displayable string (e.g., "VARCHAR2(50)" or "EMAIL").
    pub datatype: String,
    /// Inline constraint flags, comma-separated (e.g., "PK, NN, U" or empty).
    pub constraints: String,
}

/// A formatted table-level constraint row.
///
/// Represents constraints that span multiple columns (PK, UQ, NN, FK, CHECK)
/// and are displayed in a separate section below the attributes.
#[derive(Clone, Debug)]
pub struct SvgTableConstraintRow {
    /// Formatted constraint description (e.g., "FK ref_student (id_student) -> student").
    pub text: String,
}

/// A relationship edge rendered with crow's-foot notation.
///
/// Connects two table cards with an orthogonal polyline route and
/// cardinality symbols at each endpoint.
#[derive(Clone, Debug)]
pub struct SvgRelationEdge {
    /// Starting anchor point on the source table card.
    pub from: Pos2,
    /// Ending anchor point on the target table card.
    pub to: Pos2,
    /// Routing path waypoints between the endpoints, including start and end.
    ///
    /// The route is orthogonal (only horizontal and vertical segments).
    pub route: Vec<Pos2>,
    /// Cardinality at the source end (e.g., 0..1, 1..N).
    pub from_cardinality: Cardinality,
    /// Cardinality at the target end.
    pub to_cardinality: Cardinality,
    /// Relationship type: identifying (solid line) or non-identifying (dashed).
    pub kind: RelationshipKind,
    /// Color of the edge line and endpoint symbols.
    pub color: Color32,
}

impl AppStella {
    /// Writes the current model to an SVG file using the default export settings.
    ///
    /// Convenience wrapper around [`to_svg_with_options`] with automatic layout
    /// and the application's current theme. Errors are printed to stderr.
    ///
    /// # Arguments
    /// * `path` — File path where the SVG will be written.
    pub fn to_svg(&self, path: &str) {
        self.to_svg_with_options(path, SvgExportOptions::default(), None, true);
    }

    /// Writes the current model to an SVG file using explicit export options.
    ///
    /// # Arguments
    /// * `path` — File path where the SVG will be written.
    /// * `options` — Layout and theme configuration.
    /// * `workbench_rects` — Optional map of table positions from the live editor.
    /// * `default_dark_mode` — Whether the application is currently in dark mode.
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
    ///
    /// This is the main entry point for SVG generation. It builds the scene
    /// and then renders it to XML.
    ///
    /// # Arguments
    /// * `options` — Layout and theme configuration.
    /// * `workbench_rects` — Optional map of table positions from the live editor.
    /// * `default_dark_mode` — Whether the application is currently in dark mode.
    ///
    /// # Returns
    /// A complete SVG document as an XML string.
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

/// Builds an SVG scene from the current application state using automatic layout.
///
/// Convenience wrapper that always uses the automatic grid layout.
pub fn model_to_svg_scene(app: &AppStella) -> SvgScene {
    model_to_svg_scene_with_layout(app, SvgLayoutMode::Automatic, None)
}

/// Builds an SVG scene using the requested layout and optional workbench rects.
///
/// # Arguments
/// * `app` — The application state containing tables, domains, and relationships.
/// * `layout` — Whether to use automatic grid layout or reuse workbench positions.
/// * `workbench_rects` — Optional map of table screen rectangles from the live editor.
///
/// # Returns
/// A fully constructed [`SvgScene`] ready for rendering to XML.
pub fn model_to_svg_scene_with_layout(
    app: &AppStella,
    layout: SvgLayoutMode,
    workbench_rects: Option<&HashMap<TableId, Rect>>,
) -> SvgScene {
    let mut tables = match (layout, workbench_rects) {
        (SvgLayoutMode::Workbench, Some(rects)) => {
            map_tables_to_nodes_with_rects(app.tables(), app.domains(), rects)
        }
        _ => map_tables_to_nodes(app.tables(), app.domains()),
    };

    let mut rects = HashMap::new();

    for node in &tables {
        rects.insert(node.id, node.rect);
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

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for node in &tables {
        min_x = min_x.min(node.rect.left());
        min_y = min_y.min(node.rect.top());
        max_x = max_x.max(node.rect.right());
        max_y = max_y.max(node.rect.bottom());
    }

    for rel in &relations {
        min_x = min_x.min(rel.from.x).min(rel.to.x);
        min_y = min_y.min(rel.from.y).min(rel.to.y);
        max_x = max_x.max(rel.from.x).max(rel.to.x);
        max_y = max_y.max(rel.from.y).max(rel.to.y);

        for point in &rel.route {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 0.0;
        max_y = 0.0;
    }

    let padding = 60.0;
    let offset = vec2(padding - min_x, padding - min_y);

    for node in &mut tables {
        node.rect = node.rect.translate(offset);
    }
    for rel in &mut relations {
        rel.from += offset;
        rel.to += offset;
        rel.route = rel.route.iter().map(|p| *p + offset).collect();
    }

    SvgScene {
        width: ((max_x - min_x) + padding * 2.0).max(840.0),
        height: ((max_y - min_y) + padding * 2.0).max(520.0),
        tables,
        relations,
    }
}

/// Estimates the minimum width needed to display a table without text overflow.
///
/// Calculates pixel width needed for:
/// - Title text (scaled by 0.45x in SVG rendering)
/// - Attribute row widths (name + datatype + inline constraints with spacing)
/// - Table constraint row widths (which start at name_x and flow right)
///
fn estimate_table_width(
    title: &str,
    attributes: &[SvgAttributeRow],
    constraints: &[SvgTableConstraintRow],
) -> f32 {
    // Approximate character pixel widths at SVG font sizes
    const CHAR_WIDTH_TITLE: f32 = 18.0; // font-size 30, scaled to 0.45 = ~13.5, but accounting for non-mono
    const CHAR_WIDTH_BODY: f32 = 7.0; // font-size 12
    const CHAR_WIDTH_CONSTRAINT: f32 = 6.5; // font-size 11

    // Padding: left (12px) + right (12px)
    const HORIZONTAL_PADDING: f32 = 24.0;

    // Minimum width floor
    const MIN_WIDTH: f32 = 200.0;

    // Estimate title width (with scale 0.45 applied in render_table)
    let title_width = (title.len() as f32 * CHAR_WIDTH_TITLE * 0.45).max(80.0);

    // Estimate attribute rows: name + type + constraints with column spacing
    let max_attr_width = attributes
        .iter()
        .map(|attr| {
            let name_w = attr.name.len() as f32 * CHAR_WIDTH_BODY;
            let type_w = attr.datatype.len() as f32 * CHAR_WIDTH_BODY;
            let constraint_w = attr.constraints.len() as f32 * CHAR_WIDTH_BODY;
            // name_x at left (12px), datatype_x at 50% of width, constraints_x at right
            // Very rough: assume name takes ~30%, type takes ~30%, constraints takes ~20%
            // Plus spacing between columns
            name_w + type_w + constraint_w + 60.0
        })
        .fold(0.0_f32, f32::max);

    // Estimate constraint rows (rendered at name_x, flowing right to near right edge)
    let max_constraint_width = constraints
        .iter()
        .map(|c| c.text.len() as f32 * CHAR_WIDTH_CONSTRAINT + 24.0) // +24 for left padding and margin
        .fold(0.0_f32, f32::max);

    title_width
        .max(max_attr_width)
        .max(max_constraint_width)
        .max(MIN_WIDTH)
        + HORIZONTAL_PADDING
}

/// Converts all tables and domains from the model into SVG table nodes with automatic layout.
///
/// Lays out tables in a multi-column grid (3 columns per row) with automatic
/// height and width calculation based on content. Extracts and formats
/// attributes and table-level constraints.
///
/// # Arguments
/// * `tables` — All tables in the model.
/// * `domains` — All domains in the model (used for attribute type resolution).
///
/// # Returns
/// A vector of [`SvgTableNode`] with positioned rectangles and formatted content.
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

        // Calculate dimensions
        let table_width = estimate_table_width(&table.title, &attributes, &table_constraints);
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

        let rect = Rect::from_min_size(pos2(x, y), vec2(table_width, h));
        row_max = row_max.max(h);

        out.push(SvgTableNode {
            id,
            title: table.title.clone(),
            attributes,
            table_constraints,
            rect,
        });

        x += table_width + 40.0; // Add spacing proportional to actual table width
        col += 1;
    }

    out
}

/// Overrides automatic layout positions with workbench screen coordinates.
///
/// Takes nodes produced by [`map_tables_to_nodes`] and replaces their rectangles
/// with the positions from the live editor, if available.
///
/// # Arguments
/// * `tables` — All tables in the model.
/// * `domains` — All domains in the model.
/// * `table_rects` — Map of table IDs to their workbench screen rectangles.
///
/// # Returns
/// A vector of [`SvgTableNode`] with workbench positions where available.
fn map_tables_to_nodes_with_rects(
    tables: &SlotMap<TableId, crate::model::entities::table::Table>,
    domains: &SlotMap<DomainId, Domain>,
    table_rects: &HashMap<TableId, Rect>,
) -> Vec<SvgTableNode> {
    let mut nodes = map_tables_to_nodes(tables, domains);
    for node in &mut nodes {
        if let Some(rect) = table_rects.get(&node.id) {
            // Preserve the automatically calculated minimum width/height
            // but allow the workbench rect to provide position and optionally
            // a larger canvas size. This prevents a too-small workbench
            // rectangle from causing content overflow while still respecting
            // the user's placement.
            let min_rect = node.rect;
            let new_width = rect.width().max(min_rect.width());
            let new_height = rect.height().max(min_rect.height());
            node.rect =
                Rect::from_min_size(pos2(rect.left(), rect.top()), vec2(new_width, new_height));
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
/// * `attr` — The attribute to format.
/// * `domains` — Domain collection for type resolution.
///
/// # Returns
/// A formatted [`SvgAttributeRow`] ready for rendering.
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

/// Renders an SVG scene to XML string format using the dark theme.
///
/// Convenience wrapper around [`render_svg_scene_with_theme`].
///
/// # Arguments
/// * `scene` — The SVG scene to render.
///
/// # Returns
/// A complete SVG document as a string.
pub fn render_svg_scene(scene: &SvgScene) -> String {
    render_svg_scene_with_theme(scene, SvgTheme::dark())
}

/// Renders an SVG scene to XML string format with a specific theme.
///
/// Produces valid SVG markup that can be written to a file or displayed in a viewer.
/// Includes proper XML declaration, SVG header with viewBox, and grouped elements
/// for relations and tables.
///
/// # Arguments
/// * `scene` — The SVG scene to render.
/// * `theme` — Color palette to use.
///
/// # Returns
/// A complete SVG document as an XML string.
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
///
/// # Arguments
/// * `table` — The table node to render.
/// * `theme` — Color palette for fill, stroke, and text colors.
///
/// # Returns
/// An SVG [`Group`] containing the complete table card.
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
        let name_style = SvgTextStyle::new(theme.body_text, 12, "start").with_baseline("middle");
        group = group.add(svg_text(name_x, y, &attr.name, name_style));

        let type_style = SvgTextStyle::new(theme.body_text, 12, "middle").with_baseline("middle");
        group = group.add(svg_text(datatype_x, y, &attr.datatype, type_style));

        if !attr.constraints.is_empty() {
            let constraint_style =
                SvgTextStyle::new(theme.body_text, 12, "end").with_baseline("middle");
            group = group.add(svg_text(
                constraints_x,
                y,
                &attr.constraints,
                constraint_style,
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
        let section_style = SvgTextStyle::new(theme.section_title_text, 11, "start")
            .with_baseline("middle")
            .with_weight("600");
        group = group.add(svg_text(name_x, y, "Table constraints", section_style));

        y += 16.0;
        for constraint in &table.table_constraints {
            let constraint_style =
                SvgTextStyle::new(theme.constraint_text, 11, "start").with_baseline("middle");
            group = group.add(svg_text(name_x, y, &constraint.text, constraint_style));
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
/// * `table` — The table to extract constraints from.
/// * `tables` — All tables (used for FK reference resolution).
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
/// Names are returned in the table's explicit attribute order if available,
/// otherwise sorted alphabetically for stable output.
///
/// # Arguments
/// * `table` — The table containing the attributes.
/// * `ids` — Set of attribute IDs to resolve.
///
/// # Returns
/// A vector of attribute names in display order.
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
        table
            .attr_order
            .iter()
            .filter(|id| ids.contains(id))
            .filter_map(|id| table.attributes.get(*id).map(|attr| attr.name.clone()))
            .collect()
    }
}

/// Renders a single relationship edge with crow's-foot notation.
///
/// Produces SVG elements for the line, endpoints with cardinality symbols
/// (circles, bars, crow's feet), and the appropriate styling.
///
/// # Arguments
/// * `rel` — The relationship edge to render.
///
/// # Returns
/// An SVG [`Group`] containing the complete relationship edge.
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
/// - Circle for "zero" minimum cardinality.
/// - Bar for "one" minimum cardinality.
/// - Crow's foot for "many" maximum cardinality.
/// - Bar for "one" maximum cardinality.
///
/// # Arguments
/// * `point` — Endpoint position on the table card edge.
/// * `direction` — Direction vector along the relationship line (outward from table).
/// * `card` — Cardinality constraint (min and max).
/// * `color` — Stroke color for the endpoint symbols.
///
/// # Returns
/// An SVG [`Group`] containing the endpoint symbols.
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
/// * `points` — Waypoints along the route.
/// * `color` — Line stroke color.
/// * `dashed` — Whether to use a dashed stroke (for non-identifying relationships).
///
/// # Returns
/// An SVG [`Group`] containing the line segments.
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

/// Renders a perpendicular bar symbol (used for "one" cardinality).
///
/// # Arguments
/// * `center` — Center point of the bar.
/// * `normal` — Perpendicular direction vector to the relationship line.
/// * `color` — Stroke color.
///
/// # Returns
/// An SVG [`Line`] element.
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

/// Renders a crow's foot symbol (used for "many" cardinality).
///
/// # Arguments
/// * `apex` — Tip of the crow's foot (pointing toward the relationship line).
/// * `direction` — Direction vector along the relationship line.
/// * `normal` — Perpendicular direction vector.
/// * `color` — Stroke color.
///
/// # Returns
/// An SVG [`Group`] containing three line segments.
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

/// Converts an egui [`Color32`] to a hex color string.
///
/// # Arguments
/// * `c` — The color to convert.
///
/// # Returns
/// A hex string in the format `"#RRGGBB"`.
fn color_hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

/// Returns a human-readable label for a cardinality pair.
///
/// # Arguments
/// * `c` — The cardinality constraint.
///
/// # Returns
/// A string like `"0..1"`, `"0 ..N"`, `"1..1"`, or `"1..N"`.
fn cardinality_label(c: Cardinality) -> &'static str {
    match (c.min, c.max) {
        (CardinalityMin::Zero, CardinalityMax::One) => "0..1",
        (CardinalityMin::Zero, CardinalityMax::Many) => "0..N",
        (CardinalityMin::One, CardinalityMax::One) => "1..1",
        (CardinalityMin::One, CardinalityMax::Many) => "1..N",
    }
}

/// Returns a human-readable label for a relationship kind.
///
/// # Arguments
/// * `kind` — The relationship type.
///
/// # Returns
/// `"identifying"` or `"non-identifying"`.
fn relation_kind_label(kind: RelationshipKind) -> &'static str {
    if kind == RelationshipKind::Identifying {
        "identifying"
    } else {
        "non-identifying"
    }
}

/// Removes duplicate relationship edges from the scene.
///
/// Two edges are considered duplicates if they connect the same points
/// with the same kind, cardinalities, and color.
///
/// # Arguments
/// * `relations` — The raw list of edges (may contain duplicates).
///
/// # Returns
/// A deduplicated vector of edges.
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

/// Quantizes a float coordinate for deduplication comparison.
///
/// Rounds to one decimal place to avoid floating-point jitter creating
/// false non-duplicates.
///
/// # Arguments
/// * `v` — The coordinate to quantize.
///
/// # Returns
/// The quantized value as an integer (value × 10).
fn quant(v: f32) -> i32 {
    (v * 10.0).round() as i32
}

/// Renders a simple line segment.
///
/// # Arguments
/// * `a` — Start point.
/// * `b` — End point.
/// * `color` — Stroke color.
///
/// # Returns
/// An SVG [`Line`] element.
fn render_line(a: Pos2, b: Pos2, color: &str) -> Line {
    Line::new()
        .set("x1", a.x)
        .set("y1", a.y)
        .set("x2", b.x)
        .set("y2", b.y)
        .set("stroke", color)
        .set("stroke-width", 1.8)
}

/// Creates an SVG text element with common styling.
///
/// # Arguments
/// * `x` — X coordinate.
/// * `y` — Y coordinate.
/// * `fill` — Text color (hex string).
/// * `font_size` — Font size in pixels.
/// * `anchor` — Text anchor (`"start"`, `"middle"`, or `"end"`).
/// * `dominant_baseline` — Optional dominant baseline alignment.
/// * `font_weight` — Optional font weight (e.g., `"600"`).
/// * `content` — The text content.
///
/// # Returns
/// An SVG [`SvgText`] element.
/// Options for styling SVG text elements.
#[derive(Clone, Copy)]
struct SvgTextStyle {
    fill: &'static str,
    font_size: u32,
    anchor: &'static str,
    dominant_baseline: Option<&'static str>,
    font_weight: Option<&'static str>,
}

impl SvgTextStyle {
    fn new(fill: &'static str, font_size: u32, anchor: &'static str) -> Self {
        Self {
            fill,
            font_size,
            anchor,
            dominant_baseline: None,
            font_weight: None,
        }
    }

    fn with_baseline(mut self, baseline: &'static str) -> Self {
        self.dominant_baseline = Some(baseline);
        self
    }

    fn with_weight(mut self, weight: &'static str) -> Self {
        self.font_weight = Some(weight);
        self
    }
}

/// Creates an SVG text element with the given position and style.
fn svg_text(x: f32, y: f32, content: &str, style: SvgTextStyle) -> SvgText {
    let mut text = SvgText::new(content.to_owned())
        .set("x", x)
        .set("y", y)
        .set("fill", style.fill)
        .set("font-family", "sans-serif")
        .set("font-size", style.font_size)
        .set("text-anchor", style.anchor);

    if let Some(baseline) = style.dominant_baseline {
        text = text.set("dominant-baseline", baseline);
    }
    if let Some(weight) = style.font_weight {
        text = text.set("font-weight", weight);
    }

    text
}

/// Distributes relationship anchor points evenly along table card edges.
///
/// When multiple edges connect to the same side of a table, they are spaced
/// evenly to avoid overlap and improve readability.
///
/// # Arguments
/// * `edges` — All relationship edges (modified in place).
/// * `rects` — Map of table IDs to their screen rectangles.
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

/// Determines which table card side a point lies on.
///
/// Uses a small epsilon tolerance to handle floating-point imprecision.
///
/// # Arguments
/// * `point` — The point to test.
/// * `rects` — Map of table IDs to their screen rectangles.
///
/// # Returns
/// `Some((table_id, side))` if the point is on a table edge, `None` otherwise.
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

/// Returns the coordinate used for sorting endpoints on a shared side.
///
/// When distributing anchors along a vertical side (left/right), endpoints
/// are sorted by their Y coordinate. For horizontal sides (top/bottom),
/// they are sorted by X.
///
/// # Arguments
/// * `edge` — The relationship edge.
/// * `is_from` — Whether we're looking at the `from` endpoint.
/// * `side` — Which side of the table the endpoint is on.
///
/// # Returns
/// The coordinate value used for sorting.
fn opposite_coord(edge: &CrowFootEdge, is_from: bool, side: Side) -> f32 {
    let other = if is_from { edge.to } else { edge.from };
    match side {
        Side::Left | Side::Right => other.y,
        Side::Top | Side::Bottom => other.x,
    }
}

/// Calculates an evenly-spaced anchor point along a table card edge.
///
/// Divides the available edge length (minus padding) into `count + 1` segments
/// and places the anchor at the `slot` position.
///
/// # Arguments
/// * `rect` — The table card rectangle.
/// * `side` — Which edge to place the anchor on.
/// * `slot` — Zero-based index of this anchor among all anchors on this side.
/// * `count` — Total number of anchors on this side.
///
/// # Returns
/// The calculated anchor position.
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

/// Returns the outward normal vector for a given side.
///
/// Used to calculate stub points that extend beyond the table card edge
/// before the orthogonal routing begins.
///
/// # Arguments
/// * `side` — The rectangle side.
///
/// # Returns
/// A unit vector pointing outward from the table.
fn side_outward(side: Side) -> Vec2 {
    match side {
        Side::Left => vec2(-1.0, 0.0),
        Side::Right => vec2(1.0, 0.0),
        Side::Top => vec2(0.0, -1.0),
        Side::Bottom => vec2(0.0, 1.0),
    }
}

/// Routes an orthogonal path between two table card anchors.
///
/// Extends the start and end points with short stubs, then finds the best
/// orthogonal path through candidate waypoints.
///
/// # Arguments
/// * `from` — Starting anchor point.
/// * `to` — Ending anchor point.
/// * `rects` — All table card rectangles (used for obstacle avoidance).
///
/// # Returns
/// A vector of waypoints including the original start and end points.
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

/// Finds the best orthogonal route between two stub points.
///
/// Evaluates multiple candidate paths (L-shapes, Z-shapes, and lane-based routes)
/// and selects the one with the lowest score (shortest length, fewest bends,
/// no collisions with other table cards).
///
/// # Arguments
/// * `from` — Start stub point.
/// * `to` — End stub point.
/// * `rects` — All table card rectangles.
/// * `from_id` — Optional ID of the source table (excluded from collision detection).
/// * `to_id` — Optional ID of the target table (excluded from collision detection).
///
/// # Returns
/// The best route as a vector of waypoints.
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

/// Scores a candidate route for quality.
///
/// Lower scores are better. Penalizes:
/// - Collisions with other table cards (massive penalty).
/// - Long total path length.
/// - Excessive bends (corners).
///
/// # Arguments
/// * `points` — Waypoints of the candidate route.
/// * `rects` — All table card rectangles.
/// * `from_id` — Optional source table ID (excluded from collision).
/// * `to_id` — Optional target table ID (excluded from collision).
///
/// # Returns
/// The numerical score (lower is better).
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

/// Tests whether an axis-aligned line segment intersects a rectangle (with margin).
///
/// Only handles perfectly horizontal or vertical segments, which is sufficient
/// for orthogonal routing.
///
/// # Arguments
/// * `a` — Segment start point.
/// * `b` — Segment end point.
/// * `rect` — The rectangle to test against.
/// * `margin` — Extra padding around the rectangle.
///
/// # Returns
/// `true` if the segment hits the expanded rectangle.
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

/// Removes redundant waypoints from an orthogonal route.
///
/// Deletes consecutive duplicate points and collinear middle points
/// (where three points form a straight line).
///
/// # Arguments
/// * `points` — The raw waypoint list.
///
/// # Returns
/// A simplified route with no unnecessary waypoints.
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
