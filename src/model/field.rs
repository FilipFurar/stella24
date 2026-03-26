use crate::model::datatype::DataType;
use crate::app::DomainId;
use crate::model::constraints::foreign_key::ForeignKey;

slotmap::new_key_type! {
    /// Unique type for FieldId keys
    pub struct FieldId;
}

/// Field is one row in your table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Field {
    /// Field name
    pub(crate) name: String,
    /// Type of Field
    field_type: FieldType,
    /// Can be NULL?
    nullable: bool,
    /// Is a primary key
    primary_key: bool,
}

impl Field {
    pub fn default_fk() -> Self {
        Self {
            name: "".to_string(),
            field_type: FieldType::ForeignKey(ForeignKey::default()),
            nullable: false,
            primary_key: false,
        }
    }
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
    
    pub fn set_pk(&mut self, val: bool) {
        self.primary_key = val;
    }

    pub fn pk(&self) -> bool {
        self.primary_key
    }

    pub fn fk(&self) -> bool {
        if let FieldType::ForeignKey(_fk) = &self.field_type {
            return true
        }
        false
    }

    pub fn set_type(&mut self, new_type: FieldType) {
        self.field_type = new_type;
    }

}

impl Field {
    pub fn default_primary_key() -> Self {
        Self {
            name: "id".to_string(),
            field_type: FieldType::Data(DataType {
                base: 3,
                params: vec![1, 0],
            }),
            nullable: false,
            primary_key: true,
        }
    }

    pub fn set_null(&mut self, value: bool) {
        self.nullable = value;
    }

}

/// The possible row options
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FieldType {
    /// Built-in data type
    Data(DataType),
    /// Domain type
    Domain(DomainId),
    /// Foreign key constraint
    ForeignKey(ForeignKey),
}

impl FieldType {
    /// Can this FieldType be NULL?
    pub fn is_nullable_supported(&self) -> bool {
        matches!(self, FieldType::Data(_) | FieldType::Domain(_))
    }
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
