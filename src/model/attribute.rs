// model/attribute

use crate::app::DomainId;
use crate::model::datatype::DataType;

slotmap::new_key_type! {
    /// Unique type for FieldId keys
    pub struct AttrId;
}

/// Field is one row in your table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Attribute {
    /// Field name
    pub name: String,

    /// Type of Field
    pub attribute_type: AttributeType,

    pub pk: bool,

    pub nullable: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeCategory {
    Logical,
    Domain,
    ForeignKey,
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_type(&self) -> &AttributeType {
        &self.attribute_type
    }

    pub fn attribute_type_mut(&mut self) -> &mut AttributeType {
        &mut self.attribute_type
    }

    pub fn set_type(&mut self, new_type: AttributeType) {
        self.attribute_type = new_type;
    }
}

impl Attribute {
    pub fn default_primary_key() -> Self {
        Self {
            name: "id".to_string(),
            attribute_type: AttributeType::Logical(DataType {
                base: 3,
                params: vec![1, 0],
            }),
            pk: true,
            nullable: false,
        }
    }
}

/// The possible column options
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AttributeType {
    /// Built-in data type
    Logical(DataType),
    /// Domain type
    Domain(DomainId),
    /// Foreign key, this is a single attribute, that is a part of a foreign key constraint
    /// AttrId - the ID of the attribute that this FK attribute corresponds to
    ForeignKeyAttribute(AttrId),
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            name: "id".to_string(),
            attribute_type: AttributeType::Logical(DataType {
                base: 0,
                params: vec![1],
            }),
            pk: false,
            nullable: true,
        }
    }
}
