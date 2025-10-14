use std::collections::HashMap;
use eframe::epaint::Color32;
use gethostname::gethostname;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

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

pub struct AppStella<'a> {
    tables: Vec<Table<'a>>,
    domains: Vec<Domain>,
    connectors: Vec<Connector>,
}

impl<'a> Default for AppStella<'a> {
    fn default() -> Self {
        Self {
            tables: vec![Table {title: "ahoj1".to_string(), fields: vec![], connections: vec![]}, Table {
                title: "ahoj2".to_string(),
                fields: vec![],
                connections: vec![],
            }],
            domains: vec![],
            connectors: vec![],
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
                        title: "kkt".to_string(),
                        fields: vec![],
                        connections: vec![],
                    };
                    self.tables.push(table);
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
                }
                if ui
                    .add(egui::Button::new("Connector").stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {
                    let connector = Connector {
                        connections: (0, 1),
                    };
                    self.connectors.push(connector);

                }
            });
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");
            for table in &self.tables {

                ui.label(&table.title);
            }
            for domain  in &self.domains {
                ui.label(&domain.title);
            }
            for connector in &self.connectors {
                let (i1, i2) = connector.connections;
                let t1 = &self.tables[i1];
                let t2 = &self.tables[i2];
                let text = t1.title.clone() + &t2.title.clone();
                ui.label(text.to_string());
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
