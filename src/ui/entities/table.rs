use eframe::epaint::Color32;
use egui::{RichText, Ui};
use slotmap::SlotMap;
use crate::app::{DomainId, TableId};
use crate::model::field::FieldId;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);

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

                            pk.field_type_mut().draw(ui, id, domain, tables);
                            let mut pk: bool = true;
                            if ui.checkbox(&mut pk, RichText::new("PK").color(RED).strong()).changed() {
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

        if self.fks().len() > 0 {
            egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(1.0, BLUE))
                .show(ui, |ui| {
                    for (id, fk) in &mut self.fks_mut().iter_mut() {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut fk.name).desired_width(75.0));

                            fk.field_type_mut().draw(ui, id, domain, tables);
                            let mut pk = false;
                            if ui.checkbox(&mut pk, "PK").changed() {
                                to_pk = Some(id);
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

                            field.field_type_mut().draw(ui, id, domain, tables);
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
        

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                self.new_field();
            }
            if ui.button("Add FK").clicked() {
                self.new_fk();
            }
        });

        ui.separator();
    }
}