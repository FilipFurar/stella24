// ui/entities/table.rs

use eframe::epaint::Color32;
use egui::{RichText, Stroke, Ui};
use slotmap::{Key, SlotMap};
use crate::app::{DomainId, TableId};
use crate::model::constraints::constraint::PrimaryKey;
use crate::model::datatype::{DataType, DATA_TYPES};
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

    fn draw_pks(&mut self, ui: &mut Ui) {
        if self.pk.attributes.len() > 0 {
            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, RED))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(egui::RichText::new("🔑").color(RED)));

                        ui.add(egui::TextEdit::singleline(&mut self.pk.name).desired_width(75.0));

                    });
                    for att in &self.pk.attributes {
                        if let Some(a) = self.attributes.get(*att) {
                            ui.label(RichText::new(&a.name).color(RED).strong());
                        }
                    }
                });
        }
    }

    fn draw_fks(&mut self, ui: &mut Ui, tables: &SlotMap<TableId, Table>) {
        if self.fks.len() > 0 {
            for (i, fk) in &mut self.fks {
                egui::Frame::group(ui.style())
                    .stroke(Stroke::new(1.0, BLUE))
                    .show(ui, |ui|{
                        //fk.display(ui, tables);
                    });
            }
        }
    }

    fn draw_attributes(&mut self, ui: &mut Ui,
                       current_id: TableId,
                       domains: &SlotMap<DomainId, Domain>,
                       tables: &SlotMap<TableId, Table>) {

        let mut to_delete: Option<AttrId> = None;
        let mut pk_changes: Vec<(AttrId, bool)> = vec![];

        for (id, attr) in self.attributes_mut() {
            let changes = attr.draw_attribute(ui, id, domains, tables, current_id);

            if let Some((id, added)) = changes.pk_change {
                pk_changes.push((id, added));
            }
            if let Some(id) = changes.delete {
                to_delete = Some(id);
            }
        }

        for (id, added) in pk_changes {
            if added {
                self.pk.attributes.insert(id);
            } else {
                self.pk.attributes.remove(&id);
            }
        }

        if let Some(id) = to_delete {
            self.remove_field(id);
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut Ui,
        current_id: TableId,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
    ) {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.separator();

        self.draw_attributes(ui, current_id, domains, tables);

        self.draw_pks(ui);
        self.draw_fks(ui, tables);

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                self.new_field();
            }

            if ui.button("Add FK").clicked() {
                //self.new_fk();
            }
        });

        ui.separator();
    }
}