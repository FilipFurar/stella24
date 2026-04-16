use crate::model::attribute::{AttrId, Attribute};
use crate::model::constraints::constraint::Unique;
use eframe::epaint::Color32;
use egui::{Id, Modal, Ui};
use slotmap::SlotMap;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);

impl Unique {
    pub fn draw(&mut self, ui: &mut Ui, attributes: &SlotMap<AttrId, Attribute>) {
        let mut to_delete: Vec<AttrId> = Vec::new();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("U").color(GREEN));
            ui.text_edit_singleline(&mut self.name);
        });
        for attr_id in &self.attributes {
            if let Some(attr) = attributes.get(*attr_id) {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&attr.name).color(GREEN));
                    if ui.button("🗑").clicked() {
                        to_delete.push(*attr_id);
                    }
                });
            }
        }

        for delete in to_delete {
            self.attributes.remove(&delete);
        }
    }

    pub fn attribute_modal(&mut self, ui: &mut Ui, _attributes: &SlotMap<AttrId, Attribute>) {
        Modal::new(Id::new("unique_modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("Edit Unique constraint");

            let mut should_close = false;
            let mut save_to_fks = false;

            /*for (i, attribute) in attributes {
                if ui.selectable_value()
            }*/

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_close = true;
                    save_to_fks = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });
    }
}
