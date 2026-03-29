// ui/entities/domain.rs

use egui::Ui;
use slotmap::Key;
use crate::app::DomainId;
use crate::model::datatype::{DataType, DATA_TYPES};
use crate::model::entities::domain::Domain;

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