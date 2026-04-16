// model/entities/table.rs
use crate::model::attribute::{AttrId, Attribute};
use crate::model::constraints::constraint::{FkId, ForeignKey, NotNull, PrimaryKey, Unique};
use slotmap::SlotMap;

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table columns, contain "physical" attributes
    pub attributes: SlotMap<AttrId, Attribute>,

    /// Primary key constraint, contains only Ids of attributes that the PK is made out of
    pub pk: PrimaryKey,

    /// Stores ForeignKey constraints
    pub fks: SlotMap<FkId, ForeignKey>,

    /// Vector of Unique constraints, does not contain the values itself, only I
    pub uniques: Vec<Unique>,

    pub not_nulls: Vec<NotNull>,

    #[serde(skip)]
    pub open_modal: bool,

    #[serde(skip)]
    pub current_fk: Option<ForeignKey>,
}

impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Attribute::default());
    }

    pub fn change_fk(&mut self, fk_id: FkId, foreign_key: ForeignKey) {
        self.fks.remove(fk_id).expect("ERR");
        self.fks.insert(foreign_key);
    }

    pub fn remove_fk(&mut self) {}

    pub fn add_field(&mut self, field: Attribute) {
        self.attributes.insert(field);
    }

    pub fn remove_field(&mut self, id: AttrId) {
        if self.attributes.remove(id).is_none() {
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
            open_modal: false,
            current_fk: None,
        }
    }
}
