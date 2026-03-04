use crate::app::{DomainId, TableId};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::domain::Domain;
use crate::model::table::Table;
use egui::Ui;
use slotmap::{KeyData, SlotMap};
//use egui_cable::port::Port;

/// UI implementation for tables
impl Table {
    /// Return the title as string slice
    pub(crate) fn title(&self) -> &str {
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
        let mut to_delete: Option<usize> = None;
        let mut need_sorting: bool = false;

        for (id, field) in self.fields_mut().iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut field.name).desired_width(75.0));

                field.field_type_mut().draw(ui, id, domain);

                if ui.checkbox(&mut field.primary_key(), "PK").changed() {
                    if field.primary_key() {
                        field.nullable = false;
                    }
                    need_sorting = true;
                }

                ui.add_enabled_ui(!field.primary_key, |ui| {
                    ui.checkbox(&mut field.nullable, "NULL");
                });

                if ui.button("🗑").clicked() {
                    to_delete = Some(id);
                }
            });
        }

        self.sort_by_key();

        if let Some(id) = to_delete {
            self.remove_field(id);
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
    pub fn draw(&mut self, ui: &mut Ui, id: KeyData) {
        ui.text_edit_singleline(&mut self.name);
        ui.horizontal(|ui| {
            ui.label("Type:");
            let selected_text = DATA_TYPES[self.data_type.base].name.to_string();
            egui::ComboBox::from_id_salt(format!("type_{}", id.as_ffi()))
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
