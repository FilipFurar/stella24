use super::datatype::{DataType};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    Data(DataType),
    Domain(usize),
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: "id".to_string(),
            field_type: FieldType::Data(DataType {
                base: 0,
                params: vec![1],
            }),
            nullable: false,
            primary_key: false,
        }
    }
}
