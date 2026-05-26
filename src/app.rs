//! Main application state and UI event handling.

use crate::app::exports::sql::sql_export::{build_sql, SqlDialect};
use crate::app::exports::svg_export::{SvgLayoutMode, SvgThemeChoice};
use crate::model::{entities::domain::Domain, entities::table::Table};
use crate::ui::changes::extend_commands;
pub use command::{Command, CommandQueue};
use eframe::Storage;
use egui::{Color32, Key, KeyboardShortcut, Modifiers};
#[cfg(not(target_arch = "wasm32"))]
use gethostname::gethostname;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use slotmap::SlotMap;
use std::collections::HashMap;

mod command;
pub mod exports;
mod persistence;

pub const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
pub const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const MAX_HISTORY_STATES: usize = 100;

#[derive(Default, Clone)]
pub(crate) enum SqlExportModal {
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
pub(crate) enum SvgExportModal {
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
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AppStella {
    pub tables: SlotMap<TableId, Table>,
    pub domains: SlotMap<DomainId, Domain>,
    #[serde(default)]
    pub(crate) selected_sql_dialect: SqlDialect,
    #[serde(skip)]
    command_queue: CommandQueue,
    #[serde(skip)]
    pub(crate) sql_export_modal: SqlExportModal,
    #[serde(skip)]
    pub(crate) svg_export_modal: SvgExportModal,
    #[serde(skip)]
    pub workbench_table_rects: HashMap<TableId, egui::Rect>,
    #[serde(skip)]
    pub workbench_pan: egui::Vec2,
    #[serde(skip)]
    pub workbench_zoom: f32,
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

impl Default for AppStella {
    fn default() -> Self {
        Self {
            tables: SlotMap::with_key(),
            domains: SlotMap::with_key(),
            command_queue: CommandQueue::default(),
            sql_export_modal: SqlExportModal::default(),
            svg_export_modal: SvgExportModal::default(),
            selected_sql_dialect: SqlDialect::default(),
            workbench_table_rects: HashMap::new(),
            workbench_pan: egui::Vec2::ZERO,
            workbench_zoom: 1.0,
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
            if app.workbench_zoom == 0.0 {
                app.workbench_zoom = 1.0;
            }
            app
        } else {
            Default::default()
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

    /// Opens the SQL export modal and prepares SQL for the selected dialect.
    pub fn export_sql(&mut self) {
        self.sql_export_modal =
            match build_sql(self.selected_sql_dialect, self.tables(), self.domains()) {
                Ok(sql) => SqlExportModal::Success { sql },
                Err(err) => SqlExportModal::Error {
                    message: format!("Error exporting SQL: {err}"),
                },
            };
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
                            extend_commands(&mut domain_commands, changes);
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
}

impl eframe::App for AppStella {
    /// Renders one frame of the application.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.options_mut(|options| {
            options.zoom_with_keyboard = true;
        });

        let undo_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Z);
        let redo_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);

        let new_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);
        let open_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
        let save_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);

        let new_table_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::T);
        let new_domain_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::D);

        if ctx.input_mut(|i| i.consume_shortcut(&new_table_shortcut)) {
            self.dispatch(Command::CreateTable {
                title: Table::default().title,
            })
        }

        if ctx.input_mut(|i| i.consume_shortcut(&new_domain_shortcut)) {
            self.dispatch(Command::CreateDomain {
                name: "".to_string(),
                data_type: Default::default(),
            })
        }

        if self.can_undo() && ctx.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
            self.dispatch(Command::Undo);
        }
        if self.can_redo() && ctx.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
            self.dispatch(Command::Redo);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&new_shortcut)) {
            self.handle_new();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_shortcut(&open_shortcut))
            && let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file()
        {
            self.handle_open(path);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_shortcut(&save_shortcut))
            && let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).save_file()
        {
            self.handle_save(path);
        }

        self.draw_workbench(ctx);

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

                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Save").clicked()
                        && let Some(path) =
                            FileDialog::new().add_filter("JSON", &["json"]).save_file()
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
                });
                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    let text = format!("Welcome, {}", gethostname().to_string_lossy());

                    #[cfg(target_arch = "wasm32")]
                    let text = "Welcome, web user".to_string();

                    ui.label(text);
                });
            });
        });

        self.draw_workbench_menu(ctx);
        self.draw_domains_panel(ctx);
        self.draw_svg_export_modal(ctx);
        self.draw_sql_export_modal(ctx);
        self.flush_commands();
    }

    /// Autopersistance
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
