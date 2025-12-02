use std::fmt::{Display, Formatter};
use eframe::epaint::Color32;
use egui::{vec2, Id, Ui};
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

struct DataTypeDef {
    name: &'static str,
    param_count: usize,
}

struct FieldType {
    base: usize,
    params: Vec<u32>,
}

static DATA_TYPES: &[DataTypeDef] = &[
    DataTypeDef { name: "CHAR", param_count: 0 },
    DataTypeDef { name: "VARCHAR", param_count: 1 },
    DataTypeDef { name: "BOOL", param_count: 0 },
    DataTypeDef { name: "NUMBER", param_count: 2 },
    DataTypeDef { name: "DATE", param_count: 0 },
];


struct Table {
    title: String,
    fields: Vec<Field>,
    //connectors: Vec<usize>,
}

struct Field {
    name: String,
    field_type: FieldType,
    //params: Vec<String>,
    nullable: bool,
    primary_key: bool,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: "name".to_string(),
            field_type: FieldType {
                base: 1, params: vec![5]
            },
            nullable: true,
            primary_key: false,
        }
    }
}


struct Domain {
    title: String,
    field_type: FieldType,
}

struct Connector {
    connections: (usize, usize),
}

impl Default for Connector {
    fn default() -> Self {
        Self {
            connections: (0, 0),
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            fields: vec![],
            //connectors: vec![],
        }
    }
}

trait Node {
    fn title(&self) -> &str;
    fn draw(&mut self, ui: &mut Ui, id: usize);
    fn can_delete(&self) -> bool {
        true
    }
}

impl Node for Table {
    fn title(&self) -> &str { &self.title }

    fn draw(&mut self, ui: &mut Ui, id: usize) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();
        let mut to_delete: Option<usize> = None;
        let mut need_sorting: bool = false;

        for (id, field) in self.fields.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut field.name).desired_width(75.0));
                egui::ComboBox::from_id_salt(format!("table_dt_{id}"))
                    .selected_text(DATA_TYPES[field.field_type.base].name)
                    .show_ui(ui, |ui| {
                        for (i, dt) in DATA_TYPES.iter().enumerate() {
                            if ui
                                .selectable_label(field.field_type.base == i, dt.name)
                                .clicked()
                            {
                                field.field_type.base = i;
                                field.field_type.params = vec![0; dt.param_count];
                            }
                        }
                    });


                //ui.add(egui::TextEdit::singleline(&mut field.data_type.data_type).desired_width(75.0));
                for param in &mut field.field_type.params {
                    ui.add(
                        egui::DragValue::new(param)
                            .speed(1)
                    );
                }

                if ui.checkbox(&mut field.primary_key, "PK").changed() {
                    if field.primary_key {
                        field.nullable = false;
                    }

                    need_sorting = true;
                }

                ui.add_enabled_ui(!field.primary_key, |ui| {
                    ui.checkbox(&mut field.nullable, "NULL");
                });
                if ui.button("🗑️").clicked() {
                    to_delete = Some(id);
                }
            });
        }
        self.fields
            .sort_by_key(|f| !f.primary_key);


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

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();

        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("domain_dt")
                .selected_text(DATA_TYPES[self.field_type.base].name)
                .show_ui(ui, |ui| {
                    for (i, dt) in DATA_TYPES.iter().enumerate() {
                        if ui
                            .selectable_label(self.field_type.base == i, dt.name)
                            .clicked()
                        {
                            self.field_type.base = i;
                            self.field_type.params = vec![0; dt.param_count];
                        }
                    }
                });

            /*for param in &mut self.data_type {
                ui.add(egui::TextEdit::singleline(param).desired_width(35.0));
            }*/
        });
    }
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

impl Default for Domain {
    fn default() -> Self {
        Self {
            title: "Domain".to_string(),
            field_type: FieldType {
                base: 1,           // VARCHAR
                params: vec![5],
            },
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