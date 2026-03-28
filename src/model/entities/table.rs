use slotmap::SlotMap;
use crate::model::field::{Attribute, FieldId};

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table rows
    attributes: SlotMap<FieldId, Attribute>,
    
    pk: 
}


impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Attribute::default());
    }

    pub fn new_fk(&mut self) {
        let field = Attribute::default_fk();
        self.attributes.insert(field);
    }

    pub fn add_field(&mut self, field: Attribute) {
        self.attributes.insert(field);
    }

    pub fn remove_field(&mut self, id: FieldId) {
        if let None =self.attributes.remove(id) {
            panic!("ID not found")
        }
    }

    pub fn fields(&self) -> &SlotMap<FieldId, Attribute> {
        &self.attributes
    }

    pub fn fields_mut(&mut self) -> &mut SlotMap<FieldId, Attribute> {
        &mut self.attributes
    }

    pub fn pks(&self) -> impl Iterator<Item = (FieldId, &Attribute)> {
        self.attributes
            .iter()
            .filter(|(_, field)| field.pk())
    }

    pub fn pks_mut(&mut self) -> impl Iterator<Item = (FieldId, &mut Attribute)> {
        self.attributes
            .iter_mut()
            .filter(|(_, field)| field.pk())
    }

    /// Adds field to primary key, automatically tries to remove it from attributes
    pub fn add_to_pk(&mut self, field_id: FieldId) {
        if let Some(attr) = self.fields_mut().get_mut(field_id) {
            attr.set_pk(true);
        }
    }

    pub fn remove_from_pk(&mut self, field_id: FieldId) {
        if let Some(attr) = self.attributes.get_mut(field_id) {
            attr.set_pk(false);
        }
    }

    /*pub fn remove_fk_from_pk(&mut self, field_id: FieldId) {
        if let Some(field) = self.pk.remove(field_id) {
            self.fks.insert(field);
        }
    }*/
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            attributes: SlotMap::with_key(),
        }
    }
}
