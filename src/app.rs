use egui::{Color32, Id, vec2};
use gethostname::gethostname;
use slotmap::{Key, SlotMap};
use std::fs;

//use egui_phosphor_icons::{add_fonts, icons, Icon};

use crate::model::{/*connector::Connector,*/ domain::Domain, table::Table};

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

slotmap::new_key_type! {
/// Unique type for TableIDs (keys)
pub struct TableId; }
slotmap::new_key_type! {
/// Unique type for Domain IDs (keys)
pub struct DomainId; }
slotmap::new_key_type! {
    /// Unique type for FieldId keys
    pub struct FieldId;
}

/// Main application struct
/// Stores tables and domains (for now)
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct AppStella {
    tables: SlotMap<TableId, Table>,
    domains: SlotMap<DomainId, Domain>,
}

impl AppStella {
    pub fn tables(&self) -> &SlotMap<TableId, Table> {
        &self.tables
    }
    pub fn domains(&self) -> &SlotMap<DomainId, Domain> {
        &self.domains
    }
}

impl AppStella {
    /// If we have a state saved in storage, load it, call default constructor otherwise
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app
        } else {
            Default::default()
        }
    }

    /// Save file to disk
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            if let Err(err) = fs::write(&path, json) {
                eprintln!("Error saving file: {}", err);
            }
        }
    }

    /// Open file on disk
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<AppStella>(&json) {
                self.tables = state.tables;
                self.domains = state.domains;
            }
        }
    }

    /// Creates a new canvas
    pub fn handle_new(&mut self) {
        /*egui::Window::new("Save the current file?")
        .id(Id::from("new_confirm_save"))
        .resizable(true)
        .collapsible(false)
        .default_size(vec2(300.0, 200.0))
        .show(ctx, |ui| {
            if ui.button("Save").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                self.handle_save(path);
                self.items.clear();
            }
        }
            if ui.button("Don't save").clicked() {*/

        self.tables.clear();
        self.domains.clear();
        /*}
        });*/
    }

    pub fn export_html(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("HTML", &["html"])
            .save_file()
        {
            self.to_html(path.to_str().unwrap());
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
                    let table = Table::default();

                    self.tables.insert(table);
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
                    self.domains.insert(domain);
                }
                if ui
                    .add(
                        egui::Button::new("Connector")
                            .min_size(vec2(120.0, 25.0))
                            .stroke(egui::Stroke::new(1.0, PINK)),
                    )
                    .clicked()
                {}
            });
            ui.add_space(2.0);
        });
    }

    fn draw_workbench(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");

            let mut table_to_delete: Option<TableId> = None;
            let mut domain_to_delete: Option<DomainId> = None;

            for (id, table) in self.tables.iter_mut() {
                let window_id = Id::new(id);
                let title = table.title().to_owned();

                egui::Window::new(title)
                    .id(window_id)
                    .resizable(true)
                    .collapsible(false)
                    .default_size(vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        table.draw(ui, id, &self.domains);

                        if table.can_delete() {
                            ui.separator();
                            if ui.button("Delete").clicked() {
                                table_to_delete = Some(id);
                            }
                        }
                    });
            }

            egui::SidePanel::right("domains")
                .resizable(true)
                .default_width(260.0)
                .show(ctx, |ui| {
                    ui.heading("Domains");

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (id, domain) in self.domains.iter_mut() {
                            ui.group(|ui| {
                                domain.draw(ui, id.data());
                                if ui.button("🗑").clicked() {
                                    domain_to_delete = Some(id);
                                }
                            });
                        }
                    });
                });

            if let Some(idx) = table_to_delete {
                self.tables.remove(idx);
            }

            if let Some(idx) = domain_to_delete {
                self.domains.remove(idx);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl eframe::App for AppStella {
    /// Runs every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.handle_new();
                    }
                    if !is_web && ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.handle_open(path);
                        }
                    }
                    if !is_web && ui.button("Save").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.handle_save(path);
                        }
                    }
                    if ui.button("Export HTML").clicked() {
                        self.export_html();
                    }
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

        self.draw_workbench(ctx);
    }
}
