use eframe::epaint::Color32;
use crate::app::{DomainId, FieldId, TableId};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::domain::Domain;
use crate::model::table::Table;
use egui::Ui;
use slotmap::{Key, SlotMap};
//use egui_cable::port::Port;

const RED: Color32 = Color32::from_rgb(200, 10, 70);

/// UI implementation for tables
impl Table {
    /// Return the title as string slice
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns true if table can be deleted
    pub fn can_delete(&self) -> bool {
        true
    }

    /// Draw the Table contents in Workbench
    pub fn draw(&mut self, ui: &mut Ui, _id: TableId, domain: &SlotMap<DomainId, Domain>) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();
        let mut to_delete: Option<FieldId> = None;
        let mut to_pk: Option<FieldId> = None;
        let mut to_fields: Option<FieldId> = None;

        if self.pk().len() > 0 {
            egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(1.0, RED))
                .show(ui, |ui| {
                    for (id, pk) in self.pk_mut() {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut pk.name).desired_width(75.0));

                            pk.field_type_mut().draw(ui, id, domain);

                            let mut pk: bool = true;
                            if ui.checkbox(&mut pk, "PK").changed() {
                                if !pk {
                                    to_fields = Some(id);
                                }
                            }

                            if ui.button("🗑").clicked() {
                                to_delete = Some(id);
                            }
                        });
                    }
                });
        }

        if self.fields().len() > 0 {
            egui::Frame::group(ui.style())
                .show(ui, |ui| {
                    for (id, field) in self.fields_mut() {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut field.name).desired_width(75.0));

                            field.field_type_mut().draw(ui, id, domain);

                            let mut pk: bool = false;
                            if ui.checkbox(&mut pk, "PK").changed() {
                                if pk {
                                    to_pk = Some(id);
                                }
                            }

                            ui.checkbox(&mut field.nullable, "NULL");

                            if ui.button("🗑").clicked() {
                                to_delete = Some(id);
                            }
                        });
                    }
                });
        }


        if let Some(id) = to_delete {
            self.remove_field(id);
        }

        if let Some(id) = to_fields {
            self.remove_from_pk(id);
        } else if let Some(id) = to_pk {
            self.add_to_pk(id);
        }


        if ui.button("Add").clicked() {
            self.new_field()
        }

        ui.separator();
        /*ui.horizontal(|ui| {
            ui.add(Port::new(format!("port{}-0", id)));
            ui.add(Port::new(format!("port{}-1", id)));
        });*/
    }
}

/// UI implementation for domains
impl Domain {
    /// Draw domain's contents in Workbench
    pub fn draw(&mut self, ui: &mut Ui, id: DomainId) {
        ui.text_edit_singleline(&mut self.name);
        ui.horizontal(|ui| {
            ui.label("Type:");
            let selected_text = DATA_TYPES[self.data_type.base].name.to_string();
            egui::ComboBox::from_id_salt(format!("type_{}", id.data().as_ffi()))
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
            self.data_type.draw_params(ui);
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
