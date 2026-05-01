// model/entities/table.rs
use crate::model::attribute::{AttrId, Attribute};
use crate::model::constraints::check::Check;
use crate::model::constraints::constraint::{FkId, ForeignKey, NotNull, PrimaryKey, Unique};
use slotmap::SlotMap;

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,

    /// Table columns, contain "physical" attributes
    #[serde(default)]
    pub attributes: SlotMap<AttrId, Attribute>,

    /// Primary key constraint, contains only Ids of attributes that the PK is made out of
    #[serde(default)]
    pub pk: PrimaryKey,

    /// Stores ForeignKey constraints
    #[serde(default)]
    pub fks: SlotMap<FkId, ForeignKey>,

    /// Vector of Unique constraints, does not contain the values itself, only I
    #[serde(default)]
    pub uniques: Vec<Unique>,

    #[serde(default)]
    pub not_nulls: Vec<NotNull>,

    #[serde(default)]
    pub checks: Vec<Check>,

    #[serde(skip)]
    pub open_modal: bool,

    #[serde(skip)]
    pub current_fk: Option<ForeignKey>,

    #[serde(skip)]
    pub current_unique: Option<usize>,
}

impl Table {
    pub fn new_field(&mut self) {
        self.attributes.insert(Attribute::default());
    }

    pub fn change_fk(&mut self, fk_id: FkId, foreign_key: ForeignKey) {
        self.fks.remove(fk_id).expect("ERR");
        self.fks.insert(foreign_key);
    }

    pub fn add_foreign_key(&mut self, foreign_key: ForeignKey) -> FkId {
        self.fks.insert(foreign_key)
    }

    pub fn remove_foreign_key(&mut self, fk_id: FkId) -> Option<ForeignKey> {
        let fk = self.fks.remove(fk_id)?;
        for attid in &fk.local_attrs {
            self.attributes.remove(*attid);
        }
        Some(fk)
    }

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

    pub fn add_unique(&mut self, unique: Unique) -> usize {
        for attr_id in &unique.attributes {
            if let Some(attribute) = self.attributes.get_mut(*attr_id) {
                attribute.unique = true;
            }
        }

        self.uniques.push(unique);
        self.uniques.len().saturating_sub(1)
    }

    pub fn remove_unique(&mut self, index: usize) -> Option<Unique> {
        if index >= self.uniques.len() {
            return None;
        }

        let unique = self.uniques.remove(index);
        for attr_id in &unique.attributes {
            let still_unique = self.pk.attributes.contains(attr_id)
                || self
                    .uniques
                    .iter()
                    .any(|candidate| candidate.attributes.contains(attr_id));
            if !still_unique && let Some(attr) = self.attributes.get_mut(*attr_id) {
                attr.unique = false;
            }
        }

        Some(unique)
    }

    pub fn rename_unique(&mut self, index: usize, name: String) {
        if let Some(unique) = self.uniques.get_mut(index) {
            unique.name = name;
        }
    }

    pub fn add_unique_attribute(&mut self, index: usize, attr: AttrId) {
        if let Some(unique) = self.uniques.get_mut(index) {
            unique.attributes.insert(attr);
            if let Some(attribute) = self.attributes.get_mut(attr) {
                attribute.unique = true;
            }
        }
    }

    pub fn remove_unique_attribute(&mut self, index: usize, attr: AttrId) {
        if let Some(unique) = self.uniques.get_mut(index) {
            unique.attributes.remove(&attr);
        }

        let still_unique = self.pk.attributes.contains(&attr)
            || self
                .uniques
                .iter()
                .any(|candidate| candidate.attributes.contains(&attr));
        if !still_unique && let Some(attribute) = self.attributes.get_mut(attr) {
            attribute.unique = false;
        }
    }

    pub fn add_check(&mut self, check: Check) -> usize {
        self.checks.push(check);
        self.checks.len().saturating_sub(1)
    }

    pub fn remove_check(&mut self, index: usize) -> Option<Check> {
        if index < self.checks.len() {
            Some(self.checks.remove(index))
        } else {
            None
        }
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
            checks: vec![],
            open_modal: false,
            current_fk: None,
            current_unique: None,
        }
    }
}
