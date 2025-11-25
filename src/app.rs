use std::collections::HashMap;
use std::fmt::Debug;
use eframe::epaint::Color32;
use egui::{vec2, Id, Ui, Vec2};
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

struct Type {
    data_type: String,
    params: Option<u32>,
}

impl Type {
    fn get_type_string(&self) -> String {
        let mut string = self.data_type.clone();
        match self.params {
            None => {
                string
            }
            Some(n) => {
                let param = format!("({})", n);
                string.push_str(&*param);
                string
            }
        }
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
    //fn
}

trait Node {
    fn title(&self) -> &str;
    fn title_mut(&mut self) -> &mut String;
    fn draw(&mut self, ui: &mut Ui, id: usize);
}

impl Node for Table {
    fn title(&self) -> &str { &self.title }
    fn title_mut(&mut self) -> &mut String { &mut self.title }

    fn draw(&mut self, ui: &mut Ui, id: usize) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });

        for field in &self.fields {
            ui.horizontal(|ui| {
                ui.label(&field.name);
                ui.label(&field.data_type.get_type_string());
            });
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

        ui.label(self.defined_as.get_type_string());
    }
}

impl Node for Connector {
    fn title(&self) -> &str { "Connector" }
    fn title_mut(&mut self) -> &mut String { panic!("Connectors have no title") }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.label(format!("Connects Table {} → Table {}", self.connections.0, self.connections.1));
    }
}

impl Default for Domain {
    fn default() -> Self {
        let datatype = Type {
            data_type: "varchar".to_string(),
            params: Some(5),
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
                    /*if self.tables.len() > 1 {
                        let connector = Connector {
                            connections: (0, 1),
                        };
                        self.connectors.push(connector);
                    }*/
                }
            });
            ui.add_space(2.0);

        });
        /*egui::SidePanel::left("properties").default_width(400.0).resizable(true).min_width(300.0f32).show(ctx, |ui| {
            ui.heading("Properties");
            match &self.current_item {
                Some(x) => match x {
                    ItemType::Connector(idx) => {
                        let connector = &mut self.connectors[*idx];
                        let (mut i1, mut i2) = connector.connections;

                        ui.horizontal(|ui| {
                            egui::ComboBox::from_label("From")
                                .selected_text(self.tables.get(i1).map(|t| t.title.clone()).unwrap_or_default())
                                .show_ui(ui, |ui| {
                                    for (table_index, table) in self.tables.iter().enumerate() {
                                        ui.selectable_value(&mut i1, table_index, &table.title);
                                    }
                                });

                            egui::ComboBox::from_label("To")
                                .selected_text(self.tables.get(i2).map(|t| t.title.clone()).unwrap_or_default())
                                .show_ui(ui, |ui| {
                                    for (table_index, table) in self.tables.iter().enumerate() {
                                        ui.selectable_value(&mut i2, table_index, &table.title);
                                    }
                                });
                        });

                        connector.connections = (i1, i2);
                    }
                },
                None => {}
            }
        });*/

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");

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
                    });
            }
            /*for (idx, connector) in self.connectors.iter().enumerate() {
                let (i1, i2) = connector.connections;
                let t1 = &self.tables[i1];
                let t2 = &self.tables[i2];
                let text = format!("{} - {}", t1.title, t2.title);

            }*/

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}