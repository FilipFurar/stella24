use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, AttributeType};
use crate::model::datatype::DataType;
/*
#[derive(Clone, Debug)]
pub enum Command {
    // Table lifecycle
    CreateTable { title: String },
    DeleteTable(TableId),
    RenameTable { id: TableId, new_title: String },

    // Attribute lifecycle
    CreateAttribute {
        table: TableId,
        name: String,
        attr_type: AttributeType
    },
    DeleteAttribute { table: TableId, id: AttrId },
    RenameAttribute { table: TableId, id: AttrId, name: String },

    // Type changes (the popup interactions)
    SetAttributeType {
        table: TableId,
        attr: AttrId,
        new_type: AttributeType
    },

    // PK management
    AddToPrimaryKey { table: TableId, attr: AttrId },
    RemoveFromPrimaryKey { table: TableId, attr: AttrId },
    RenamePrimaryKey { table: TableId, name: String },

    // FK management
    CreateForeignKey {
        table: TableId,
        name: String,
        attr: AttrId,  // The FK column
        references: TableId
    },
    DeleteForeignKey { table: TableId, id: usize }, // or FkId
    SetFkReference { table: TableId, id: usize, references: Option<TableId> },

    // Domain management
    CreateDomain { name: String, base_type: DataType },
    DeleteDomain(DomainId),
}

// For undo/redo support later
#[derive(Default)]
pub struct CommandHistory {
    pub(crate) undo_stack: Vec<Command>,
    pub(crate) redo_stack: Vec<Command>,
}*/
