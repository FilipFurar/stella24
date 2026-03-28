use std::collections::HashSet;
use slotmap::SlotMap;
use crate::app::TableId;
use crate::model::field::AttrId;

slotmap::new_key_type! {
    /// Unique type for Primary Key keys
    pub struct PkId;
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PrimaryKey {
    pub name: String,
    pub attributes: HashSet<AttrId>,}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ForeignKey {
    name: String,
    references: Option<TableId>,
}

slotmap::new_key_type! {
    /// Unique type for Unique keys
    pub struct UniqueId;
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Unique {
    name: String,
    attributes: SlotMap<UniqueId, AttrId>,
}

slotmap::new_key_type! {
    /// Unique type for Not Null keys
    pub struct NotNullId;
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NotNull {
    name: String,
    attributes: SlotMap<NotNullId, AttrId>,
}

pub enum ConstraintType {
    PrimaryKey,
    ForeignKey,
    NotNull,
    Unique,
    Check,
}

impl PrimaryKey {
    pub fn new() -> Self {
        Self {
            name: "primary_key".to_string(),
            attributes: HashSet::new(),
        }
    }
}

impl ForeignKey {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            references: None,
        }
    }
}