// ui/constraints/attribute

use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeCategory, AttributeType};
use crate::model::constraints::constraint::{FkId, ForeignKey};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use eframe::epaint::{Color32, Stroke};
use egui::RichText;
use egui::Ui;
use egui::{Popup, PopupCloseBehavior};
use slotmap::{Key, SlotMap};

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const RED: Color32 = Color32::from_rgb(194, 73, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);

/// A struct for storing UI changes, like attribute deletion or adding/removing from PK
#[derive(Default)]
pub struct AttributeChanges {
    /// Stores the attribute to add or remove from primary key
    pub pk_change: Option<(AttrId, bool)>,

    /// Stores the attribute to delete
    pub delete: Option<AttrId>,
}

impl DataType {
    /// Draw all parameters of the data type
    pub fn draw_params(&mut self, ui: &mut Ui) {
        if !self.params.is_empty() {
            ui.horizontal(|ui| {
                ui.label("(");
                for (_i, param) in self.params.iter_mut().enumerate() {
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
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
        current_table: TableId,
    ) -> AttributeChanges {
        let mut changes = AttributeChanges::default();
        let stroke = Color32::DARK_GRAY;

        egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, stroke))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(75.0));

                    let display_text =
                        self.attribute_type
                            .display_text(domains, tables, current_table);

                    match &self.attribute_type {
                        AttributeType::Logical(datatype) => {
                            self.attribute_type.draw_compact(
                                ui,
                                id,
                                domains,
                                tables,
                                current_table,
                            );
                        }
                        AttributeType::Domain(domain) => {
                            self.attribute_type.draw_compact(
                                ui,
                                id,
                                domains,
                                tables,
                                current_table,
                            );
                        }
                        AttributeType::ForeignKeyAttribute(fk_attr) => {
                            //self.attribute_type.draw_fk_attribute(ui, tables.get())
                        }
                    }

                    let mut is_pk = self.pk;
                    if ui.checkbox(&mut is_pk, "🔑").changed() {
                        self.pk = is_pk;
                        changes.pk_change = Some((id, is_pk));
                    }

                    if ui.button("🗑").clicked() {
                        changes.delete = Some(id);
                    }
                });
            });
        changes
    }
}

impl AttributeType {
    /// Returns the text that should be displayed for each type of attribute
    pub fn display_text(
        &self,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
        current_table: TableId,
    ) -> String {
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
            AttributeType::Domain(did) => domains
                .get(*did)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Invalid domain".to_string()),
            AttributeType::ForeignKeyAttribute(fk) => {
                if let Some(table) = tables.get(current_table) {
                    if let Some(attribute) = table.attributes.get(*fk) {
                        return format!("FK → {}", attribute.name);
                    }
                    return "attr err".to_string();
                }
                "table err".to_string()
            }
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

    pub fn draw_compact(
        &mut self,
        ui: &mut Ui,
        id: AttrId,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
        current_table: TableId,
    ) {
        let button_text = self.display_text(domains, tables, current_table);
        let button_response = ui.button(&button_text);
        let popup_id = ui.make_persistent_id(format!("type_popup_{}", id.data().as_ffi()));

        if button_response.clicked() {
            let was_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
            ui.data_mut(|d| d.insert_temp(popup_id, !was_open));
        }

        let is_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

        if !is_open {
            return;
        }

        let inner = Popup::from_response(&button_response)
            .id(popup_id)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                ui.set_min_width(250.0);
                self.draw_popup(ui, id, domains, tables, current_table);
            });

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|d| d.insert_temp(popup_id, false));
            return;
        }
    }

    /// Draws the popup for type selection
    fn draw_popup(
        &mut self,
        ui: &mut Ui,
        id: AttrId,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
        current_table: TableId,
    ) {
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
                    AttributeCategory::Domain => domains
                        .keys()
                        .next()
                        .map(|did| AttributeType::Domain(did))
                        .unwrap_or_else(|| {
                            AttributeType::Logical(DataType {
                                base: 0,
                                params: vec![],
                            })
                        }),
                    AttributeCategory::ForeignKey => AttributeType::ForeignKeyAttribute(id),
                };
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
                                }
                            }
                        });

                    for (i, param) in dt.params.iter_mut().enumerate() {
                        ui.add(egui::DragValue::new(param).speed(1).range(0..=1_000_000));
                    }
                }
                AttributeType::Domain(domain_id) => {
                    let selected = domains.get(*domain_id).map(|d| &d.name).expect("err");
                    egui::ComboBox::from_id_salt(format!("domain_{}", id.data().as_ffi()))
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            for (did, domain) in domains {
                                if ui
                                    .selectable_label(*domain_id == did, &domain.name)
                                    .clicked()
                                {
                                    *domain_id = did;
                                }
                            }
                        });
                }
                AttributeType::ForeignKeyAttribute(fk) => {
                    /*let selected = fk.references
                        .and_then(|tid| tables.get(tid))
                        .map(|t| t.title.clone())
                        .unwrap_or_else(|| "Select table".to_string());

                    egui::ComboBox::from_id_salt(format!("fk_{}", id.data().as_ffi()))
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            for (tid, table) in tables {
                                if tid == current_table { continue; }
                                if ui.selectable_label(fk.references == Some(tid), &table.title).clicked() {
                                    fk.references = Some(tid);
                                }
                            }
                        });*/
                }
            }
        });
    }
}
