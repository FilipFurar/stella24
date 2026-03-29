// model/constraints/constraints.rs
use std::collections::HashSet;
use slotmap::SlotMap;
use crate::app::TableId;
use crate::model::field::AttrId;

slotmap::new_key_type! {
    pub struct FkId;
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PrimaryKey {
    pub name: String,
    pub attributes: HashSet<AttrId>,}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ForeignKey {
    pub name: String,
    pub references: Option<TableId>,
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
            name: "ref_".to_string(),
            references: None,
        }
    }
}