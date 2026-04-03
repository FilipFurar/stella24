// ui/entities/table.rs

use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::ForeignKey;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use eframe::epaint::Color32;
use egui::{Id, Modal, RichText, Stroke, Ui};
use slotmap::SlotMap;
use std::ops::Index;

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);

impl Table {
    fn handle_fk_modal(&mut self, ui: &mut Ui, tables: &SlotMap<TableId, Table>) {
        // Take ownership out of Option, put back if not closing
        let mut fk = match self.current_fk.take() {
            Some(fk) => fk,
            None => return,
        };

        let mut should_close = false;
        let mut save_to_fks = false;

        Modal::new(Id::new("fk_modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("New Foreign Key");

            ui.label("Name:");
            ui.text_edit_singleline(&mut fk.name);

            let selected_text = fk
                .references
                .and_then(|id| tables.get(id))
                .map(|t| t.title.clone())
                .unwrap_or_else(|| "No table selected".to_string());

            egui::ComboBox::from_id_salt("fk_table_pick")
                .selected_text(&selected_text)
                .show_ui(ui, |ui| {
                    if tables.is_empty() {
                        ui.weak("No tables available");
                    } else {
                        for (tid, table) in tables.iter() {
                            ui.selectable_value(&mut fk.references, Some(tid), &table.title);
                        }
                    }
                });

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

        if should_close {
            if save_to_fks {
                if let Some(other_table_id) = fk.references {
                    if let Some(other_table) = tables.get(other_table_id) {
                        for other_table_attr_id in &other_table.pk.attributes {
                            fk.local_attrs.insert(*other_table_attr_id);

                            let current_table_fk_id = self.fks.insert(fk.clone());

                            self.attributes.insert(Attribute {
                                name: format!(
                                    "{}_{}",
                                    self.fks.get(current_table_fk_id).expect("fk_id error").name,
                                    other_table
                                        .attributes
                                        .get(*other_table_attr_id)
                                        .expect("error attribut id")
                                        .name
                                ),
                                attribute_type: AttributeType::ForeignKeyAttribute(*other_table_attr_id),
                                pk: false,
                                nullable: false,
                            });
                        }
                    }
                }
            }
            // else: drop fk (Cancel)
        } else {
            // Put it back if not closing
            self.current_fk = Some(fk);
        }
    }

    /// Draw the primary key constraint
    fn draw_pk(&mut self, ui: &mut Ui) {
        if self.pk.attributes.len() > 0 {
            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, RED))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(RichText::new("🔑").color(RED)));

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

    /// Draw ForeignKey constraints
    pub fn draw_fks(&mut self, ui: &mut Ui) {
        if self.fks.len() < 1 {
            return;
        }

        for (_, fk) in &self.fks {
            fk.display(ui);
        }
    }

    /// Draw all attributes
    fn draw_attributes(
        &mut self,
        ui: &mut Ui,
        current_id: TableId,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
    ) {
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

        self.draw_pk(ui);
        self.draw_fks(ui);

        self.handle_fk_modal(ui, tables);

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                self.new_field();
            }
            if ui.button("Add FK").clicked() {
                self.current_fk = Some(ForeignKey::new());
            }
        });

        ui.separator();
    }
}
