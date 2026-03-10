use slotmap::SlotMap;
use crate::app::{FieldId, TableId};
use super::constraints::field::Field;

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
    fks: Vec<TableId>,
}

impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Field::default());
    }

    pub fn add_field(&mut self, field: Field) {
        self.attributes.insert(field);
    }

    pub fn remove_field(&mut self, id: FieldId) {
        if let None =self.attributes.remove(id) {
            self.pk.remove(id);
        }

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
        if self.pk.contains_key(field_id) {
            panic!("id already in use");
        }
        if let Some(removed_field) = self.attributes.remove(field_id) {
            self.pk.insert(removed_field);
        } else {
            panic!("field not found");
        }
    }

    pub fn remove_from_pk(&mut self, field_id: FieldId) {
        if self.attributes.contains_key(field_id) {
            panic!("id already in use");
        }

        if let Some(removed_field) = self.pk.remove(field_id) {
            self.attributes.insert(removed_field);
        } else {
            panic!("field not found");
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            attributes: SlotMap::with_key(),
            pk: SlotMap::with_key(),
            fks: vec![],
        }
    }
}
