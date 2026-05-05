// ui/constraints/attribute

use crate::model::attribute::{AttrId, Attribute, AttributeCategory, AttributeType};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::ui::context::TableUiContext;
use eframe::epaint::{Color32, Stroke};
use egui::Ui;
use egui::{Popup, PopupCloseBehavior};
use slotmap::Key;

const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

/// A struct for storing UI changes per attribute row
#[derive(Default)]
pub struct AttributeChanges {
    pub rename_changed: bool,
    pub type_changed: bool,
    pub not_null_changed: bool,
    pub unique_changed: bool,
    pub pk_change: Option<bool>,
    pub delete: bool,
}

impl DataType {
    /// Draw all parameters of the data type
    pub fn draw_params(&mut self, ui: &mut Ui) {
        if !self.params.is_empty() {
            ui.horizontal(|ui| {
                ui.label("(");
                for param in self.params.iter_mut() {
                    ui.add(egui::DragValue::new(param).speed(1).range(0..=1_000_000));
                }
                ui.label(")");
            });
        }
    }
}

impl Attribute {
    /// Draw a single attribute
    pub fn draw_attribute(
        &mut self,
        ui: &mut Ui,
        id: AttrId,
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
                        changes.rename_changed = true;
                    }

                    if let AttributeType::Logical(_) | AttributeType::Domain(_) =
                        &self.attribute_type
                    {
                        if self.attribute_type.draw_compact(ui, id, ctx) {
                            changes.type_changed = true;
                        }
                    }

                    ui.add_enabled_ui(!self.pk, |ui| {
                        if ui.checkbox(&mut self.not_null, "NN").changed() {
                            changes.not_null_changed = true;
                        }
                    });

                    ui.add_enabled_ui(!disable_inline_unique, |ui| {
                        if ui.checkbox(&mut self.unique, "U").changed() {
                            changes.unique_changed = true;
                        }
                    });

                    if self.pk {
                        if !self.not_null {
                            self.not_null = true;
                            changes.not_null_changed = true;
                        }
                    }

                    let mut is_pk = self.pk;
                    if ui.checkbox(&mut is_pk, "🔑").changed() {
                        self.pk = is_pk;
                        changes.pk_change = Some(is_pk);
                    }

                    if ui.button("🗑").clicked() {
                        changes.delete = true;
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
            AttributeType::Logical(dt) => {
                let base_name = DATA_TYPES[dt.base].name;
                let params = if dt.params.is_empty() {
                    String::new()
                } else {
                    format!(
                        "({})",
                        dt.params
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!("{}{}", base_name, params)
            }
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
                    AttributeCategory::Logical => AttributeType::Logical(DataType {
                        base: 0,
                        params: vec![],
                    }),
                    AttributeCategory::Domain => ctx
                        .domains
                        .first()
                        .map(|domain| AttributeType::Domain(domain.id))
                        .unwrap_or_else(|| {
                            AttributeType::Logical(DataType {
                                base: 0,
                                params: vec![],
                            })
                        }),
                    AttributeCategory::ForeignKey => AttributeType::ForeignKeyAttribute(id),
                };
                changed = true;
            }

            match self {
                AttributeType::Logical(dt) => {
                    egui::ComboBox::from_id_salt(format!("logical_{}", id.data().as_ffi()))
                        .selected_text(DATA_TYPES[dt.base].name)
                        .show_ui(ui, |ui| {
                            for (i, def) in DATA_TYPES.iter().enumerate() {
                                if ui.selectable_label(dt.base == i, def.name).clicked() {
                                    dt.base = i;
                                    dt.params = vec![0; def.param_count];
                                    changed = true;
                                }
                            }
                        });

                    for param in dt.params.iter_mut() {
                        if ui
                            .add(egui::DragValue::new(param).speed(1).range(0..=1_000_000))
                            .changed()
                        {
                            changed = true;
                        }
                    }
                }
                AttributeType::Domain(domain_id) => {
                    let name_option = ctx.domain_name(*domain_id);
                    let mut selected: &str = "";

                    if name_option.is_some() {
                        selected = ctx.domain_name(*domain_id).expect("err");
                    }
                    if name_option.is_none() {
                        selected = "";
                    }

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
