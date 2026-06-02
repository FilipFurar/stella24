//! Main application state and UI event handling.

use crate::app::exports::sql::sql_export::{SqlDialect, build_sql};
use crate::app::exports::svg_export::{SvgLayoutMode, SvgThemeChoice};
use crate::model::{
    attribute::AttributeType, datatype::DataType, entities::domain::Domain, entities::table::Table,
};
use crate::ui::changes::extend_commands;
pub use command::{Command, CommandQueue};
use eframe::Storage;
use egui::{Color32, KeyboardShortcut, Modifiers, Sense, Stroke, Ui};
#[cfg(not(target_arch = "wasm32"))]
use gethostname::gethostname;

use egui_keybind::{Keybind, Shortcut};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

mod command;
pub mod exports;
mod persistence;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);
const CHECK_COLOR: Color32 = Color32::from_rgb(149, 117, 205);



const MAX_HISTORY_STATES: usize = 100;

#[derive(Clone, Default)]
pub struct Modals {
    pub sql_export_modal: SqlExportModal,
    pub svg_export_modal: SvgExportModal,
    pub project_settings_modal: ProjectSettingsModal,
    pub preferences_modal: PreferencesModal,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppColors {
    #[serde(default)]
    pub tables_color: Color32,
    #[serde(default)]
    pub domains_color: Color32,
    #[serde(default)]
    pub pk_color: Color32,
    #[serde(default)]
    pub fk_color: Color32,
    #[serde(default)]
    pub uq_color: Color32,
    #[serde(default)]
    pub chck_color: Color32,
}

impl AppColors {
    pub fn draw_color_pickers(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Tables");
            ui.color_edit_button_srgba(&mut self.tables_color);
        });

        ui.horizontal(|ui| {
            ui.label("Domains");
            ui.color_edit_button_srgba(&mut self.domains_color);
        });

        ui.horizontal(|ui| {
            ui.label("PKs");
            ui.color_edit_button_srgba(&mut self.pk_color);
        });

        ui.horizontal(|ui| {
            ui.label("FKs");
            ui.color_edit_button_srgba(&mut self.fk_color);
        });

        ui.horizontal(|ui| {
            ui.label("Uniques");
            ui.color_edit_button_srgba(&mut self.uq_color);
        });

        ui.horizontal(|ui| {
            ui.label("Checks");
            ui.color_edit_button_srgba(&mut self.chck_color);
        });
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct KeyBinds {
    #[serde(default)]
    pub new_file: Shortcut,
    #[serde(default)]
    pub open_file: Shortcut,
    #[serde(default)]
    pub save_file: Shortcut,
    #[serde(default)]
    pub save_file_as: Shortcut,
    #[serde(default)]
    pub export_svg: Shortcut,
    #[serde(default)]
    pub export_sql: Shortcut,
    #[serde(default)]
    pub quit: Shortcut,
    #[serde(default)]
    pub new_table: Shortcut,
    #[serde(default)]
    pub new_domain: Shortcut,
    #[serde(default)]
    pub undo: Shortcut,
    #[serde(default)]
    pub redo: Shortcut,
}

impl KeyBinds {
    pub fn draw_keybinds(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("New file");
            ui.add(
                Keybind::new(&mut self.new_file, "new_file_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::N,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Open file");
            ui.add(
                Keybind::new(&mut self.open_file, "open_file_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::O,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Save file");
            ui.add(
                Keybind::new(&mut self.save_file, "save_file_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::S,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Save file as");
            ui.add(
                Keybind::new(&mut self.save_file_as, "save_file_as_keybind")
                    .with_reset(Shortcut::new(
                        Some(KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL | Modifiers::SHIFT,
                            logical_key: egui::Key::S,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Export SVG");
            ui.add(
                Keybind::new(&mut self.export_svg, "export_svg_keybind")
                    .with_reset(Shortcut::new(None, None))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Export SQL");
            ui.add(
                Keybind::new(&mut self.export_sql, "export_sql_keybind")
                    .with_reset(Shortcut::new(None, None))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Quit");
            ui.add(
                Keybind::new(&mut self.quit, "quit_keybind")
                    .with_reset(Shortcut::new(None, None))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("New table");
            ui.add(
                Keybind::new(&mut self.new_table, "new_table_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::T,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("New domain");
            ui.add(
                Keybind::new(&mut self.new_domain, "new_domain_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::D,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Undo");
            ui.add(
                Keybind::new(&mut self.undo, "undo_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::Z,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Redo");
            ui.add(
                Keybind::new(&mut self.redo, "redo_keybind")
                    .with_reset(Shortcut::new(
                        Some(egui::KeyboardShortcut {
                            modifiers: egui::Modifiers::CTRL,
                            logical_key: egui::Key::R,
                        }),
                        None,
                    ))
                    .with_reset_key(Some(egui::Key::Escape)),
            );
        });
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Preferences {
    #[serde(default)]
    pub colors: AppColors,
    #[serde(default)]
    pub key_binds: KeyBinds,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            tables_color: BLUE,
            domains_color: GREEN,
            pk_color: RED,
            fk_color: BLUE,
            uq_color: GREEN,
            chck_color: CHECK_COLOR,
        }
    }
}

#[derive(Default, Clone)]
pub enum SqlExportModal {
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
pub enum SvgExportModal {
    #[default]
    Hidden,
    Open {
        layout: SvgLayoutMode,
        theme: SvgThemeChoice,
    },
}

#[derive(Default, Clone)]
pub enum ProjectSettingsModal {
    #[default]
    Hidden,
    Open,
}

#[derive(Default, Clone)]
pub enum PreferencesModal {
    #[default]
    Hidden,
    Open,
}

/// Undo history item: either an inverse command that can be executed to undo,
/// or a snapshot of the previous app state.
#[derive(Clone)]
enum UndoItem {
    Inverse(Command),
    Snapshot(Box<AppStella>),
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
/// Serializable rectangle representation used for persisted workbench layout.
///
/// This stores only primitive values so it remains stable across egui updates
/// and can be embedded directly in app state JSON.
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Default)]
pub struct PersistedRect {
    /// Top-left corner in workbench world coordinates.
    pub min: [f32; 2],
    /// Rectangle width/height in workbench world coordinates.
    pub size: [f32; 2],
}

impl PersistedRect {
    fn from_rect(rect: egui::Rect) -> Self {
        Self {
            min: [rect.min.x, rect.min.y],
            size: [rect.width(), rect.height()],
        }
    }

    fn to_rect(self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.min[0], self.min[1]),
            egui::vec2(self.size[0], self.size[1]),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug)]
/// Serializable mapping between a table id and its workbench rectangle.
pub struct TableLayoutEntry {
    /// Table key for which the layout was captured.
    pub table: TableId,
    /// Persisted rectangle of the table window.
    pub rect: PersistedRect,
}

/// Different SQL dialects can have different settings on top of general project settings
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum SqlDialectSettings {
    Oracle(OracleSettings),
    Sqlite(SqliteSettings),
    Postgres(PostgresSettings),
}

/// Settings for Oracle PL/SQL projects.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct OracleSettings {}

/// Settings for Oracle SQLite projects.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SqliteSettings {}

/// Settings for Oracle PostgreSQL projects.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PostgresSettings {}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ProjectSettings {
    /// Name of the current project.
    #[serde(default)]
    pub project_name: String,

    /// Path for loading/saving project file
    #[serde(default)]
    pub path: PathBuf,

    /// SQL dialect of the current project.
    #[serde(default)]
    pub selected_sql_dialect: SqlDialect,

    /// Specific settings for the selected SQL dialect.
    //#[serde(default)]
    pub dialect_settings: SqlDialectSettings,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            project_name: "".to_string(),
            path: Default::default(),
            selected_sql_dialect: Default::default(),
            dialect_settings: SqlDialectSettings::Oracle(OracleSettings {}),
        }
    }
}

/// Main application state for the ER diagram editor.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AppStella {
    #[serde(default)]
    pub tables: SlotMap<TableId, Table>,
    #[serde(default)]
    pub domains: SlotMap<DomainId, Domain>,
    #[serde(default)]
    pub domain_order: Vec<DomainId>,
    /// Project settings
    #[serde(default)]
    pub settings: ProjectSettings,
    /// App settings (preferences)
    #[serde(default)]
    pub preferences: Preferences,
    /// Persisted workbench layout used for startup/file restore.
    #[serde(default)]
    pub workbench_table_layout: Vec<TableLayoutEntry>,
    #[serde(skip)]
    command_queue: CommandQueue,
    #[serde(skip)]
    pub modals: Modals,
    #[serde(skip)]
    pub workbench_table_rects: HashMap<TableId, egui::Rect>,
    #[serde(skip)]
    pub workbench_pan: egui::Vec2,
    #[serde(skip)]
    pub workbench_zoom: f32,
    #[serde(skip)]
    pub dragged_domain: Option<DomainId>,
    #[serde(skip)]
    pub dragged_domain_from_index: Option<usize>,
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

    /// Normalize any stored datatype parameter vectors to the selected type.
    fn normalize_datatypes(&mut self) {
        for domain in self.domains.values_mut() {
            domain.data_type.normalize_params();
        }

        for table in self.tables.values_mut() {
            for attribute in table.attributes.values_mut() {
                if let AttributeType::Logical(dt) = &mut attribute.attribute_type {
                    dt.normalize_params();
                }
            }
        }
    }

    fn translate_datatypes_to(&mut self, target_dialect: SqlDialect) {
        for domain in self.domains.values_mut() {
            domain.data_type.translate_to(target_dialect);
        }

        for table in self.tables.values_mut() {
            for attribute in table.attributes.values_mut() {
                if let AttributeType::Logical(dt) = &mut attribute.attribute_type {
                    dt.translate_to(target_dialect);
                }
            }
        }
    }

    pub(crate) fn set_sql_dialect(&mut self, target_dialect: SqlDialect) {
        let current = self.settings.selected_sql_dialect;
        if current == target_dialect {
            return;
        }

        self.translate_datatypes_to(target_dialect);
        self.settings.selected_sql_dialect = target_dialect;
        self.settings.dialect_settings = match target_dialect {
            SqlDialect::Oracle => SqlDialectSettings::Oracle(OracleSettings {}),
            SqlDialect::Sqlite => SqlDialectSettings::Sqlite(SqliteSettings {}),
            SqlDialect::Postgres => SqlDialectSettings::Postgres(PostgresSettings {}),
        };
    }
}

impl Default for AppStella {
    fn default() -> Self {
        Self {
            tables: SlotMap::with_key(),
            domains: SlotMap::with_key(),
            domain_order: Vec::new(),
            settings: Default::default(),
            preferences: Default::default(),
            workbench_table_layout: Vec::new(),
            command_queue: CommandQueue::default(),
            modals: Default::default(),
            workbench_table_rects: HashMap::new(),
            workbench_pan: egui::Vec2::ZERO,
            workbench_zoom: 1.0,
            dragged_domain: None,
            dragged_domain_from_index: None,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }
}

impl AppStella {
    /// Restores the app state from persistence storage when available.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.restore_workbench_rects_from_layout();
            if app.workbench_zoom == 0.0 {
                app.workbench_zoom = 1.0;
            }
            app.normalize_datatypes();
            app
        } else {
            Default::default()
        }
    }

    /// Clears the current canvas and starts a fresh diagram.
    pub fn handle_new(&mut self) {
        self.dispatch(Command::NewCanvas);
        self.flush_commands();
        self.modals.project_settings_modal = ProjectSettingsModal::Open;
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
        self.modals.svg_export_modal = SvgExportModal::Open {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Default,
        };
    }

    pub fn open_project_settings_modal(&mut self) {
        self.modals.project_settings_modal = ProjectSettingsModal::Open;
    }

    pub fn open_preferences_modal(&mut self) {
        self.modals.preferences_modal = PreferencesModal::Open;
    }

    /// Opens the SQL export modal and prepares SQL for the selected dialect.
    pub fn export_sql(&mut self) {
        self.modals.sql_export_modal = match build_sql(
            self.settings.selected_sql_dialect,
            self.tables(),
            self.domains(),
        ) {
            Ok(sql) => SqlExportModal::Success { sql },
            Err(err) => SqlExportModal::Error {
                message: format!("Error exporting SQL: {err}"),
            },
        };
    }

    fn draw_domains_panel(&mut self, ctx: &egui::Context) {
        let mut domain_to_delete: Option<DomainId> = None;
        let mut domain_commands: Vec<Command> = Vec::new();

        let domain_order = self.domain_order.clone();
        let mut row_rects: Vec<(DomainId, egui::Rect)> = Vec::new();
        let mut drop_index: Option<usize> = None;

        egui::SidePanel::right("domains")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Domains");

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, id) in domain_order.into_iter().enumerate() {
                        let Some(domain) = self.domains.get_mut(id) else {
                            continue;
                        };

                        let is_dragged = self.dragged_domain == Some(id);
                        let card_response = ui.scope(|ui| {
                            egui::Frame::group(ui.style())
                                .stroke(Stroke::new(1.0, self.preferences.colors.domains_color))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let handle = ui
                                            .add(egui::Label::new("≡").sense(Sense::drag()))
                                            .on_hover_text("Drag to reorder")
                                            .on_hover_cursor(egui::CursorIcon::Grab);
                                        if handle.drag_started() {
                                            self.dragged_domain = Some(id);
                                            self.dragged_domain_from_index = Some(index);
                                        }

                                        ui.strong("Domain");
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("🗑").clicked() {
                                                    domain_to_delete = Some(id);
                                                }
                                            },
                                        );
                                    });

                                    let changes = domain.draw(ui, id, &self.preferences.colors);
                                    extend_commands(&mut domain_commands, changes);
                                });
                        });

                        let rect = card_response.response.rect;
                        row_rects.push((id, rect));

                        if self.dragged_domain.is_some()
                            && !is_dragged
                            && let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos())
                            && rect.contains(pointer_pos)
                        {
                            let center_y = rect.center().y;
                            drop_index = Some(if pointer_pos.y > center_y {
                                index + 1
                            } else {
                                index
                            });
                        }
                    }

                    if let Some(idx) = drop_index {
                        let y = if idx == 0 {
                            row_rects.first().map(|(_, r)| r.top())
                        } else {
                            row_rects
                                .get(idx.saturating_sub(1))
                                .map(|(_, r)| r.bottom())
                        }
                        .unwrap_or_else(|| ui.min_rect().top());

                        if let Some((_, first_rect)) = row_rects.first() {
                            ui.painter().line_segment(
                                [
                                    egui::pos2(first_rect.left(), y),
                                    egui::pos2(first_rect.right(), y),
                                ],
                                Stroke::new(2.0, ui.visuals().selection.bg_fill),
                            );
                        }
                    }
                });
            });

        if self.dragged_domain.is_some() && ctx.input(|i| i.pointer.any_released()) {
            if let Some(from_idx) = self.dragged_domain_from_index {
                let to_idx = drop_index.unwrap_or(from_idx);
                if from_idx != to_idx {
                    let id = self.domain_order.remove(from_idx);
                    let to_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    let to_idx = to_idx.min(self.domain_order.len());
                    self.domain_order.insert(to_idx, id);
                }
            }
            self.dragged_domain = None;
            self.dragged_domain_from_index = None;
        }

        for cmd in domain_commands {
            self.dispatch(cmd);
        }
        if let Some(idx) = domain_to_delete {
            self.dispatch(Command::DeleteDomain { domain: idx });
        }
    }
}

impl AppStella {
    /// Snapshot the current runtime table rectangles into serializable layout entries.
    ///
    /// Call this right before serializing `AppStella` so saved state reflects
    /// the latest drag operations from the workbench.
    pub(crate) fn sync_layout_from_workbench_rects(&mut self) {
        self.workbench_table_layout = self
            .workbench_table_rects
            .iter()
            .map(|(table, rect)| TableLayoutEntry {
                table: *table,
                rect: PersistedRect::from_rect(*rect),
            })
            .collect();
    }

    /// Restore runtime `workbench_table_rects` from the serialized layout payload.
    ///
    /// This is used after loading from eframe storage or JSON files.
    pub(crate) fn restore_workbench_rects_from_layout(&mut self) {
        self.workbench_table_rects = self
            .workbench_table_layout
            .iter()
            .map(|entry| (entry.table, entry.rect.to_rect()))
            .collect();
    }
}

impl eframe::App for AppStella {
    /// Renders one frame of the application.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.options_mut(|options| {
            options.zoom_with_keyboard = true;
        });

        let undo_shortcut = self.preferences.key_binds.undo.keyboard();
        let redo_shortcut = self.preferences.key_binds.redo.keyboard();

        let new_shortcut = self.preferences.key_binds.new_file.keyboard();
        let open_shortcut = self.preferences.key_binds.open_file.keyboard();
        let save_shortcut = self.preferences.key_binds.save_file.keyboard();
        let save_as_shortcut = self.preferences.key_binds.save_file_as.keyboard();

        let export_svg_shortcut = self.preferences.key_binds.export_svg.keyboard();
        let export_sql_shortcut = self.preferences.key_binds.export_sql.keyboard();
        let quit_shortcut = self.preferences.key_binds.quit.keyboard();

        let new_table_shortcut = self.preferences.key_binds.new_table.keyboard();
        let new_domain_shortcut = self.preferences.key_binds.new_domain.keyboard();

        if let Some(shortcut) = new_table_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.dispatch(Command::CreateTable {
                title: Table::default().title,
            })
        }

        if let Some(shortcut) = new_domain_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.dispatch(Command::CreateDomain {
                name: "".to_string(),
                data_type: DataType::default_for_dialect(self.settings.selected_sql_dialect),
            })
        }

        if let Some(shortcut) = undo_shortcut
            && self.can_undo()
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.dispatch(Command::Undo);
        }

        if let Some(shortcut) = redo_shortcut
            && self.can_redo()
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.dispatch(Command::Redo);
        }

        if let Some(shortcut) = new_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.handle_new();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shortcut) = open_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
            && let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file()
        {
            self.handle_open(path);
        }

        let Ok(save_path) = PathBuf::from_str(&format!(
            "{}/{}.json",
            &self.settings.path.display().to_string(),
            &self.settings.project_name
        ));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shortcut) = save_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.handle_save(save_path);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shortcut) = save_as_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
            && let Some(path) = FileDialog::new()
                .set_directory(&self.settings.path)
                .set_file_name(&self.settings.project_name)
                .add_filter("JSON", &["json"])
                .save_file()
        {
            self.handle_save(path);
        }

        if let Some(shortcut) = export_svg_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.open_svg_export_modal();
        }

        if let Some(shortcut) = export_sql_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            self.export_sql();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shortcut) = quit_shortcut
            && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let clrs = &self.preferences.colors.clone();
        self.draw_workbench(ctx, clrs);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.handle_new();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Open").clicked()
                        && let Some(path) =
                            FileDialog::new().add_filter("JSON", &["json"]).pick_file()
                    {
                        self.handle_open(path);
                    }

                    let Ok(save_path) = PathBuf::from_str(&format!(
                        "{}/{}.json",
                        &self.settings.path.display().to_string(),
                        &self.settings.project_name
                    ));
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Save").clicked() {
                        self.handle_save(save_path);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Save as").clicked()
                        && let Some(path) = FileDialog::new()
                            .set_directory(&self.settings.path)
                            .set_file_name(&self.settings.project_name)
                            .add_filter("JSON", &["json"])
                            .save_file()
                    {
                        self.handle_save(path);
                    }

                    if ui.button("Export SVG").clicked() {
                        self.open_svg_export_modal();
                    }

                    if ui.button("Export SQL").clicked() {
                        self.export_sql();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
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
                    if ui.button("Project settings").clicked() {
                        self.open_project_settings_modal();
                    }
                    if ui.button("Preferences").clicked() {
                        self.open_preferences_modal();
                    }
                });

                ui.separator();
                egui::widgets::global_theme_preference_switch(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    let text = format!("Welcome, {}", gethostname().to_string_lossy());

                    #[cfg(target_arch = "wasm32")]
                    let text = "Welcome, web user".to_string();

                    ui.label(text);
                    ui.separator();
                    ui.label(format!("Project: {}", &self.settings.project_name));
                });
            });
        });

        self.draw_workbench_menu(ctx);
        self.draw_svg_export_modal(ctx);
        self.draw_project_settings_modal(ctx);
        self.draw_preferences_modal(ctx);
        self.draw_sql_export_modal(ctx);
        self.draw_domains_panel(ctx);

        self.flush_commands();
    }

    /// Autopersistance
    fn save(&mut self, storage: &mut dyn Storage) {
        self.sync_layout_from_workbench_rects();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
