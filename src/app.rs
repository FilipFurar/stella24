use eframe::epaint::Color32;
use gethostname::gethostname;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

enum ItemKind {
    Table,
    Domain,
    Connector,
}

pub struct AppStella {
    items: Vec<ItemKind>,
}

impl Default for AppStella {
    fn default() -> Self {
        Self {
            items: Vec::new(),
        }
    }
}

impl AppStella {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
                    .add(egui::Button::new("Table").stroke(egui::Stroke::new(1.0, BLUE)))
                    .clicked()
                {
                    self.items.push(ItemKind::Table);
                }
                if ui
                    .add(egui::Button::new("Domain").stroke(egui::Stroke::new(1.0, GREEN)))
                    .clicked()
                {
                    self.items.push(ItemKind::Domain);
                }
                if ui
                    .add(egui::Button::new("Connector").stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {
                    self.items.push(ItemKind::Connector);

                }
            });
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");
            for item in &self.items {
                ui.label("kkt");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
