use eframe::epaint::Color32;
use gethostname::gethostname;

// Define color constants for UI elements
const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);


// Main application struct
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Provides default values for fields
pub struct AppStella {

}

// Default implementation for AppStella
impl Default for AppStella {
    fn default() -> Self {
        Self {

        }
    }
}

// Main application logic
impl AppStella {
    // Create a new AppStella instance, restoring from storage if available
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app
        } else {
            Default::default()
        }
    }
}

// Implementation of the eframe application
impl eframe::App for AppStella {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu panel for File operations
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        // Close the app
                    }
                });

                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui); // Add UI theme preferences

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let text = "Welcome, ".to_string() + &gethostname().to_string_lossy();
                    ui.label(text);
                })
            });
        });

        // Workbench menu for adding items
        egui::TopBottomPanel::top("workbenchmenu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::menu::bar(ui, |ui| {
                if ui
                    .add(egui::Button::new("Table").stroke(egui::Stroke::new(1.0, BLUE)))
                    .clicked()
                {
                }
                if ui
                    .add(egui::Button::new("Domain").stroke(egui::Stroke::new(1.0, GREEN)))
                    .clicked()
                {
                }
                if ui
                    .add(egui::Button::new("Connector").stroke(egui::Stroke::new(1.0, PINK)))
                    .clicked()
                {
                }
            });
            ui.add_space(2.0);
        });

        // Central panel for displaying workbench items
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");


            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui); // Display debug warnings if applicable
            });
        });
    }

    // Save the current state to storage
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
