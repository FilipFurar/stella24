use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::domain::Domain;
use crate::model::field::Field;
use crate::model::table::Table;
use egui::Ui;
//use egui_cable::port::Port;

impl Table {
    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn can_delete(&self) -> bool {
        true
    }

    pub(crate) fn draw(&mut self, ui: &mut Ui, _id: usize, domain: &[Domain]) {
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

                field.field_type.draw(ui, id, domain);

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

impl Domain {
    pub(crate) fn draw(&mut self, ui: &mut Ui, id: usize) {

            ui.text_edit_singleline(&mut self.name);
            ui.horizontal(|ui| {
                ui.label("Type:");
                let selected_text = DATA_TYPES[self.data_type.base].name.to_string();
                egui::ComboBox::from_id_salt(format!("type_{id}"))
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (i, def) in DATA_TYPES.iter().enumerate() {
                            if ui.selectable_label(false, def.name).clicked() {
                                self.data_type = DataType {
                                    base: i,
                                    params: vec![0; def.param_count],
                                }
                            }
                        }
                    });
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
