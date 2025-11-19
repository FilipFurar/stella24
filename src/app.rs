use std::collections::HashMap;
use std::fmt::Debug;
use eframe::epaint::Color32;
use egui::{vec2, Vec2};
use egui_cable::prelude::*;
use gethostname::gethostname;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

pub struct AppStella<'a> {
    tables: Vec<Table<'a>>,
    domains: Vec<Domain>,
    connectors: Vec<Connector>,
}
enum ItemType {
    Table(usize),
    Domain(usize),
    Connector(usize),
}

struct Table<'a> {
    title: String,
    fields: Vec<(String, String)>,
    connections: Vec<&'a Connector>,
}

struct Domain {
    title: String,
    defined_as: String,
}

struct Connector {
    connections: (usize, usize),
}


impl<'a> Default for AppStella<'a> {
    fn default() -> Self {
        Self {
            tables: vec![],
            domains: vec![],
            connectors: vec![],
        }
    }
}

impl<'a> AppStella<'a> {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl<'a> eframe::App for AppStella<'a> {
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
                    let table = Table {
                        title: "Title".to_string(),
                        fields: vec![],
                        connections: vec![],
                    };

                    self.tables.push(table);
                    let idx = self.tables.len() - 1;


                }
                if ui
                    .add(egui::Button::new("Domain").min_size(vec2(120.0, 25.0)).stroke(egui::Stroke::new(1.0, GREEN)))
                    .clicked()
                {
                    let domain = Domain {
                        title: "Title".to_string(),
                        defined_as: "char(20)".to_string(),
                    };
                    self.domains.push(domain);
                    let idx = self.domains.len() - 1;
                }
                if ui
                    .add(egui::Button::new("Connector").min_size(vec2(120.0, 25.0)).stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {
                    if self.tables.len() > 1 {
                        let connector = Connector {
                            connections: (0, 1),
                        };
                        self.connectors.push(connector);
                        let idx = self.connectors.len() - 1;
                    }
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
            for (idx, table) in self.tables.iter_mut().enumerate() {
                let my_window_id_str : String = format!("table{}", idx);
                let my_window_id = egui::Id::new(&my_window_id_str);
                let title = &table.title;
                egui::Window::new(title)
                    .id(my_window_id)
                    .resizable(true)
                    .default_size(egui::vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Title:");
                            ui.text_edit_singleline(&mut table.title);
                        });

                        for (field_name, field_type) in &table.fields {
                            ui.horizontal(|ui| {
                                ui.label(field_name);
                                ui.label(field_type);
                            });
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add(Port::new(format!("port{}-0", my_window_id_str)));
                            ui.add(Port::new(format!("port{}-1", my_window_id_str)));
                        });
                    });


            }
            for (idx, domain) in self.domains.iter_mut().enumerate() {
                let window_id:egui::Id = format!("domain{}", idx).into();
                let title = &domain.title;
                egui::Window::new(title)
                    .id(window_id)
                    .resizable(true)
                    .default_size(egui::vec2(300.0, 200.0))
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Title:");
                            ui.text_edit_singleline(&mut domain.title);
                        });

                        ui.label(&domain.defined_as);
                    });

            }
            for (idx, connector) in self.connectors.iter().enumerate() {
                let (i1, i2) = connector.connections;
                let t1 = &self.tables[i1];
                let t2 = &self.tables[i2];
                let text = format!("{} - {}", t1.title, t2.title);

            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
