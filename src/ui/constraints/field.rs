use eframe::epaint::Color32;
use egui::RichText;
use crate::app::{DomainId, TableId};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use crate::model::field::{AttrId, AttributeType};
use egui::Ui;
use slotmap::{Key, SlotMap};

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);


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

impl AttributeType {
    pub fn draw(&mut self, ui: &mut Ui, id: AttrId, domains: &SlotMap<DomainId, Domain>) {
        let selected_text = match self {
            AttributeType::Data(dt) => DATA_TYPES[dt.base].name.to_string(),
            AttributeType::Domain(i) => domains
                .get(*i)
                .map(|d| d.name.clone())
                .unwrap_or("Invalid domain".into()),
        };

        egui::ComboBox::from_id_salt(format!("type_{}", id.data().as_ffi()))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (i, def) in DATA_TYPES.iter().enumerate() {
                    if ui.selectable_label(false, def.name).clicked() {
                        *self = AttributeType::Data(DataType {
                            base: i,
                            params: vec![0; def.param_count],
                        });
                    }
                }

                if !domains.is_empty() {
                    ui.separator();
                    for (i, domain) in domains.iter() {
                        if ui.selectable_label(false, &domain.name).clicked() {
                            *self = AttributeType::Domain(i);
                        }
                    }
                }
            });

        if let AttributeType::Data(dt) = self {
            dt.draw_params(ui);
        }
    }
}