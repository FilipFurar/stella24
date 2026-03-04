use super::datatype::DataType;
use crate::app::DomainId;

/// Field is one row in your table
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub primary_key: bool,
}

/// The possible row options
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    Data(DataType),
    Domain(DomainId),
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
