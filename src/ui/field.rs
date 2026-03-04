use crate::app::DomainId;
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::domain::Domain;
use crate::model::field::FieldType;
use egui::Ui;
use slotmap::SlotMap;

impl DataType {
    pub fn draw_params(&mut self, ui: &mut Ui) {
        if !self.params.is_empty() {
            ui.horizontal(|ui| {
                ui.label("(");

                for (_i, param) in self.params.iter_mut().enumerate() {
                    ui.add(egui::DragValue::new(param).speed(1).range(0..=1_000_000));
                }

                ui.label(")");
            });
        }
    }
}

/// UI display for different FieldTypes
impl FieldType {
    /// Draw Field (Built-in or a domain type) and the parameters settings
    pub fn draw(&mut self, ui: &mut Ui, id: usize, domains: &SlotMap<DomainId, Domain>) {
        let selected_text = match self {
            FieldType::Data(dt) => DATA_TYPES[dt.base].name.to_string(),
            FieldType::Domain(i) => domains
                .get(*i)
                .map(|d| d.name.clone())
                .unwrap_or("Invalid domain".into()),
        };

        egui::ComboBox::from_id_salt(format!("type_{id}"))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (i, def) in DATA_TYPES.iter().enumerate() {
                    if ui.selectable_label(false, def.name).clicked() {
                        *self = FieldType::Data(DataType {
                            base: i,
                            params: vec![0; def.param_count],
                        });
                    }
                }

                if !domains.is_empty() {
                    ui.separator();
                    for (i, domain) in domains.iter() {
                        if ui.selectable_label(false, &domain.name).clicked() {
                            *self = FieldType::Domain(i);
                        }
                    }
                }
            });

        if let FieldType::Data(dt) = self {
            dt.draw_params(ui);
        }
    }
}
