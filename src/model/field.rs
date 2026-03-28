use crate::model::datatype::DataType;
use crate::app::DomainId;
use crate::model::constraints::foreign_key::ForeignKey;

slotmap::new_key_type! {
    /// Unique type for FieldId keys
    pub struct FieldId;
}

/// Field is one row in your table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Attribute {
    /// Field name
    pub name: String,
    /// Type of Field
    field_type: AttributeType, 
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_type(&self) -> &AttributeType {
        &self.field_type
    }

    pub fn field_type_mut(&mut self) -> &mut AttributeType {
        &mut self.field_type
    }
    

    pub fn set_type(&mut self, new_type: AttributeType) {
        self.field_type = new_type;
    }

}

impl Attribute {
    pub fn default_primary_key() -> Self {
        Self {
            name: "id".to_string(),
            field_type: AttributeType::Data(DataType {
                base: 3,
                params: vec![1, 0],
            }),
        }
    }
}

/// The possible row options
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AttributeType {
    /// Built-in data type
    Data(DataType),
    /// Domain type
    Domain(DomainId),
}

impl AttributeType {
    /// Can this FieldType be NULL?
    pub fn is_nullable_supported(&self) -> bool {
        matches!(self, AttributeType::Data(_) | AttributeType::Domain(_))
    }
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            name: "id".to_string(),
            field_type: AttributeType::Data(DataType {
                base: 0,
                params: vec![1],
            }),
        }
    }
}
