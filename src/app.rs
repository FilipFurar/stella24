use egui::{vec2, Color32, Id, Ui};
use gethostname::gethostname;

use crate::model::{
    item::ItemType,
    table::Table,
    domain::Domain,
    connector::Connector,
};
use crate::ui::node::Node;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

pub struct AppStella {
    items: Vec<ItemType>,
}

impl Node for Connector {
    fn title(&self) -> &str { "Connector" }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.label(format!("Connects Table {} → Table {}", self.connections.0, self.connections.1));
    }

    fn can_delete(&self) -> bool {
        false
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

    fn draw_workbench_menu(&mut self, ctx: &egui::Context) {
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
    }
    fn draw_workbench(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");

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

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
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


        self.draw_workbench_menu(ctx);

        self.draw_workbench(ctx);
    }


}