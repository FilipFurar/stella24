// ui/constraints/attribute

use crate::app::{Command, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeCategory, AttributeType};
use crate::model::datatype::{CharOrByte, DataType};
use crate::ui::changes::IntoCommands;
use crate::ui::context::TableUiContext;
use eframe::epaint::{Color32, Stroke};
use egui::Ui;
use egui::{Popup, PopupCloseBehavior};
use slotmap::Key;

const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

/// A struct for storing staged commands from an attribute row.
#[derive(Default, Debug)]
pub struct AttributeChanges {
    pub commands: Vec<Command>,
}

impl IntoCommands for AttributeChanges {
    fn into_commands(self) -> Vec<Command> {
        self.commands
    }
}

impl DataType {
    /// Draw all parameters of the data type
    pub fn draw_params(&mut self, ui: &mut Ui, id_salt: impl std::hash::Hash) -> bool {
        self.normalize_params();

        if self.params.is_empty() {
            return false;
        }

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("(");

            if self.param_name(1) == Some("length_semantics") {
                let size_name = self.param_name(0);
                let semantics_name = self.param_name(1);

                if let Some(param) = self.params.get_mut(0) {
                    let mut drag_value =
                        ui.add(egui::DragValue::new(param).speed(1).range(0..=40_000));

                    if let Some(name) = size_name {
                        drag_value = drag_value.on_hover_text(name);
                    }

                    if drag_value.changed() {
                        changed = true;
                    }
                }

                let mut selected = if self.params.get(1).copied().unwrap_or(0) == 1 {
                    CharOrByte::Char
                } else {
                    CharOrByte::Byte
                };
                let selected_before = selected;

                let combo = egui::ComboBox::from_id_salt((id_salt, "length_semantics"))
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, CharOrByte::Char, "Char");
                        ui.selectable_value(&mut selected, CharOrByte::Byte, "Byte");
                    })
                    .response;

                if let Some(name) = semantics_name {
                    combo.on_hover_text(name);
                }

                if selected != selected_before {
                    if let Some(semantics) = self.params.get_mut(1) {
                        *semantics = if selected == CharOrByte::Char { 1 } else { 0 };
                    }
                    changed = true;
                }
            } else {
                for param_index in 0..self.params.len() {
                    let param_name = self.param_name(param_index);
                    if let Some(param) = self.params.get_mut(param_index) {
                        let mut drag_value =
                            ui.add(egui::DragValue::new(param).speed(1).range(0..=1_000_000));

                        if let Some(name) = param_name {
                            drag_value = drag_value.on_hover_text(name);
                        }

                        if drag_value.changed() {
                            changed = true;
                        }
                    }
                }
            }

            ui.label(")");
        });

        changed
    }
}

impl Attribute {
    /// Draw a single attribute
    pub fn draw_attribute(
        &mut self,
        ui: &mut Ui,
        id: AttrId,
        table_id: TableId,
        ctx: &TableUiContext,
        disable_inline_unique: bool,
    ) -> AttributeChanges {
        let mut changes = AttributeChanges::default();
        let stroke = Color32::DARK_GRAY;

        egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, stroke))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.name)
                                .desired_width(10.0)
                                .clip_text(false),
                        )
                        .changed()
                    {
                        changes.commands.push(Command::RenameAttribute {
                            table: table_id,
                            attr: id,
                            name: self.name.clone(),
                        });
                    }

                    if let AttributeType::Logical(_) | AttributeType::Domain(_) =
                        &self.attribute_type
                        && self.attribute_type.draw_compact(ui, id, ctx)
                    {
                        changes.commands.push(Command::SetAttributeType {
                            table: table_id,
                            attr: id,
                            attribute_type: self.attribute_type.clone(),
                        });
                    }

                    ui.add_enabled_ui(!self.pk, |ui| {
                        if ui.checkbox(&mut self.not_null, "NN").changed() {
                            changes.commands.push(Command::SetAttributeNotNull {
                                table: table_id,
                                attr: id,
                                value: self.not_null,
                            });
                        }
                    });

                    ui.add_enabled_ui(!disable_inline_unique, |ui| {
                        if ui.checkbox(&mut self.unique, "U").changed() {
                            changes.commands.push(Command::SetAttributeUnique {
                                table: table_id,
                                attr: id,
                                value: self.unique,
                            });
                        }
                    });

                    if self.pk && !self.not_null {
                        self.not_null = true;
                        changes.commands.push(Command::SetAttributeNotNull {
                            table: table_id,
                            attr: id,
                            value: self.not_null,
                        });
                    }

                    let mut is_pk = self.pk;
                    if ui.checkbox(&mut is_pk, "🔑").changed() {
                        self.pk = is_pk;
                        changes.commands.push(Command::SetAttributePrimaryKey {
                            table: table_id,
                            attr: id,
                            value: is_pk,
                        });
                    }

                    if ui.button("🗑").clicked() {
                        changes.commands.push(Command::DeleteAttribute {
                            table: table_id,
                            attr: id,
                        });
                    }
                });
            });
        changes
    }
}

impl AttributeType {
    /// Returns the text that should be displayed for each type of attribute
    pub fn display_text(&self, ctx: &TableUiContext) -> String {
        match self {
            AttributeType::Logical(dt) => dt.display_text(),
            AttributeType::Domain(did) => ctx
                .domain_name(*did)
                .map(|name| name.to_string())
                .unwrap_or_else(|| "Invalid domain".to_string()),
            AttributeType::ForeignKeyAttribute(fk) => ctx
                .current_table_attribute_name(*fk)
                .map(|name| format!("FK -> {}", name))
                .unwrap_or_else(|| "attr err".to_string()),
        }
    }

    /// Map an AttributeCategory to AttributeType
    pub fn category(&self) -> AttributeCategory {
        match self {
            AttributeType::Logical(_) => AttributeCategory::Logical,
            AttributeType::Domain(_) => AttributeCategory::Domain,
            AttributeType::ForeignKeyAttribute(_) => AttributeCategory::ForeignKey,
        }
    }

    /// Drawa single FK attribute
    pub fn draw_fk_attribute(&mut self, ui: &mut Ui, referenced_attribute: &Attribute) {
        egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, BLUE))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&referenced_attribute.name);
                    //ui.label(&referenced_attribute.attribute_type)
                });
            });
    }

    pub fn draw_compact(&mut self, ui: &mut Ui, id: AttrId, ctx: &TableUiContext) -> bool {
        let mut changed = false;
        let button_text = self.display_text(ctx);
        let button_response = ui.button(&button_text);
        let popup_id = ui.make_persistent_id(format!("type_popup_{}", id.data().as_ffi()));

        if button_response.clicked() {
            let was_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
            ui.data_mut(|d| d.insert_temp(popup_id, !was_open));
        }

        let is_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

        if !is_open {
            return false;
        }

        let _inner = Popup::from_response(&button_response)
            .id(popup_id)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                ui.set_min_width(250.0);
                if self.draw_popup(ui, id, ctx) {
                    changed = true;
                }
            });

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|d| d.insert_temp(popup_id, false));
        }

        changed
    }

    /// Draws the popup for type selection
    fn draw_popup(&mut self, ui: &mut Ui, id: AttrId, ctx: &TableUiContext) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            let current_category = self.category();
            let mut new_category = current_category;

            egui::ComboBox::from_id_salt(format!("cat_{}", id.data().as_ffi()))
                .selected_text(format!("{:?}", current_category))
                .show_ui(ui, |ui| {
                    for cat in [AttributeCategory::Logical, AttributeCategory::Domain] {
                        if ui
                            .selectable_label(current_category == cat, format!("{:?}", cat))
                            .clicked()
                        {
                            new_category = cat;
                        }
                    }
                });

            if new_category != current_category {
                *self = match new_category {
                    AttributeCategory::Logical => AttributeType::Logical(
                        DataType::default_for_dialect(ctx.selected_sql_dialect),
                    ),
                    AttributeCategory::Domain => ctx
                        .domains
                        .first()
                        .map(|domain| AttributeType::Domain(domain.id))
                        .unwrap_or_else(|| {
                            AttributeType::Logical(DataType::default_for_dialect(
                                ctx.selected_sql_dialect,
                            ))
                        }),
                    AttributeCategory::ForeignKey => AttributeType::ForeignKeyAttribute(id),
                };
                changed = true;
            }

            match self {
                AttributeType::Logical(dt) => {
                    let catalog = dt.catalog();
                    egui::ComboBox::from_id_salt(format!("logical_{}", id.data().as_ffi()))
                        .selected_text(dt.type_name())
                        .show_ui(ui, |ui| {
                            for i in 0..catalog.len() {
                                let Some(name) = catalog.name(i) else {
                                    continue;
                                };
                                if ui.selectable_label(dt.base == i, name).clicked() {
                                    *dt = DataType::new(ctx.selected_sql_dialect, i);
                                    changed = true;
                                }
                            }
                        });

                    if dt.draw_params(ui, id.data().as_ffi()) {
                        changed = true;
                    }
                }
                AttributeType::Domain(domain_id) => {
                    let name_option = ctx.domain_name(*domain_id);

                    let selected: &str = match name_option {
                        None => "",
                        Some(_) => ctx.domain_name(*domain_id).expect("err"),
                    };

                    egui::ComboBox::from_id_salt(format!("domain_{}", id.data().as_ffi()))
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            for domain in &ctx.domains {
                                if ui
                                    .selectable_label(*domain_id == domain.id, &domain.name)
                                    .clicked()
                                {
                                    *domain_id = domain.id;
                                    changed = true;
                                }
                            }
                        });
                }
                _ => {}
            }
        });

        changed
    }
}
