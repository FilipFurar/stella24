use eframe::epaint::Color32;
use egui::{RichText, Stroke, Ui};
use slotmap::SlotMap;
use crate::app::{DomainId, TableId};
use crate::model::constraints::constraint::PrimaryKey;
use crate::model::field::{AttrId, AttributeType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);

impl Table {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn can_delete(&self) -> bool {
        true
    }

    pub fn draw(
        &mut self,
        ui: &mut Ui,
        _id: TableId,
        domain: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
    ) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();

        let mut to_delete: Option<AttrId> = None;
        let mut pk_changes: Vec<(AttrId, bool)> = vec![];

        for (id, attr) in self.attributes_mut() {
            let stroke = Color32::DARK_GRAY;

            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, stroke))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut attr.name).desired_width(75.0));

                        attr.attribute_type_mut().draw(ui, id, domain);

                        let mut is_pk = attr.pk;
                        if ui.checkbox(&mut is_pk, "PK").changed() {
                            attr.pk = is_pk;
                            pk_changes.push((id, is_pk));
                        }

                        if ui.button("🗑").clicked() {
                            to_delete = Some(id);
                        }
                    });
                });
        }

        for (id, added) in pk_changes {
            if added {
                self.pk.attributes.insert(id);
            } else {
                self.pk.attributes.remove(&id);
            }
        }

        if self.pk.attributes.len() > 0 {
            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, RED))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.pk.name).desired_width(75.0));

                        ui.horizontal(|ui| {
                            for att in &self.pk.attributes {
                                if let Some(a) = self.attributes.get(*att) {
                                    ui.label(&a.name);
                                }
                            }
                        });
                    });
                });
        }

        if let Some(id) = to_delete {
            self.remove_field(id);
        }

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                self.new_field();
            }
            if ui.button("Add PK").clicked() {
                self.change_pk(PrimaryKey::new());
            }
            if ui.button("Add FK").clicked() {
                self.new_fk();
            }
        });

        ui.separator();
    }
}