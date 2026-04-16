// ui/entities/table.rs

use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::{FkId, ForeignKey, Unique};
use crate::model::entities::table::Table;
use crate::ui::context::TableUiContext;
use eframe::epaint::Color32;
use egui::{Id, Modal, RichText, Stroke, Ui};

const RED: Color32 = Color32::from_rgb(194, 73, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const GREEN: Color32 = Color32::from_rgb(66, 170, 125);

#[derive(Default, Debug)]
pub struct AttributeRowChanges {
    pub attr_id: AttrId,
    pub rename_changed: bool,
    pub type_changed: bool,
    pub not_null_changed: bool,
    pub unique_changed: bool,
    pub pk_change: Option<bool>,
    pub delete: bool,
}

#[derive(Default)]
pub struct TableChanges {
    pub title_changed: bool,
    pub add_attribute: bool,
    pub attribute_changes: Vec<AttributeRowChanges>,
}

impl Table {
    /*fn handle_attribute_modal(&mut self, ui: &mut Ui) {
        let mut should_close = false;
        let mut save_to_attributes = false;

        let mut attr = match self.current_attr.take() {
            Some(attr) => attr,
            None => return,
        };

        Modal::new(Id::new("attribute_modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("New Attribute");

            ui.label("Name:");
            ui.text_edit_singleline(&mut attr.name);

        });
    }*/

    fn handle_fk_modal(&mut self, ui: &mut Ui, ctx: &TableUiContext) {
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
                .and_then(|id| ctx.table_title(id))
                .map(|t| t.to_string())
                .unwrap_or_else(|| "No table selected".to_string());

            egui::ComboBox::from_id_salt("fk_table_pick")
                .selected_text(&selected_text)
                .show_ui(ui, |ui| {
                    if ctx.tables.is_empty() {
                        ui.weak("No tables available");
                    } else {
                        for table in &ctx.tables {
                            ui.selectable_value(&mut fk.references, Some(table.id), &table.title);
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
                // Check prerequisites before saving
                let can_save = fk
                    .references
                    .map(|id| ctx.table_has_pk(id))
                    .unwrap_or(false);

                if can_save {
                    if let Some(other_table_id) = fk.references {
                        let fk_name = fk.name.clone();
                        if let Some(pk_attrs) = ctx.table_pk_attributes(other_table_id) {
                            for (other_attr_id, other_attr_name) in pk_attrs {
                                let local_attr = Attribute {
                                    name: format!("{}_{}", fk_name, other_attr_name),
                                    attribute_type: AttributeType::ForeignKeyAttribute(
                                        *other_attr_id,
                                    ),
                                    pk: false,
                                    not_null: false,
                                    unique: false,
                                };
                                let local_attr_key = self.attributes.insert(local_attr);
                                fk.local_attrs.insert(local_attr_key);
                            }
                        }
                    }

                    // Insert the fully-built FK
                    self.fks.insert(fk);
                }
                // else: drop fk (validation failed)
            }
            // else: drop fk (cancelled)
        } else {
            // Put it back if not closing
            self.current_fk = Some(fk);
        }
    }

    /// Draw the primary key constraint
    fn draw_pk(&mut self, ui: &mut Ui) {
        if !self.pk.attributes.is_empty() {
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
        if self.fks.is_empty() {
            return;
        }

        let mut to_delete: Vec<FkId> = Vec::new();

        for (fkid, fk) in &mut self.fks {
            let fkid = fkid;

            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, BLUE))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        fk.draw(ui);
                        if ui.button("🗑").clicked() {
                            to_delete.push(fkid);
                        }
                    });
                });
        }

        // Batch delete with cleanup
        for fkid in to_delete {
            if let Some(fk) = self.fks.remove(fkid) {
                for attid in fk.local_attrs {
                    self.attributes.remove(attid);
                }
            }
        }
    }

    pub fn draw_uniques(&mut self, ui: &mut Ui) {
        let mut to_delete: Vec<usize> = Vec::new();

        for (i, unique) in &mut self.uniques.iter_mut().enumerate() {
            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, GREEN))
                .show(ui, |ui| {
                    unique.draw(ui, &self.attributes);
                    if ui.button("🗑").clicked() {
                        to_delete.push(i);
                    }
                });
        }

        for i in to_delete {
            self.uniques.remove(i);
        }
    }

    /// Draw all attributes
    fn draw_attributes(&mut self, ui: &mut Ui, ctx: &TableUiContext) -> Vec<AttributeRowChanges> {
        let mut result = Vec::new();

        for (id, attr) in self.attributes_mut() {
            let changes = attr.draw_attribute(ui, id, ctx);
            if changes.rename_changed
                || changes.type_changed
                || changes.not_null_changed
                || changes.unique_changed
                || changes.pk_change.is_some()
                || changes.delete
            {
                result.push(AttributeRowChanges {
                    attr_id: id,
                    rename_changed: changes.rename_changed,
                    type_changed: changes.type_changed,
                    not_null_changed: changes.not_null_changed,
                    unique_changed: changes.unique_changed,
                    pk_change: changes.pk_change,
                    delete: changes.delete,
                });
            }
        }

        result
    }

    pub fn draw(&mut self, ui: &mut Ui, ctx: &TableUiContext) -> TableChanges {
        let mut changes = TableChanges::default();

        ui.horizontal(|ui| {
            ui.label("Title:");
            if ui.text_edit_singleline(&mut self.title).changed() {
                changes.title_changed = true;
            }
        });
        ui.separator();

        changes.attribute_changes = self.draw_attributes(ui, ctx);

        self.draw_pk(ui);
        self.draw_fks(ui);

        self.draw_uniques(ui);
        //self.draw_not_nulls(ui);

        self.handle_fk_modal(ui, ctx);
        //self.handle_attribute_modal(ui);

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                changes.add_attribute = true;
            }
            if ui.button("Add FK").clicked() {
                self.current_fk = Some(ForeignKey::new());
            }
            if ui.button("Add U").clicked() {
                self.uniques.push(Unique::new());
            }
        });

        ui.separator();
        changes
    }
}
