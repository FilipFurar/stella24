use crate::model::attribute::{AttrId, Attribute};
use crate::model::constraints::constraint::Unique;
use eframe::epaint::Color32;
use egui::{Id, Modal, Ui};
use slotmap::SlotMap;
use std::collections::HashSet;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);

impl Unique {
    pub fn draw(&mut self, ui: &mut Ui, attributes: &SlotMap<AttrId, Attribute>) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("U").color(GREEN));
            ui.text_edit_singleline(&mut self.name);
        });
        for attr_id in &self.attributes {
            if let Some(attr) = attributes.get(*attr_id) {
                ui.label(egui::RichText::new(&attr.name).color(GREEN));
            }
        }
    }

    pub fn attribute_modal(
        &mut self,
        ui: &mut Ui,
        attributes: &SlotMap<AttrId, Attribute>,
        unique_index: usize,
    ) -> bool {
        let modal_id = Id::new(("unique_modal", unique_index));
        let draft_id = Id::new(("unique_modal_draft", unique_index));
        let mut draft = ui
            .data_mut(|d| d.get_temp::<HashSet<AttrId>>(draft_id))
            .unwrap_or_else(|| self.attributes.clone());

        let mut should_close = false;
        let mut save_changes = false;

        Modal::new(modal_id).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("Edit Unique constraint");

            ui.separator();
            for (attr_id, attribute) in attributes {
                let mut selected = draft.contains(&attr_id);
                if ui.checkbox(&mut selected, &attribute.name).changed() {
                    if selected {
                        draft.insert(attr_id);
                    } else {
                        draft.remove(&attr_id);
                    }
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_close = true;
                    save_changes = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

        if should_close {
            if save_changes {
                self.attributes = draft;
            }
            ui.data_mut(|d| d.remove::<HashSet<AttrId>>(draft_id));
            true
        } else {
            ui.data_mut(|d| d.insert_temp(draft_id, draft));
            false
        }
    }
}
