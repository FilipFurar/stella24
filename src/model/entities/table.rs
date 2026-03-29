// model/entities/table.rs
use slotmap::SlotMap;
use crate::model::constraints::constraint::{ForeignKey, NotNull, FkId, PrimaryKey, Unique};
use crate::model::field::{Attribute, AttrId};

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table rows
    pub attributes: SlotMap<AttrId, Attribute>,
    
    pub pk: PrimaryKey,

    pub fks: SlotMap<FkId, AttrId>,

    uniques: Vec<Unique>,

    not_nulls: Vec<NotNull>,
}


impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Attribute::default());
    }


    pub fn add_field(&mut self, field: Attribute) {
        self.attributes.insert(field);
    }

    pub fn remove_field(&mut self, id: AttrId) {
        if let None =self.attributes.remove(id) {
            panic!("ID not found")
        }
    }

    pub fn fields(&self) -> &SlotMap<AttrId, Attribute> {
        &self.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut SlotMap<AttrId, Attribute> {
        &mut self.attributes
    }


    pub fn change_pk(&mut self, pk: PrimaryKey) {
        self.pk = pk;
    }

    pub fn add_pk(&mut self, pk: AttrId) {
        self.pk.attributes.insert(pk);
        if let Some(pkey) = self.attributes.get_mut(pk) {
            pkey.pk = true;
        }
    }

    pub fn remove_pk(&mut self, pk_id: AttrId) -> bool {
        let p = self.pk.attributes.remove(&pk_id);
        if let Some(attr) = self.attributes.get_mut(pk_id) {
            attr.pk = false;
        }
        p
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            attributes: SlotMap::with_key(),
            pk: PrimaryKey::new(),
            fks: SlotMap::with_key(),
            uniques: vec![],
            not_nulls: vec![],
        }
    }
}
