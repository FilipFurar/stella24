use egui::Ui;
use crate::model::domain::Domain;
use super::datatype::{DataType, DATA_TYPES};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: Box<dyn FieldType>,
    pub nullable: bool,
    pub primary_key: bool,
}

/*#[derive(serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    Data(DataType),
    Domain(Domain)
}*/

#[typetag::serde(tag = "type")]
pub trait FieldType {
    fn draw(&mut self, ui: &mut Ui, id: usize);
}

#[typetag::serde]
impl FieldType for DataType {
    fn draw(&mut self, ui: &mut Ui, id: usize) {
        egui::ComboBox::from_id_salt(format!("table_dt_{id}"))
            .selected_text(DATA_TYPES[self.base].name)
            .show_ui(ui, |ui| {
                for (i, dt) in DATA_TYPES.iter().enumerate() {
                    if ui
                        .selectable_label(self.base == i, dt.name)
                        .clicked()
                    {
                        self.base = i;
                        self.params = vec![0; dt.param_count];
                    }
                }
            });
        for param in &mut self.params {
            ui.add(egui::DragValue::new(param).speed(1));
        }
    }
}

#[typetag::serde]
impl FieldType for Domain {
    fn draw(&mut self, ui: &mut Ui, _id: usize) {
        ui.horizontal(|ui| {
            ui.label(&self.name);
            for param in &mut self.data_type.params {
                ui.add(egui::DragValue::new(param).speed(1));
            }
        });
    }
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: "id".to_string(),
            field_type: Box::new(DataType {
                base: 0,
                params: vec![0],
            }),
            nullable: false,
            primary_key: false,
        }
    }
}
