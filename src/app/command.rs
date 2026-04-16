use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::{FkId, ForeignKey, Unique};
use crate::model::datatype::DataType;

/// Intent-based state changes.
///
/// Keep this enum small and explicit: UI emits intents, `AppStella` executes them.
#[derive(Debug)]
pub enum Command {
    // App/file lifecycle
    NewCanvas,

    // Table lifecycle
    CreateTable { title: String },
    DeleteTable { table: TableId },
    RenameTable { table: TableId, title: String },

    // Domain lifecycle
    CreateDomain { name: String, data_type: DataType },
    DeleteDomain { domain: DomainId },
    RenameDomain { domain: DomainId, name: String },
    SetDomainType { domain: DomainId, data_type: DataType },

    // Attribute lifecycle
    AddAttribute { table: TableId, attribute: Attribute },
    DeleteAttribute { table: TableId, attr: AttrId },
    RenameAttribute { table: TableId, attr: AttrId, name: String },
    SetAttributeType {
        table: TableId,
        attr: AttrId,
        attribute_type: AttributeType,
    },
    SetAttributeNotNull { table: TableId, attr: AttrId, value: bool },
    SetAttributeUnique { table: TableId, attr: AttrId, value: bool },
    SetAttributePrimaryKey { table: TableId, attr: AttrId, value: bool },

    // Primary key
    RenamePrimaryKey { table: TableId, name: String },

    // Foreign keys
    AddForeignKey { table: TableId, foreign_key: ForeignKey },
    DeleteForeignKey { table: TableId, fk: FkId },
    SetForeignKeyReference {
        table: TableId,
        fk: FkId,
        references: Option<TableId>,
    },

    // Unique constraints
    AddUnique { table: TableId, unique: Unique },
    DeleteUnique { table: TableId, index: usize },
    RenameUnique { table: TableId, index: usize, name: String },
    AddUniqueAttribute { table: TableId, index: usize, attr: AttrId },
    RemoveUniqueAttribute { table: TableId, index: usize, attr: AttrId },
}

/// Minimal FIFO queue for command batching per frame.
#[derive(Default, Debug)]
pub struct CommandQueue {
    pending: Vec<Command>,
}

impl CommandQueue {
    pub fn push(&mut self, command: Command) {
        self.pending.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Drains pending commands in insertion order.
    pub fn drain(&mut self) -> impl Iterator<Item = Command> + '_ {
        self.pending.drain(..)
    }
}
