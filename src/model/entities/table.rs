use slotmap::SlotMap;
use crate::model::constraints::foreign_key::ForeignKey;
use crate::model::field::{Field, FieldId, FieldType};

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table rows
    attributes: SlotMap<FieldId, Field>,

    /// Primary key
    pk: SlotMap<FieldId, Field>,

    /// Foreign keys
    fks: SlotMap<FieldId, Field>,
}


impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Field::default());
    }

    pub fn new_fk(&mut self) {
        let field = Field {
            name: "".to_string(),
            field_type: FieldType::ForeignKey(ForeignKey::default()),
            nullable: false,
        };
        self.fks.insert(field);
    }

    pub fn add_field(&mut self, field: Field) {
        self.attributes.insert(field);
    }

    pub fn remove_field(&mut self, id: FieldId) {
        if let None =self.attributes.remove(id) && let None = self.fks.remove(id) && let None = self.pk.remove(id) {
            panic!("ID not found")
        }
    }

    pub fn remove_fk(&mut self, id: FieldId) {
        self.fks.remove(id);
    }

    pub fn fields(&self) -> &SlotMap<FieldId, Field> {
        &self.attributes
    }

    pub fn fields_mut(&mut self) -> &mut SlotMap<FieldId, Field> {
        &mut self.attributes
    }

    pub fn pk(&self) -> &SlotMap<FieldId, Field> { &self.pk }

    pub fn pk_mut(&mut self) -> &mut SlotMap<FieldId, Field> { &mut self.pk }

    /// Adds field to primary key, automatically tries to remove it from attributes
    pub fn add_to_pk(&mut self, field_id: FieldId) {
        if let Some(removed_field) = self.attributes.remove(field_id) {
            self.pk.insert(removed_field);
        } else if let Some(removed_field) = self.fks.remove(field_id) {
            self.pk.insert(removed_field);
        } else {
            panic!("field not found");
        }
    }

    pub fn remove_from_pk(&mut self, field_id: FieldId) {
        if let Some(removed_field) = self.pk.remove(field_id) {
            match removed_field.field_type {
                FieldType::Data(_) => {self.attributes.insert(removed_field);}
                FieldType::Domain(_) => {self.attributes.insert(removed_field);}
                FieldType::ForeignKey(_) => {self.fks.insert(removed_field);}
            }
        } else {
            panic!("field not found");
        }
    }


    pub fn fk_to_pk(&mut self, field: Field) {
        self.pk.insert(field);
    }

    pub fn fk_id_to_pk(&mut self, field_id: FieldId) {
        if let Some(field) = self.fks.remove(field_id) {
            self.pk.insert(field);
        } else if let Some(field) = self.pk.remove(field_id) {
            self.fks.insert(field);
        } else {
            panic!("pozor");
        }
    }

    /*pub fn remove_fk_from_pk(&mut self, field_id: FieldId) {
        if let Some(field) = self.pk.remove(field_id) {
            self.fks.insert(field);
        }
    }*/

    pub fn fks(&self) -> &SlotMap<FieldId, Field> {
        &self.fks
    }

    pub fn fks_mut(&mut self) -> &mut SlotMap<FieldId, Field> {
        &mut self.fks
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            attributes: SlotMap::with_key(),
            pk: SlotMap::with_key(),
            fks: SlotMap::with_key(),
        }
    }
}
