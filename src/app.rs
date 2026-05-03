//! Main application state and UI event handling.

use crate::app::exports::sql_export::build_oracle_sql;
use crate::app::exports::svg_export::{SvgExportOptions, SvgLayoutMode, SvgThemeChoice};
use crate::model::attribute::Attribute;
use crate::model::{entities::domain::Domain, entities::table::Table};
use crate::ui::context::TableUiContext;
use crate::ui::widgets::crow_foot::{build_edges, draw_crow_foot_edge};
pub use command::{Command, CommandQueue};
use egui::{Color32, Id, Key, KeyboardShortcut, Modifiers, vec2};
use gethostname::gethostname;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::fs;

mod command;
pub mod exports;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const MAX_HISTORY_STATES: usize = 100;

#[derive(Default, Clone)]
enum SqlExportModal {
    #[default]
    Hidden,
    Success {
        sql: String,
    },
    Error {
        message: String,
    },
}

#[derive(Default, Clone)]
enum SvgExportModal {
    #[default]
    Hidden,
    Open {
        layout: SvgLayoutMode,
        theme: SvgThemeChoice,
    },
}

/// Undo history item: either an inverse command that can be executed to undo,
/// or a snapshot of the previous app state.
#[derive(Clone)]
enum UndoItem {
    Inverse(Command),
    Snapshot(AppStella),
}

slotmap::new_key_type! {
    /// Unique type for TableIDs (keys)
    pub struct TableId;
}

slotmap::new_key_type! {
/// Unique type for Domain IDs (keys)
    pub struct DomainId;
}

/// Main application state for the ER diagram editor.
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
pub struct AppStella {
    pub tables: SlotMap<TableId, Table>,
    pub domains: SlotMap<DomainId, Domain>,
    #[serde(skip)]
    command_queue: CommandQueue,
    #[serde(skip)]
    sql_export_modal: SqlExportModal,
    #[serde(skip)]
    svg_export_modal: SvgExportModal,
    #[serde(skip)]
    workbench_table_rects: HashMap<TableId, egui::Rect>,
    #[serde(skip)]
    undo_history: Vec<UndoItem>,
    #[serde(skip)]
    redo_history: Vec<UndoItem>,
}

impl AppStella {
    /// Returns the current table collection.
    pub fn tables(&self) -> &SlotMap<TableId, Table> {
        &self.tables
    }

    /// Returns the current domain collection.
    pub fn domains(&self) -> &SlotMap<DomainId, Domain> {
        &self.domains
    }
}

impl AppStella {
    fn draw_highlighted_code(ui: &mut egui::Ui, content: &str, language: &str, rows: usize) {
        let mut view = content.to_owned();
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf.as_str(),
                language,
            );
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut view)
                .desired_width(f32::INFINITY)
                .desired_rows(rows)
                .code_editor()
                .font(egui::TextStyle::Monospace)
                .interactive(true)
                .layouter(&mut layouter),
        );
    }

    /// Restores the app state from persistence storage when available.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app
        } else {
            Default::default()
        }
    }

    /// Enqueues a command to be applied after the current UI frame.
    pub fn dispatch(&mut self, cmd: Command) {
        self.command_queue.push(cmd);
    }

    /// Applies all queued commands in FIFO order.
    pub fn flush_commands(&mut self) {
        if self.command_queue.is_empty() {
            return;
        }

        let commands: Vec<Command> = self.command_queue.drain().collect();
        for cmd in commands {
            self.execute(cmd);
        }
    }

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::Undo => {
                if let Some(item) = self.undo_history.pop() {
                    match item {
                        UndoItem::Inverse(inv_cmd) => {
                            // Apply inverse without recording it to undo_history; compute redo item
                            if let Some(redo_item) = self.apply_inverse_without_recording(inv_cmd) {
                                self.redo_history.push(redo_item);
                            }
                        }
                        UndoItem::Snapshot(prev_state) => {
                            // Push current snapshot to redo and restore previous state
                            let current = self.clone_without_histories();
                            self.redo_history.push(UndoItem::Snapshot(current));
                            self.tables = prev_state.tables;
                            self.domains = prev_state.domains;
                            self.workbench_table_rects = prev_state.workbench_table_rects;
                        }
                    }
                }
            }
            Command::Redo => {
                if let Some(item) = self.redo_history.pop() {
                    match item {
                        UndoItem::Inverse(cmd_to_apply) => {
                            if let Some(undo_item) =
                                self.apply_inverse_without_recording(cmd_to_apply)
                            {
                                self.undo_history.push(undo_item);
                            }
                        }
                        UndoItem::Snapshot(prev_state) => {
                            let current = self.clone_without_histories();
                            self.undo_history.push(UndoItem::Snapshot(current));
                            self.tables = prev_state.tables;
                            self.domains = prev_state.domains;
                            self.workbench_table_rects = prev_state.workbench_table_rects;
                        }
                    }
                }
            }
            other => {
                // For normal commands: try to compute a cheap inverse before applying.
                // If we cannot, fall back to snapshotting the whole state.
                self.redo_history.clear();
                if let Some(pre_inv) = self.compute_inverse_pre(&other) {
                    // We were able to compute inverse based on current state
                    self.execute_command(other);
                    self.push_undo_item(UndoItem::Inverse(pre_inv));
                } else {
                    let snapshot = self.clone_without_histories();
                    self.execute_command(other);
                    self.push_undo_item(UndoItem::Snapshot(snapshot));
                }
            }
        }
    }

    fn push_undo_item(&mut self, item: UndoItem) {
        self.undo_history.push(item);
        if self.undo_history.len() > MAX_HISTORY_STATES {
            self.undo_history.remove(0);
        }
    }

    /// Applies a command without recording to undo_history. Returns an UndoItem that
    /// represents the inverse of the command applied (suitable for pushing to the other stack).
    fn apply_inverse_without_recording(&mut self, cmd: Command) -> Option<UndoItem> {
        // Try to compute a cheap inverse for cmd given current state
        if let Some(pre_inv) = self.compute_inverse_pre(&cmd) {
            self.execute_command(cmd);
            Some(UndoItem::Inverse(pre_inv))
        } else {
            // Snapshot fallback
            let snapshot = self.clone_without_histories();
            self.execute_command(cmd);
            Some(UndoItem::Snapshot(snapshot))
        }
    }

    /// Compute an inverse command from the current state for commands that can be inverted
    /// cheaply. Returns None when snapshot fallback should be used.
    fn compute_inverse_pre(&self, cmd: &Command) -> Option<Command> {
        use Command::*;
        match cmd {
            RenameTable { table, .. } => self.tables.get(*table).map(|t| RenameTable {
                table: *table,
                title: t.title.clone(),
            }),
            RenameAttribute { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| RenameAttribute {
                        table: *table,
                        attr: *attr,
                        name: a.name.clone(),
                    })
                } else {
                    None
                }
            }
            SetAttributeType { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeType {
                        table: *table,
                        attr: *attr,
                        attribute_type: a.attribute_type.clone(),
                    })
                } else {
                    None
                }
            }
            SetAttributeNotNull { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeNotNull {
                        table: *table,
                        attr: *attr,
                        value: a.not_null,
                    })
                } else {
                    None
                }
            }
            SetAttributeUnique { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeUnique {
                        table: *table,
                        attr: *attr,
                        value: a.unique,
                    })
                } else {
                    None
                }
            }
            SetAttributePrimaryKey { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributePrimaryKey {
                        table: *table,
                        attr: *attr,
                        value: a.pk,
                    })
                } else {
                    None
                }
            }
            RenamePrimaryKey { table, .. } => self.tables.get(*table).map(|t| RenamePrimaryKey {
                table: *table,
                name: t.pk.name.clone(),
            }),
            RenameDomain { domain, .. } => self.domains.get(*domain).map(|d| RenameDomain {
                domain: *domain,
                name: d.name.clone(),
            }),
            SetDomainType { domain, .. } => self.domains.get(*domain).map(|d| SetDomainType {
                domain: *domain,
                data_type: d.data_type.clone(),
            }),
            SetForeignKeyReference { table, fk, .. } => self
                .tables
                .get(*table)
                .and_then(|t| t.fks.get(*fk))
                .map(|f| SetForeignKeyReference {
                    table: *table,
                    fk: *fk,
                    references: f.references,
                }),
            RenameUnique { table, index, .. } => self
                .tables
                .get(*table)
                .and_then(|t| t.uniques.get(*index))
                .map(|u| RenameUnique {
                    table: *table,
                    index: *index,
                    name: u.name.clone(),
                }),
            AddUniqueAttribute { table, index, attr } => Some(RemoveUniqueAttribute {
                table: *table,
                index: *index,
                attr: *attr,
            }),
            RemoveUniqueAttribute { table, index, attr } => Some(AddUniqueAttribute {
                table: *table,
                index: *index,
                attr: *attr,
            }),
            _ => None,
        }
    }

    fn clone_without_histories(&self) -> Self {
        AppStella {
            tables: self.tables.clone(),
            domains: self.domains.clone(),
            command_queue: CommandQueue::default(),
            sql_export_modal: SqlExportModal::default(),
            svg_export_modal: SvgExportModal::default(),
            workbench_table_rects: self.workbench_table_rects.clone(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    fn execute_command(&mut self, cmd: Command) {
        match cmd {
            Command::NewCanvas => {
                self.tables.clear();
                self.domains.clear();
                self.workbench_table_rects.clear();
            }
            Command::CreateTable { title } => {
                self.tables.insert(Table {
                    title,
                    ..Default::default()
                });
            }
            Command::DeleteTable { table } => {
                self.tables.remove(table);
            }
            Command::RenameTable { table, title } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.title = title;
                }
            }
            Command::AddAttribute { table, attribute } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.add_field(attribute);
                }
            }
            Command::DeleteAttribute { table, attr } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.attributes.remove(attr);
                    t.pk.attributes.remove(&attr);
                }
            }
            Command::RenameAttribute { table, attr, name } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.name = name;
                }
            }
            Command::SetAttributeType {
                table,
                attr,
                attribute_type,
            } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.attribute_type = attribute_type;
                }
            }
            Command::SetAttributeNotNull { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.not_null = if a.pk { true } else { value };
                }
            }
            Command::SetAttributeUnique { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.unique = value;
                }
            }
            Command::SetAttributePrimaryKey { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table) {
                    if value {
                        t.pk.attributes.insert(attr);
                    } else {
                        t.pk.attributes.remove(&attr);
                    }

                    if let Some(a) = t.attributes.get_mut(attr) {
                        a.pk = value;
                        if value {
                            a.not_null = true;
                        }
                    }
                }
            }
            Command::CreateDomain { name, data_type } => {
                self.domains.insert(Domain {
                    name,
                    data_type,
                    check_constraints: vec![],
                });
            }
            Command::DeleteDomain { domain } => {
                self.domains.remove(domain);
            }
            Command::RenameDomain { domain, name } => {
                if let Some(d) = self.domains.get_mut(domain) {
                    d.name = name;
                }
            }
            Command::SetDomainType { domain, data_type } => {
                if let Some(d) = self.domains.get_mut(domain) {
                    d.data_type = data_type;
                }
            }
            Command::AddForeignKey { table, foreign_key } => {
                if let Some(referenced_table) = foreign_key.references
                    && let Some(pk_snapshot) = self.tables.get(referenced_table).map(|referenced| {
                        referenced
                            .pk
                            .attributes
                            .iter()
                            .filter_map(|attr_id| {
                                referenced
                                    .attributes
                                    .get(*attr_id)
                                    .map(|attr| (*attr_id, attr.name.clone()))
                            })
                            .collect::<Vec<_>>()
                    })
                    && let Some(current_table) = self.tables.get_mut(table)
                {
                    let mut fk = foreign_key;
                    fk.local_attrs.clear();

                    for (other_attr_id, other_attr_name) in pk_snapshot {
                        let local_attr = Attribute {
                            name: format!("{}_{}", fk.name, other_attr_name),
                            attribute_type:
                                crate::model::attribute::AttributeType::ForeignKeyAttribute(
                                    other_attr_id,
                                ),
                            pk: false,
                            not_null: false,
                            unique: false,
                        };
                        let local_attr_key = current_table.attributes.insert(local_attr);
                        fk.local_attrs.insert(local_attr_key);
                    }

                    current_table.add_foreign_key(fk);
                }
            }
            Command::DeleteForeignKey { table, fk } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_foreign_key(fk);
                }
            }
            Command::SetForeignKeyReference {
                table,
                fk,
                references,
            } => {
                if let Some(current_table) = self.tables.get_mut(table)
                    && let Some(foreign_key) = current_table.fks.get_mut(fk)
                {
                    foreign_key.references = references;
                }
            }
            Command::AddUnique { table, unique } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_unique(unique);
                }
            }
            Command::DeleteUnique { table, index } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_unique(index);
                }
            }
            Command::RenameUnique { table, index, name } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.rename_unique(index, name);
                }
            }
            Command::AddUniqueAttribute { table, index, attr } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_unique_attribute(index, attr);
                }
            }
            Command::RemoveUniqueAttribute { table, index, attr } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_unique_attribute(index, attr);
                }
            }
            Command::AddTableCheck { table, check } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_check(check);
                }
            }
            Command::DeleteTableCheck { table, index } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_check(index);
                }
            }
            Command::AddDomainCheck { domain, check } => {
                if let Some(current_domain) = self.domains.get_mut(domain) {
                    current_domain.check_constraints.push(check);
                }
            }
            Command::DeleteDomainCheck { domain, index } => {
                if let Some(current_domain) = self.domains.get_mut(domain)
                    && index < current_domain.check_constraints.len()
                {
                    current_domain.check_constraints.remove(index);
                }
            }
            _ => {}
        }
    }

    /// Saves the current application state to a JSON file.
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = serde_json::to_string(&self)
            && let Err(err) = fs::write(&path, json)
        {
            eprintln!("Error saving file: {}", err);
        }
    }

    /// Loads application state from a JSON file.
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path)
            && let Ok(state) = serde_json::from_str::<AppStella>(&json)
        {
            self.tables = state.tables;
            self.domains = state.domains;
            self.workbench_table_rects = state.workbench_table_rects;
            self.undo_history.clear();
            self.redo_history.clear();
            self.command_queue = CommandQueue::default();
        }
    }

    /// Clears the current canvas and starts a fresh diagram.
    pub fn handle_new(&mut self) {
        self.dispatch(Command::NewCanvas);
        self.flush_commands();
    }

    /// Returns true if an undo operation is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_history.is_empty()
    }

    /// Returns true if a redo operation is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_history.is_empty()
    }

    /// Opens the SVG export modal with default layout and theme choices.
    pub fn open_svg_export_modal(&mut self) {
        self.svg_export_modal = SvgExportModal::Open {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Default,
        };
    }

    /// Opens the SQL export modal and prepares Oracle SQL for the current model.
    pub fn export_sql(&mut self) {
        self.sql_export_modal = match build_oracle_sql(self.tables(), self.domains()) {
            Ok(sql) => SqlExportModal::Success { sql },
            Err(err) => SqlExportModal::Error {
                message: format!("Error exporting SQL: {err}"),
            },
        };
    }

    fn draw_sql_export_modal(&mut self, ctx: &egui::Context) {
        if matches!(self.sql_export_modal, SqlExportModal::Hidden) {
            return;
        }

        let mut close_modal = false;
        let mut save_sql: Option<String> = None;
        let mut copy_sql: Option<String> = None;

        egui::Window::new("Export SQL")
            .id(Id::new("export_sql_modal"))
            .resizable(true)
            .collapsible(false)
            .default_size(vec2(760.0, 420.0))
            .show(ctx, |ui| match &self.sql_export_modal {
                SqlExportModal::Hidden => {}
                SqlExportModal::Success { sql } => {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            Self::draw_highlighted_code(ui, sql, "sql", 12);
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save file").clicked() {
                            save_sql = Some(sql.clone());
                        }
                        if ui.button("Copy to clipboard").clicked() {
                            copy_sql = Some(sql.clone());
                        }
                        if ui.button("Close").clicked() {
                            close_modal = true;
                        }
                    });
                }
                SqlExportModal::Error { message } => {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            Self::draw_highlighted_code(ui, message, "txt", 8);
                        });

                    ui.separator();
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                }
            });

        if let Some(sql) = copy_sql {
            ctx.copy_text(sql);
        }

        if let Some(sql) = save_sql
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("SQL", &["sql"])
                .save_file()
            && let Err(err) = fs::write(path, sql)
        {
            self.sql_export_modal = SqlExportModal::Error {
                message: format!("Error exporting SQL: {err}"),
            };
        }

        if close_modal {
            self.sql_export_modal = SqlExportModal::Hidden;
        }
    }

    fn draw_svg_export_modal(&mut self, ctx: &egui::Context) {
        let (mut layout, mut theme) = match self.svg_export_modal {
            SvgExportModal::Hidden => return,
            SvgExportModal::Open { layout, theme } => (layout, theme),
        };

        let mut close_modal = false;
        let mut save_svg = false;

        egui::Window::new("Export SVG")
            .id(Id::new("export_svg_modal"))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Layout:");
                    ui.selectable_value(&mut layout, SvgLayoutMode::Automatic, "Automatic");
                    ui.selectable_value(&mut layout, SvgLayoutMode::Workbench, "Workbench");
                });

                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ui.selectable_value(&mut theme, SvgThemeChoice::Default, "Default");
                    ui.selectable_value(&mut theme, SvgThemeChoice::Light, "Light");
                    ui.selectable_value(&mut theme, SvgThemeChoice::Dark, "Dark");
                });

                if layout == SvgLayoutMode::Workbench && self.workbench_table_rects.is_empty() {
                    ui.label("No workbench positions.");
                }

                let svg = self.svg_string_with_options(
                    SvgExportOptions { layout, theme },
                    Some(&self.workbench_table_rects),
                    ctx.style().visuals.dark_mode,
                );

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save file").clicked() {
                        save_svg = true;
                    }
                    if ui.button("Copy to clipboard").clicked() {
                        ctx.copy_text(svg.clone());
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });

                if save_svg
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("SVG", &["svg"])
                        .save_file()
                    && let Err(err) = fs::write(path, &svg)
                {
                    eprintln!("Error exporting SVG: {err}");
                }
            });

        if close_modal {
            self.svg_export_modal = SvgExportModal::Hidden;
        } else {
            self.svg_export_modal = SvgExportModal::Open { layout, theme };
        }
    }

    /*fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        add_fonts(&mut fonts);
        ctx.set_fonts(fonts);
    }*/

    fn draw_workbench_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("workbenchmenu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::MenuBar::new().ui(ui, |ui| {
                if ui
                    .add(
                        egui::Button::new("Table")
                            .min_size(vec2(120.0, 25.0))
                            .stroke(egui::Stroke::new(1.0, BLUE)),
                    )
                    .clicked()
                {
                    self.dispatch(Command::CreateTable {
                        title: Table::default().title,
                    });
                }
                if ui
                    .add(
                        egui::Button::new("Domain")
                            .min_size(vec2(120.0, 25.0))
                            .stroke(egui::Stroke::new(1.0, GREEN)),
                    )
                    .clicked()
                {
                    let domain = Domain::default();
                    self.dispatch(Command::CreateDomain {
                        name: domain.name,
                        data_type: domain.data_type,
                    });
                }
            });
            ui.add_space(2.0);
        });
    }

    fn draw_domains_panel(&mut self, ctx: &egui::Context) {
        let mut domain_to_delete: Option<DomainId> = None;
        let mut domain_commands: Vec<Command> = Vec::new();

        egui::SidePanel::right("domains")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Domains");

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (id, domain) in self.domains.iter_mut() {
                        ui.group(|ui| {
                            let changes = domain.draw(ui, id);
                            for cmd in changes.commands {
                                domain_commands.push(cmd);
                            }
                            if changes.name_changed {
                                domain_commands.push(Command::RenameDomain {
                                    domain: id,
                                    name: domain.name.clone(),
                                });
                            }
                            if changes.data_type_changed {
                                domain_commands.push(Command::SetDomainType {
                                    domain: id,
                                    data_type: domain.data_type.clone(),
                                });
                            }
                            if ui.button("🗑").clicked() {
                                domain_to_delete = Some(id);
                            }
                        });
                    }
                });
            });

        for cmd in domain_commands {
            self.dispatch(cmd);
        }
        if let Some(idx) = domain_to_delete {
            self.dispatch(Command::DeleteDomain { domain: idx });
        }
    }

    fn draw_workbench(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");
            let workbench_rect = ui.available_rect_before_wrap().shrink(8.0);

            let mut table_to_delete: Option<TableId> = None;
            let mut table_commands: Vec<Command> = Vec::new();
            let mut table_rects: HashMap<TableId, egui::Rect> = HashMap::new();

            let table_keys: Vec<TableId> = self.tables.keys().collect();

            for id in table_keys {
                let window_id = Id::new(id);
                let title = self.tables[id].title.clone();

                let mut should_delete = false;

                let window = egui::Window::new(title)
                    .id(window_id)
                    .constrain_to(workbench_rect)
                    .resizable(true)
                    .collapsible(true)
                    .default_size(vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        let ui_ctx = TableUiContext::from_app(&self.tables, &self.domains, id);
                        let table = self.tables.get_mut(id).expect("table missing");
                        let changes = table.draw(ui, &ui_ctx, id);
                        for cmd in changes.commands {
                            table_commands.push(cmd);
                        }
                        if changes.title_changed {
                            table_commands.push(Command::RenameTable {
                                table: id,
                                title: table.title.clone(),
                            });
                        }
                        for row in changes.attribute_changes {
                            if row.delete {
                                table_commands.push(Command::DeleteAttribute {
                                    table: id,
                                    attr: row.attr_id,
                                });
                                continue;
                            }

                            if let Some(value) = row.pk_change {
                                table_commands.push(Command::SetAttributePrimaryKey {
                                    table: id,
                                    attr: row.attr_id,
                                    value,
                                });
                            }

                            if let Some(attr) = table.attributes.get(row.attr_id) {
                                if row.rename_changed {
                                    table_commands.push(Command::RenameAttribute {
                                        table: id,
                                        attr: row.attr_id,
                                        name: attr.name.clone(),
                                    });
                                }
                                if row.type_changed {
                                    table_commands.push(Command::SetAttributeType {
                                        table: id,
                                        attr: row.attr_id,
                                        attribute_type: attr.attribute_type.clone(),
                                    });
                                }
                                if row.not_null_changed {
                                    table_commands.push(Command::SetAttributeNotNull {
                                        table: id,
                                        attr: row.attr_id,
                                        value: attr.not_null,
                                    });
                                }
                                if row.unique_changed {
                                    table_commands.push(Command::SetAttributeUnique {
                                        table: id,
                                        attr: row.attr_id,
                                        value: attr.unique,
                                    });
                                }
                            }
                        }
                        if changes.add_attribute {
                            table_commands.push(Command::AddAttribute {
                                table: id,
                                attribute: Attribute::default(),
                            });
                        }

                        ui.separator();
                        if ui.button("Delete").clicked() {
                            should_delete = true;
                        }
                    });

                if let Some(window) = window {
                    table_rects.insert(id, window.response.rect);
                }

                if should_delete {
                    table_to_delete = Some(id);
                }
            }

            let relation_painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Background,
                Id::new("crow_foot_relations"),
            ));
            self.workbench_table_rects = table_rects.clone();
            for edge in build_edges(&self.tables, &table_rects) {
                draw_crow_foot_edge(&relation_painter, &edge);
            }
            for cmd in table_commands {
                self.dispatch(cmd);
            }

            if let Some(idx) = table_to_delete {
                self.dispatch(Command::DeleteTable { table: idx });
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl eframe::App for AppStella {
    /// Renders one frame of the application.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_web = cfg!(target_arch = "wasm32");
        let undo_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Z);
        let redo_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);

        let new_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);
        let open_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
        let save_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);


        if !is_web && self.can_undo() && ctx.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
            self.dispatch(Command::Undo);
        }
        if !is_web && self.can_redo() && ctx.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
            self.dispatch(Command::Redo);
        }
        
        if !is_web && ctx.input_mut(|i| i.consume_shortcut(&new_shortcut)) {
            self.handle_new();
        }
        if !is_web && ctx.input_mut(|i| i.consume_shortcut(&open_shortcut)) {
            if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
                self.handle_open(path);
            }
        }
        if !is_web && ctx.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
            if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
                self.handle_save(path);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.handle_new();
                    }
                    if !is_web
                        && ui.button("Open").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                    {
                        self.handle_open(path);
                    }
                    if !is_web
                        && ui.button("Save").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .save_file()
                    {
                        self.handle_save(path);
                    }
                    if !is_web && ui.button("Export SVG").clicked() {
                        self.open_svg_export_modal();
                    }
                    if !is_web && ui.button("Export SQL").clicked() {
                        self.export_sql();
                    }
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(self.can_undo(), egui::Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        self.dispatch(Command::Undo);
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.can_redo(), egui::Button::new("Redo (Ctrl+R)"))
                        .clicked()
                    {
                        self.dispatch(Command::Redo);
                        ui.close();
                    }
                });
                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let text = "Welcome, ".to_string() + &gethostname().to_string_lossy();
                    ui.label(text);
                })
            });
        });

        self.draw_workbench_menu(ctx);
        self.draw_domains_panel(ctx);

        self.draw_workbench(ctx);
        self.draw_svg_export_modal(ctx);
        self.draw_sql_export_modal(ctx);
        self.flush_commands();
    }
}
