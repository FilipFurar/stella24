use crate::model::datatype::DATA_TYPES;
use crate::model::domain::Domain;
use crate::model::field::Field;
use crate::model::table::Table;
use egui::Ui;
//use egui_cable::port::Port;

pub trait Node {
    fn title(&self) -> &str;
    fn draw(&mut self, ui: &mut Ui, id: usize);
    fn can_delete(&self) -> bool {
        true
    }
}

impl Node for Table {
    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
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
                    ui.add(egui::DragValue::new(param).speed(1));
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
        self.fields.sort_by_key(|f| !f.primary_key);

        if let Some(id) = to_delete {
            self.fields.remove(id);
        }

        if ui.button("Add").clicked() {
            self.fields.push(Field::default());
        }

        ui.separator();
        /*ui.horizontal(|ui| {
            ui.add(Port::new(format!("port{}-0", id)));
            ui.add(Port::new(format!("port{}-1", id)));
        });*/
    }
}

impl Node for Domain {
    fn title(&self) -> &str {
        &self.title
    }

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

/*impl Node for Connector {
    fn title(&self) -> &str {
        "Connector"
    }

    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.label(format!(
            "Connects Table {} → Table {}",
            self.connections.0, self.connections.1
        ));
    }

    fn can_delete(&self) -> bool {
        false
    }
}*/
