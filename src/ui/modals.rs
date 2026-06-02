use crate::AppStella;
use crate::app::exports::sql::sql_export::SqlDialect;
use crate::app::exports::svg_export::{SvgExportOptions, SvgLayoutMode, SvgThemeChoice};
use crate::app::{PreferencesModal, ProjectSettingsModal, SqlExportModal, SvgExportModal};
use crate::ui::widgets::code::draw_highlighted_code;
use eframe::emath::vec2;
use egui::Id;
use rfd::FileDialog;
use std::fs;

impl AppStella {
    /// Renders a modal for changing current model's properties
    pub fn draw_project_settings_modal(&mut self, ctx: &egui::Context) {
        if matches!(
            self.modals.project_settings_modal,
            ProjectSettingsModal::Hidden
        ) {
            return;
        }

        let mut close_modal = false;
        let current_sql_dialect = self.settings.selected_sql_dialect;
        let mut selected_sql_dialect = current_sql_dialect;

        egui::Window::new("Project settings")
            .id(Id::new("project_settings_window"))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Project name");
                    ui.text_edit_singleline(&mut self.settings.project_name);
                });

                ui.horizontal(|ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label("Project path");
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label(self.settings.path.display().to_string());
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Browse").clicked()
                        && let Some(path) = FileDialog::new().pick_folder()
                    {
                        self.settings.path = path;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Dialect");
                    egui::ComboBox::from_id_salt("sql_export_dialect")
                        .selected_text(selected_sql_dialect.to_string())
                        .show_ui(ui, |ui| {
                            for dialect in SqlDialect::ALL {
                                ui.selectable_value(
                                    &mut selected_sql_dialect,
                                    dialect,
                                    dialect.label(),
                                );
                            }
                        });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if close_modal {
            self.modals.project_settings_modal = ProjectSettingsModal::Hidden;
        }

        if selected_sql_dialect != current_sql_dialect {
            self.set_sql_dialect(selected_sql_dialect);
        }
    }

    /// Draw the modal for preferences (general settings)
    pub fn draw_preferences_modal(&mut self, ctx: &egui::Context) {
        if matches!(self.modals.preferences_modal, PreferencesModal::Hidden) {
            return;
        }

        let mut close_modal = false;

        egui::Window::new("Preferences")
            .id(Id::new("preferences_window"))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Colors");
                            self.preferences.colors.draw_color_pickers(ui);
                            egui::widgets::global_theme_preference_buttons(ui);
                        });
                    });
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Keybinds");
                            self.preferences.key_binds.draw_keybinds(ui);
                        });
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if close_modal {
            self.modals.preferences_modal = PreferencesModal::Hidden;
        }
    }

    pub fn draw_sql_export_modal(&mut self, ctx: &egui::Context) {
        if matches!(self.modals.sql_export_modal, SqlExportModal::Hidden) {
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
            .show(ctx, |ui| match &self.modals.sql_export_modal {
                SqlExportModal::Hidden => {}
                SqlExportModal::Success { sql } => {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            draw_highlighted_code(ui, sql, "sql", 12);
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
                            draw_highlighted_code(ui, message, "txt", 8);
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

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(sql) = save_sql
            && let Some(path) = FileDialog::new()
                .set_directory(&self.settings.path)
                .set_file_name(format!("{}_schema", &self.settings.project_name))
                .add_filter("SQL", &["sql"])
                .save_file()
            && let Err(err) = fs::write(path, sql)
        {
            self.modals.sql_export_modal = SqlExportModal::Error {
                message: format!("Error exporting SQL: {err}"),
            };
        }

        if close_modal {
            self.modals.sql_export_modal = SqlExportModal::Hidden;
        }
    }

    pub fn draw_svg_export_modal(&mut self, ctx: &egui::Context) {
        let (mut layout, mut theme) = match self.modals.svg_export_modal {
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

                #[cfg(not(target_arch = "wasm32"))]
                if save_svg
                    && let Some(path) = FileDialog::new()
                        .set_directory(&self.settings.path)
                        .set_file_name(format!("{}_diagram", &self.settings.project_name))
                        .add_filter("SVG", &["svg"])
                        .save_file()
                    && let Err(err) = fs::write(path, &svg)
                {
                    eprintln!("Error exporting SVG: {err}");
                }
            });

        if close_modal {
            self.modals.svg_export_modal = SvgExportModal::Hidden;
        } else {
            self.modals.svg_export_modal = SvgExportModal::Open { layout, theme };
        }
    }
}
