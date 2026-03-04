use super::datatype::DataType;
use crate::app::DomainId;

/// Field is one row in your table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Type of Field
    pub field_type: FieldType,
    /// Can be NULL?
    pub nullable: bool,
    /// Is it a primary key?
    pub primary_key: bool,
}

impl Field {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    pub fn field_type_mut(&mut self) -> &mut FieldType {
        &mut self.field_type
    }

    pub fn nullable(&self) -> bool {
        self.nullable
    }

    pub fn primary_key(&self) -> bool {
        self.primary_key
    }
}

impl Field {
    pub fn set_null(&mut self, value: bool) {
        self.nullable = value;
    }
    pub fn set_primary_key(&mut self, value: bool) {
        self.primary_key = value;
    }
}

/// The possible row options
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FieldType {
    /// Built-in data type
    Data(DataType),
    /// Domain type
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
