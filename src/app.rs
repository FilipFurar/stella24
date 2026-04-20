// app.rs

use egui::{Color32, Id, vec2};
use gethostname::gethostname;
use slotmap::SlotMap;
use std::fs;
//use crate::app::command::{Command, CommandHistory};
//use egui_phosphor_icons::{add_fonts, icons, Icon};
use crate::model::attribute::Attribute;
use crate::model::{entities::domain::Domain, entities::table::Table};
use crate::ui::context::TableUiContext;
use crate::ui::widgets::crow_foot::{build_edges, draw_crow_foot_edge};
use std::collections::HashMap;

mod command;
use command::{Command, CommandQueue};

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);

slotmap::new_key_type! {
    /// Unique type for TableIDs (keys)
    pub struct TableId;
}

slotmap::new_key_type! {
/// Unique type for Domain IDs (keys)
    pub struct DomainId;
}

/// Main application struct
/// Stores tables and domains (for now)
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct AppStella {
    tables: SlotMap<TableId, Table>,
    domains: SlotMap<DomainId, Domain>,
    #[serde(skip)]
    command_queue: CommandQueue,
    /*#[serde(skip)]
    command_queue: Vec<Command>,

    #[serde(skip)]
    history: CommandHistory,*/
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

    /// Queue a state change to be applied after UI collection.
    pub fn dispatch(&mut self, cmd: Command) {
        self.command_queue.push(cmd);
    }

    /// Apply all queued commands in FIFO order.
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
            Command::NewCanvas => {
                self.tables.clear();
                self.domains.clear();
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
                    a.unique = if a.pk { true } else { value };
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
                            a.unique = true;
                        }
                    }
                }
            }
            Command::CreateDomain { name, data_type } => {
                self.domains.insert(Domain { name, data_type, check_constraints: vec![], not_null: false });
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
            _ => {}
        }
    }

    /// Save file to disk
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = serde_json::to_string(&self)
            && let Err(err) = fs::write(&path, json)
        {
            eprintln!("Error saving file: {}", err);
        }
    }

    /// Open file on disk
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path)
            && let Ok(state) = serde_json::from_str::<AppStella>(&json)
        {
            self.tables = state.tables;
            self.domains = state.domains;
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

        self.dispatch(Command::NewCanvas);
        self.flush_commands();
        /*}
        });*/
    }

    pub fn export_svg(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SVG", &["svg"])
            .save_file()
        {
            self.to_svg(path.to_str().unwrap());
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
                    .collapsible(false)
                    .default_size(vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        let ui_ctx = TableUiContext::from_app(&self.tables, &self.domains, id);
                        let table = self.tables.get_mut(id).expect("table missing");
                        let changes = table.draw(ui, &ui_ctx);
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

                            // Apply PK toggles first so NN/U commands in the same frame
                            // observe the final PK state for this attribute.
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
                egui::Order::Foreground,
                Id::new("crow_foot_relations"),
            ));
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
    /// Runs every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.handle_new();
                    }
                    if !is_web
                        && ui.button("Open").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_file()
                    {
                        self.handle_open(path);
                    }
                    if !is_web
                        && ui.button("Save").clicked()
                        && let Some(path) = rfd::FileDialog::new().save_file()
                    {
                        self.handle_save(path);
                    }
                    if !is_web && ui.button("Export SVG").clicked() {
                        self.export_svg();
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
        self.draw_domains_panel(ctx);

        self.draw_workbench(ctx);
        self.flush_commands();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_former_pk_attribute_nullable_when_pk_removed_first() {
        let mut app = AppStella::default();
        let table_id = app.tables.insert(Table::default());

        let attr_id = {
            let table = app.tables.get_mut(table_id).expect("table missing");
            let attr_id = table.attributes.insert(Attribute {
                pk: true,
                not_null: true,
                ..Attribute::default()
            });
            table.pk.attributes.insert(attr_id);
            attr_id
        };

        app.dispatch(Command::SetAttributePrimaryKey {
            table: table_id,
            attr: attr_id,
            value: false,
        });
        app.dispatch(Command::SetAttributeNotNull {
            table: table_id,
            attr: attr_id,
            value: false,
        });
        app.flush_commands();

        let attr = app
            .tables
            .get(table_id)
            .and_then(|t| t.attributes.get(attr_id))
            .expect("attribute missing");
        assert!(!attr.pk);
        assert!(!attr.not_null);
    }
}

