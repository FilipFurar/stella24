use eframe::epaint::Color32;
use egui::RichText;
use crate::app::{DomainId, TableId};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use crate::model::field::{FieldId, FieldType};
use egui::Ui;
use slotmap::{Key, SlotMap};

const GREEN: Color32 = Color32::from_rgb(66, 170, 125);
const BLUE: Color32 = Color32::from_rgb(75, 67, 185);
const RED: Color32 = Color32::from_rgb(194, 73, 125);


impl DataType {
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

impl FieldType {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        id: FieldId,
        domains: &SlotMap<DomainId, Domain>,
        tables: &SlotMap<TableId, Table>,
    ) {
        let mut fk_selection: Option<(Option<TableId>, Option<FieldId>)> = None;

        if let FieldType::ForeignKey(fk) = self {
            let selected_table = fk.referenced_table()
                .and_then(|tid| tables.get(tid))
                .map(|t| t.title.clone())
                .unwrap_or_default();

            let selected_field = fk.referenced_field()
                .and_then(|fid| {
                    let table_id = fk.referenced_table()?;
                    let table = tables.get(table_id)?;
                    table.pk().get(fid).map(|f| f.name.clone())
                })
                .unwrap_or_default();

            let mut new_table: Option<TableId> = None;
            let mut new_field: Option<FieldId> = None;

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(format!("ref_table_{}", id.data().as_ffi()))
                    .selected_text(&selected_table)
                    .show_ui(ui, |ui| {
                        for (table_id, table) in tables {
                            let is_selected = fk.referenced_table() == Some(table_id);
                            if ui.selectable_label(is_selected, &table.title).clicked() {
                                new_table = Some(table_id);
                                new_field = None;
                            }
                        }
                    });

                egui::ComboBox::from_id_salt(format!("ref_field_{}", id.data().as_ffi()))
                    .selected_text(&selected_field)
                    .show_ui(ui, |ui| {
                        if let Some(tid) = fk.referenced_table() {
                            if let Some(table) = tables.get(tid) {
                                for (field_id, field) in table.pk() {
                                    let is_selected = fk.referenced_field() == Some(field_id);
                                    if ui.selectable_label(is_selected, &field.name).clicked() {
                                        new_field = Some(field_id);
                                    }
                                }
                            }
                        }
                    });
                ui.label(RichText::new("FK").color(BLUE));
            });

            fk_selection = Some((new_table, new_field));
        }

        if let Some((table, field)) = fk_selection {
            if let FieldType::ForeignKey(fk) = self {
                if let Some(t) = table {
                    fk.set_referenced_table(t);
                }
                if let Some(f) = field {
                    fk.set_referenced_field(f);
                }
            }
        }

        if !matches!(self, FieldType::ForeignKey(_)) {
            let selected_text = match self {
                FieldType::Data(dt) => DATA_TYPES[dt.base].name.to_string(),
                FieldType::Domain(i) => domains
                    .get(*i)
                    .map(|d| d.name.clone())
                    .unwrap_or("Invalid domain".into()),
                FieldType::ForeignKey(_) => unreachable!(),
            };

            egui::ComboBox::from_id_salt(format!("type_{}", id.data().as_ffi()))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (i, def) in DATA_TYPES.iter().enumerate() {
                        if ui.selectable_label(false, def.name).clicked() {
                            *self = FieldType::Data(DataType {
                                base: i,
                                params: vec![0; def.param_count],
                            });
                        }
                    }

                    if !domains.is_empty() {
                        ui.separator();
                        for (i, domain) in domains.iter() {
                            if ui.selectable_label(false, &domain.name).clicked() {
                                *self = FieldType::Domain(i);
                            }
                        }
                    }
                });

            if let FieldType::Data(dt) = self {
                dt.draw_params(ui);
            }
        }
    }
}