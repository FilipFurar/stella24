use std::collections::HashMap;
use std::fmt::Debug;
use eframe::epaint::Color32;
use egui::{vec2, Align, Id, Ui, Vec2, Widget};
use egui_cable::prelude::*;
use gethostname::gethostname;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

pub struct AppStella {
    items: Vec<ItemType>,
}
enum ItemType {
    Table(Table),
    Domain(Domain),
    Connector(Connector),
}

impl ItemType {
    fn node(&self) -> &dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            ItemType::Connector(c) => c,
        }
    }

    fn node_mut(&mut self) -> &mut dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            ItemType::Connector(c) => c,
        }
    }
}

struct Table {
    title: String,
    fields: Vec<Field>,
    connectors: Vec<usize>,
}

struct Field {
    name: String,
    data_type: Type,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: "name".to_string(),
            data_type: Type {
                data_type: "char".to_string(),
                params: "".to_string(),
            },
        }
    }
}

struct Type {
    data_type: String,
    params: String,
}

impl Type {
    fn get_type_string(&self) -> String {
        let mut string = self.data_type.clone();
        let param = format!("({})", self.params);
        string.push_str(&*param);
        string
    }
}

struct Domain {
    title: String,
    defined_as: Type,
}

struct Connector {
    connections: (usize, usize),
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            fields: vec![],
            connectors: vec![],
        }
    }
}

impl Table {
}

trait Node {
    fn title(&self) -> &str;
    fn title_mut(&mut self) -> &mut String;
    fn draw(&mut self, ui: &mut Ui, id: usize);

    fn can_delete(&self) -> bool {
        true
    }
}

impl Node for Table {
    fn title(&self) -> &str { &self.title }
    fn title_mut(&mut self) -> &mut String { &mut self.title }

    fn draw(&mut self, ui: &mut Ui, id: usize) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();
        let mut to_delete: Option<usize> = None;

        for (id, field) in self.fields.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut field.name).min_size(Vec2::new(100f32, 20f32)));
                ui.add(egui::TextEdit::singleline(&mut field.data_type.data_type).min_size(Vec2::new(100f32, 20f32)));
                ui.add(egui::TextEdit::singleline(&mut field.data_type.params).min_size(Vec2::new(100f32, 20f32)));
                if ui.button("🗑️").clicked() {
                    to_delete = Some(id);
                }
            });
        }

        if let Some(id) = to_delete {
            self.fields.remove(id);
        }

        if ui.button("Add").clicked() {
            self.fields.push(Field::default());
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.add(Port::new(format!("port{}-0", id)));
            ui.add(Port::new(format!("port{}-1", id)));
        });
    }
}

impl Node for Domain {
    fn title(&self) -> &str { &self.title }
    fn title_mut(&mut self) -> &mut String { &mut self.title }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();


        ui.label(self.defined_as.get_type_string());
    }
}

impl Node for Connector {
    fn title(&self) -> &str { "Connector" }
    fn title_mut(&mut self) -> &mut String { panic!("Connectors have no title") }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.label(format!("Connects Table {} → Table {}", self.connections.0, self.connections.1));
    }

    fn can_delete(&self) -> bool {
        false
    }
}

impl Default for Domain {
    fn default() -> Self {
        let datatype = Type {
            data_type: "varchar".to_string(),
            params: "5".to_string(),
        };
        Self {
            title: "Domain".to_string(),
            defined_as: datatype,
        }
    }
}

impl Default for AppStella {
    fn default() -> Self {
        Self {
            items: vec![],
        }
    }
}

impl AppStella {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for AppStella {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
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


        egui::TopBottomPanel::top("workbenchmenu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::menu::bar(ui, |ui| {
                if ui
                    .add(egui::Button::new("Table").min_size(vec2(120.0, 25.0)).stroke(egui::Stroke::new(1.0, BLUE)))
                    .clicked()
                {
                    let table = Table::default();

                    self.items.push(ItemType::Table(table));

                }
                if ui
                    .add(egui::Button::new("Domain").min_size(vec2(120.0, 25.0)).stroke(egui::Stroke::new(1.0, GREEN)))
                    .clicked()
                {
                    let domain = Domain::default();
                    self.items.push(ItemType::Domain(domain));
                }
                if ui
                    .add(egui::Button::new("Connector").min_size(vec2(120.0, 25.0)).stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {

                }
            });
            ui.add_space(2.0);

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut to_delete: Option<usize> = None;

            for (id, item) in self.items.iter_mut().enumerate() {
                let window_id = Id::new(id);
                let title = item.node().title().to_owned();

                egui::Window::new(title)
                    .id(window_id)
                    .resizable(true)
                    .collapsible(false)
                    .default_size(vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        item.node_mut().draw(ui, id);

                            if item.node().can_delete() {
                                ui.separator();
                                if ui.button("Delete").clicked() {
                                    to_delete = Some(id);
                                }
                            }
                    });
            }
            if let Some(idx) = to_delete {
                self.items.remove(idx);
            }
            ui.heading("Workbench");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}