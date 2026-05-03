// ui/entities/table.rs

use crate::app::{Command, TableId};
use crate::model::attribute::{AttrId, AttributeType};
use crate::model::constraints::check::Check;
use crate::model::constraints::constraint::{FkId, ForeignKey, Unique};
use crate::model::entities::table::Table;
use crate::ui::constraints::check::draw_check;
use crate::ui::context::TableUiContext;
use eframe::emath::{Rect, pos2, vec2};
use eframe::epaint::Color32;
use egui::{Area, Id, Modal, RichText, Sense, Stroke, Ui};
use std::collections::HashSet;

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
    pub commands: Vec<Command>,
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

    fn handle_fk_modal(&mut self, ui: &mut Ui, ctx: &TableUiContext) -> Option<ForeignKey> {
        // Take ownership out of Option, put back if not closing
        let mut fk = match self.current_fk.take() {
            Some(fk) => fk,
            None => return None,
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
                    return Some(fk);
                }
                // else: drop fk (validation failed)
            }
            // else: drop fk (cancelled)
        } else {
            // Put it back if not closing
            self.current_fk = Some(fk);
        }

        None
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
    pub fn draw_fks(&mut self, ui: &mut Ui) -> Vec<FkId> {
        if self.fks.is_empty() {
            return Vec::new();
        }

        let mut to_delete: Vec<FkId> = Vec::new();

        for (fkid, fk) in &mut self.fks {
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

        for fkid in &to_delete {
            if let Some(fk) = self.fks.remove(*fkid) {
                for attid in fk.local_attrs {
                    self.attributes.remove(attid);
                }
            }
        }

        to_delete
    }

    pub fn draw_uniques(&mut self, ui: &mut Ui, table_id: TableId) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut to_delete: Vec<usize> = Vec::new();

        for (i, unique) in &mut self.uniques.iter_mut().enumerate() {
            let before_name = unique.name.clone();
            egui::Frame::group(ui.style())
                .stroke(Stroke::new(1.0, GREEN))
                .show(ui, |ui| {
                    unique.draw(ui, &self.attributes);
                    ui.horizontal(|ui| {
                        if ui.button("Edit").clicked() {
                            self.current_unique = Some(i);
                        }
                        if ui.button("🗑").clicked() {
                            to_delete.push(i);
                        }
                    });
                });

            if unique.name != before_name {
                commands.push(Command::RenameUnique {
                    table: table_id,
                    index: i,
                    name: unique.name.clone(),
                });
            }
        }

        for i in to_delete.into_iter().rev() {
            self.remove_unique(i);
            if self.current_unique == Some(i) {
                self.current_unique = None;
            } else if let Some(current) = self.current_unique
                && i < current
            {
                self.current_unique = Some(current - 1);
            }

            commands.push(Command::DeleteUnique {
                table: table_id,
                index: i,
            });
        }

        commands
    }

    pub fn draw_checks(&mut self, ui: &mut Ui) -> Vec<usize> {
        let mut to_delete: Vec<usize> = Vec::new();

        for (i, check) in self.checks.iter_mut().enumerate() {
            if draw_check(ui, check, ("table_check", i), "sql") {
                to_delete.push(i);
            }
        }

        for i in to_delete.iter().copied().rev() {
            self.checks.remove(i);
        }

        to_delete
    }

    fn handle_unique_modal(&mut self, ui: &mut Ui, table_id: TableId) -> Vec<Command> {
        let Some(unique_index) = self.current_unique else {
            return Vec::new();
        };

        if unique_index >= self.uniques.len() {
            self.current_unique = None;
            return Vec::new();
        }

        let before_name = self.uniques[unique_index].name.clone();
        let before_attrs = self.uniques[unique_index].attributes.clone();

        let should_close = {
            let unique = &mut self.uniques[unique_index];
            unique.attribute_modal(ui, &self.attributes, unique_index)
        };

        let mut commands = Vec::new();

        if should_close {
            let unique = &self.uniques[unique_index];
            if unique.name != before_name {
                commands.push(Command::RenameUnique {
                    table: table_id,
                    index: unique_index,
                    name: unique.name.clone(),
                });
            }

            for attr in unique.attributes.difference(&before_attrs) {
                commands.push(Command::AddUniqueAttribute {
                    table: table_id,
                    index: unique_index,
                    attr: *attr,
                });
            }

            for attr in before_attrs.difference(&unique.attributes) {
                commands.push(Command::RemoveUniqueAttribute {
                    table: table_id,
                    index: unique_index,
                    attr: *attr,
                });
            }

            self.current_unique = None;
        }

        commands
    }

    /// Draw all attributes
    /// Helper to format AttributeType for the ghost preview
    fn format_attr_type(attr_type: &AttributeType) -> String {
        match attr_type {
            AttributeType::Logical(dt) => format!("{:?}", dt), // or however DataType displays
            AttributeType::Domain(_) => "Domain".to_string(),
            AttributeType::ForeignKeyAttribute(_) => "FK".to_string(),
        }
    }

    fn draw_attributes(&mut self, ui: &mut Ui, ctx: &TableUiContext) -> Vec<AttributeRowChanges> {
        let mut result = Vec::new();

        // AttrOrder is synced with SlotMap
        let existing: HashSet<_> = self.attributes.keys().collect();
        self.attr_order.retain(|id| existing.contains(id));
        for id in self.attributes.keys() {
            if !self.attr_order.contains(&id) {
                self.attr_order.push(id);
            }
        }

        let attrs_in_table_uniques: HashSet<AttrId> = self
            .uniques
            .iter()
            .flat_map(|unique| unique.attributes.iter().copied())
            .collect();

        let mut row_rects: Vec<(AttrId, Rect)> = Vec::new();
        let mut drop_index: Option<usize> = None;

        // Draw every row in the explicit order
        for (index, &id) in self.attr_order.iter().enumerate() {
            let Some(attr) = self.attributes.get_mut(id) else {
                continue;
            };

            let is_dragged = self.dragged_attr == Some(id);

            let row_response = ui.scope(|ui| {
                ui.horizontal(|ui| {
                    // Drag handle
                    let handle = ui
                        .add(egui::Label::new("≡").sense(Sense::drag()))
                        .on_hover_text("Drag to reorder")
                        .on_hover_cursor(egui::CursorIcon::Grab);
                    if handle.drag_started() {
                        self.dragged_attr = Some(id);
                        self.dragged_from_index = Some(index);
                    }

                    // ── Your existing attribute editor, untouched ──
                    let disable_inline_unique = attrs_in_table_uniques.contains(&id);
                    let changes = attr.draw_attribute(ui, id, ctx, disable_inline_unique);

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
                });
            });

            let rect = row_response.response.rect;
            row_rects.push((id, rect));

            // Detect drop zone
            if self.dragged_attr.is_some() && !is_dragged {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if rect.contains(pointer_pos) {
                        let center_y = rect.center().y;
                        drop_index = Some(if pointer_pos.y > center_y {
                            index + 1
                        } else {
                            index
                        });
                    }
                }
            }
        }

        // Draw insertion line
        if let Some(idx) = drop_index {
            let y = if idx == 0 {
                row_rects.first().map(|(_, r)| r.top())
            } else {
                row_rects
                    .get(idx.saturating_sub(1))
                    .map(|(_, r)| r.bottom())
            }
            .unwrap_or_else(|| ui.min_rect().top());

            if let Some((_, first_rect)) = row_rects.first() {
                let color = ui.visuals().selection.bg_fill;
                ui.painter().line_segment(
                    [pos2(first_rect.left(), y), pos2(first_rect.right(), y)],
                    Stroke::new(2.0, color),
                );
            }
        }

        // Floating ghost
        if let Some(dragged_id) = self.dragged_attr {
            if let Some(attr) = self.attributes.get(dragged_id) {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    Area::new(Id::new("attr_drag_ghost"))
                        .fixed_pos(pointer_pos - vec2(0.0, 12.0))
                        .interactable(false)
                        .show(ui.ctx(), |ui| {
                            let frame = egui::Frame::popup(ui.style());
                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("≡");
                                    ui.label(format!(
                                        "{} : {}",
                                        attr.name,
                                        Self::format_attr_type(&attr.attribute_type)
                                    ));
                                });
                            });
                        });
                }
            }

            // ── 7. On mouse release: apply reorder ────────────────────────────
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(from_idx) = self.dragged_from_index {
                    let to_idx = drop_index.unwrap_or(from_idx);

                    if from_idx != to_idx {
                        let id = self.attr_order.remove(from_idx);
                        let to_idx = if to_idx > from_idx {
                            to_idx - 1
                        } else {
                            to_idx
                        };
                        let to_idx = to_idx.min(self.attr_order.len());
                        self.attr_order.insert(to_idx, id);
                    }
                }
                self.dragged_attr = None;
                self.dragged_from_index = None;
            }
        }

        result
    }

    pub fn draw(&mut self, ui: &mut Ui, ctx: &TableUiContext, table_id: TableId) -> TableChanges {
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
        for fkid in self.draw_fks(ui) {
            changes.commands.push(Command::DeleteForeignKey {
                table: table_id,
                fk: fkid,
            });
        }

        changes.commands.extend(self.draw_uniques(ui, table_id));
        for index in self.draw_checks(ui).into_iter().rev() {
            changes.commands.push(Command::DeleteTableCheck {
                table: table_id,
                index,
            });
        }
        changes
            .commands
            .extend(self.handle_unique_modal(ui, table_id));
        //self.draw_not_nulls(ui);

        if let Some(fk) = self.handle_fk_modal(ui, ctx) {
            changes.commands.push(Command::AddForeignKey {
                table: table_id,
                foreign_key: fk,
            });
        }
        //self.handle_attribute_modal(ui);

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                changes.add_attribute = true;
            }
            if ui.button("New FK").clicked() {
                self.current_fk = Some(ForeignKey::new());
            }
            if ui.button("New Unique").clicked() {
                changes.commands.push(Command::AddUnique {
                    table: table_id,
                    unique: Unique::new(),
                });
            }
            if ui.button("New Check").clicked() {
                changes.commands.push(Command::AddTableCheck {
                    table: table_id,
                    check: Check::new(),
                });
            }
        });

        changes
    }
}
