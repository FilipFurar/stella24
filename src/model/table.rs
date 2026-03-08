use slotmap::SlotMap;
use crate::app::{FieldId, TableId};
use super::field::Field;

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table rows
    fields: SlotMap<FieldId, Field>,

    /// Primary key
    pk: SlotMap<FieldId, Field>,

    /// Foreign keys
    fks: Vec<TableId>,
}

impl Table {
    pub fn new_field(&mut self) {
        self.fields.insert(Field::default());
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.insert(field);
    }

    pub fn remove_field(&mut self, id: FieldId) {
        if let None =self.fields.remove(id) {
            self.pk.remove(id);
        }

    }

    pub fn fields(&self) -> &SlotMap<FieldId, Field> {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut SlotMap<FieldId, Field> {
        &mut self.fields
    }

    pub fn pk(&self) -> &SlotMap<FieldId, Field> { &self.pk }

    pub fn pk_mut(&mut self) -> &mut SlotMap<FieldId, Field> { &mut self.pk }


    pub fn add_to_pk(&mut self, field_id: FieldId) {
        if let Some(removed_field) = self.fields.remove(field_id) {
            self.pk.insert(removed_field);
        } else {
            panic!("field not found");
        }
    }

    pub fn remove_from_pk(&mut self, field_id: FieldId) {
        if let Some(removed_field) = self.pk.remove(field_id) {
            self.fields.insert(removed_field);
        } else {
            panic!("field not found");
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            fields: SlotMap::with_key(),
            pk: SlotMap::with_key(),
            fks: vec![],
        }
    }
}
