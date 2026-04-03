// model/constraints/constraints.rs
use crate::app::TableId;
use crate::model::attribute::AttrId;
use slotmap::SlotMap;
use std::collections::HashSet;

slotmap::new_key_type! {
    pub struct FkId;
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PrimaryKey {
    pub name: String,
    pub attributes: HashSet<AttrId>,
}

/// ForeignKey is the whole foreign key constraint
/// It references a single table (contains TableId of the table that we reference)
/// and it has a collection (hashset) of the local attributes that will correspond to the attributes from the other table

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ForeignKey {
    pub name: String,
    /// The table being referenced
    pub references: Option<TableId>,
    /// HashSet of IDs of local attributes from FK - corresponding to PK attributes from the other table
    pub local_attrs: HashSet<AttrId>,
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

impl PrimaryKey {
    pub fn new() -> Self {
        Self {
            name: "primary_key".to_string(),
            attributes: HashSet::new(),
        }
    }
}

impl Default for PrimaryKey {
    fn default() -> Self {
        Self::new()
    }
}

impl ForeignKey {
    pub fn new() -> Self {
        Self {
            name: "ref_".to_string(),
            references: None,
            local_attrs: Default::default(),
        }
    }
}

impl Default for ForeignKey {
    fn default() -> Self {
        Self::new()
    }
}
