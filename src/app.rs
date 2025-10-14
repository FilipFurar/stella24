use std::collections::HashMap;
use eframe::epaint::Color32;
use gethostname::gethostname;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

pub struct AppStella<'a> {
    tables: Vec<Table<'a>>,
    domains: Vec<Domain>,
    connectors: Vec<Connector>,
    current_item: Option<ItemType>,
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
            current_item: None,
        }
    }
}

impl<'a> AppStella<'a> {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
                    .add(egui::Button::new("Table").stroke(egui::Stroke::new(1.0, BLUE)))
                    .clicked()
                {
                    let table = Table {
                        title: "Title".to_string(),
                        fields: vec![],
                        connections: vec![],
                    };

                    self.tables.push(table);
                    let idx = self.tables.len() - 1;
                    self.current_item = Some(ItemType::Table(idx));


                }
                if ui
                    .add(egui::Button::new("Domain").stroke(egui::Stroke::new(1.0, GREEN)))
                    .clicked()
                {
                    let domain = Domain {
                        title: "kkt".to_string(),
                        defined_as: "char(20)".to_string(),
                    };
                    self.domains.push(domain);
                    let idx = self.domains.len() - 1;
                    self.current_item = Some(ItemType::Domain(idx));
                }
                if ui
                    .add(egui::Button::new("Connector").stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {
                    if self.tables.len() > 1 {
                        let connector = Connector {
                            connections: (0, 1),
                        };
                        self.connectors.push(connector);
                        let idx = self.connectors.len() - 1;
                        self.current_item = Some(ItemType::Connector(idx));
                    }
                }
            });
            //  ui.add_space(2.0);

        });
        egui::SidePanel::left("properties").resizable(true).min_width(200.0f32).max_width(600.0f32).show(ctx, |ui| {
            ui.heading("Properties");
            match &self.current_item {
                Some(x) => match x {
                    ItemType::Table(idx) => {
                            let table = &mut self.tables[*idx];
                            ui.text_edit_singleline(&mut table.title);
                    }
                    ItemType::Domain(idx) => {
                        let domain = &mut self.domains[*idx];
                        ui.text_edit_singleline(&mut domain.title);
                    }
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
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");
            for (idx, table) in self.tables.iter().enumerate() {
                let is_selected = matches!(self.current_item, Some(ItemType::Table(i)) if i == idx);
                if ui.selectable_label(is_selected, &table.title).clicked() {
                    self.current_item = Some(ItemType::Table(idx));
                }
            }
            for (idx, domain) in self.domains.iter().enumerate() {
                let is_selected = matches!(self.current_item, Some(ItemType::Domain(i)) if i == idx);
                if ui.selectable_label(is_selected, &domain.title).clicked() {
                    self.current_item = Some(ItemType::Domain(idx));
                }
            }
            for (idx, connector) in self.connectors.iter().enumerate() {
                let (i1, i2) = connector.connections;
                let t1 = &self.tables[i1];
                let t2 = &self.tables[i2];
                let text = format!("{} - {}", t1.title, t2.title);

                let is_selected = matches!(self.current_item, Some(ItemType::Connector(i)) if i == idx);
                if ui.selectable_label(is_selected, text).clicked() {
                    self.current_item = Some(ItemType::Connector(idx));
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
