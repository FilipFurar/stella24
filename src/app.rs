// Import necessary modules and dependencies
use eframe::epaint::Color32; // For defining color constants
use std::collections::HashMap; // For managing workbench items by ID
use std::fs; // For reading and writing files
use egui_file::FileDialog; // For handling file dialogs

// Define color constants for UI elements
const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const PINK: Color32 = Color32::from_rgb(194, 73, 125);

// Trait defining shared behavior for all workbench items
pub trait WorkbenchItem {
    // Get the unique ID of the item
    fn get_id(&self) -> i32;

    // Get the type name of the item
    fn get_type_name(&self) -> &str;

    // Default method to display item name as "Type ID"
    fn display_name(&self) -> String {
        format!("{} {}", self.get_type_name(), self.get_id())
    }
}

// Table struct representing a table workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Table {
    pub id: i32,
    title: String, // Title of the table
}

// Implement the WorkbenchItem trait for Table
impl WorkbenchItem for Table {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_type_name(&self) -> &str {
        self.title.as_str()
    }
}

// Domain struct representing a domain workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Domain {
    pub id: i32, // Unique ID of the domain
}

// Implement the WorkbenchItem trait for Domain
impl WorkbenchItem for Domain {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_type_name(&self) -> &str {
        "Domain"
    }
}

// Connector struct representing a connector workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Connector {
    pub id: i32, // Unique ID of the connector
}

// Implement the WorkbenchItem trait for Connector
impl WorkbenchItem for Connector {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_type_name(&self) -> &str {
        "Connector"
    }
}

// Enum wrapping all possible workbench item types
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum WorkbenchItemType {
    Table(Table),
    Domain(Domain),
    Connector(Connector),
}

// Implement shared functionality for WorkbenchItemType
impl WorkbenchItemType {
    // Get display name for the item based on its type
    pub fn display_name(&self) -> String {
        match self {
            WorkbenchItemType::Table(t) => t.display_name(),
            WorkbenchItemType::Domain(d) => d.display_name(),
            WorkbenchItemType::Connector(c) => c.display_name(),
        }
    }

    // Get the unique ID of the item
    pub fn get_id(&self) -> i32 {
        match self {
            WorkbenchItemType::Table(t) => t.get_id(),
            WorkbenchItemType::Domain(d) => d.get_id(),
            WorkbenchItemType::Connector(c) => c.get_id(),
        }
    }
}

// Struct for saving/loading the application state
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SavedState {
    pub workbench_items: HashMap<i32, WorkbenchItemType>, // Mapping of IDs to workbench items
}

// Main application struct
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Provides default values for fields
pub struct AppStella {
    pub workbench_items: HashMap<i32, WorkbenchItemType>, // Current workbench items
    #[serde(skip)] // Fields skipped during serialization
    pub next_id: i32, // ID to assign to the next created item
    #[serde(skip)]
    pub save_dialog: Option<FileDialog>, // Save file dialog
    #[serde(skip)]
    pub open_dialog: Option<FileDialog>, // Open file dialog
}

// Default implementation for AppStella
impl Default for AppStella {
    fn default() -> Self {
        Self {
            workbench_items: HashMap::new(),
            next_id: 1, // Start with ID 1
            save_dialog: None,
            open_dialog: None,
        }
    }
}

// Main application logic
impl AppStella {
    // Create a new AppStella instance, restoring from storage if available
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.next_id = app.workbench_items
                .keys()
                .max()
                .map_or(1, |max_id| max_id + 1);
            app
        } else {
            Default::default()
        }
    }

    // Save the current application state to a file
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        let state = SavedState {
            workbench_items: self.workbench_items.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&state) {
            if let Err(err) = fs::write(&path, json) {
                eprintln!("Error saving file: {}", err);
            }
        }
    }

    // Load application state from a file
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<SavedState>(&json) {
                self.workbench_items = state.workbench_items;
                self.next_id = self.workbench_items
                    .keys()
                    .max()
                    .map_or(1, |max_id| max_id + 1);
            }
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
                    if ui.button("New").clicked() {
                        *self = Self::default(); // Reset the application
                    }
                    if ui.button("Open").clicked() {
                        let mut dialog = FileDialog::open_file(None);
                        dialog.open();
                        self.open_dialog = Some(dialog);
                    }
                    if ui.button("Save").clicked() {
                        let mut dialog = FileDialog::save_file(None);
                        dialog.open();
                        self.save_dialog = Some(dialog);
                    }
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close); // Close the app
                        }
                    }
                });
                
                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui); // Add UI theme preferences
            });
        });

        // Save dialog handling
        if let Some(dialog) = &mut self.save_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    let path_clone = path.to_path_buf(); // Avoid mutably borrowing `self`
                    self.handle_save(path_clone);
                }
                self.save_dialog = None;
            }
        } else if let Some(dialog) = &mut self.open_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    let path_clone = path.to_path_buf(); // Avoid mutably borrowing `self`
                    self.handle_open(path_clone);
                }
                self.open_dialog = None;
            }
        }

        // Workbench menu for adding items
        egui::TopBottomPanel::top("workbenchmenu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::menu::bar(ui, |ui| {
                if ui.add(egui::Button::new("Table").stroke(egui::Stroke::new(1.0, BLUE))).clicked() {
                    self.workbench_items.insert(
                        self.next_id,
                        WorkbenchItemType::Table(Table {
                            id: self.next_id,
                            title: "test".parse().unwrap(),
                        })
                    );
                    self.next_id += 1;
                }
                if ui.add(egui::Button::new("Domain").stroke(egui::Stroke::new(1.0, GREEN))).clicked() {
                    self.workbench_items.insert(
                        self.next_id,
                        WorkbenchItemType::Domain(Domain { id: self.next_id })
                    );
                    self.next_id += 1;
                }
                if ui.add(egui::Button::new("Connector").stroke(egui::Stroke::new(1.0, PINK))).clicked() {
                    self.workbench_items.insert(
                        self.next_id,
                        WorkbenchItemType::Connector(Connector { id: self.next_id })
                    );
                    self.next_id += 1;
                }
            });
            ui.add_space(2.0);
        });

        // Central panel for displaying workbench items
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workbench");

            // Sort and display workbench items by ID
            let mut items: Vec<(&i32, &WorkbenchItemType)> = self.workbench_items.iter().collect();
            items.sort_by_key(|&(id, _)| *id);

            for (_, item) in items {
                ui.label(item.display_name());
            }

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
