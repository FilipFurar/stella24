use crate::AppStella;
use crate::app::{DomainId, MAX_HISTORY_STATES, TableId, UndoItem};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::check::Check;
use crate::model::constraints::constraint::{FkId, ForeignKey, Unique};
use crate::model::datatype::DataType;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;

/// Possible commands to dispatch as enum variants
#[derive(Debug, Clone)]
pub enum Command {
    // App/file lifecycle
    NewCanvas,
    Undo,
    Redo,

    // Table lifecycle
    CreateTable {
        title: String,
    },
    DeleteTable {
        table: TableId,
    },
    RenameTable {
        table: TableId,
        title: String,
    },

    // Domain lifecycle
    CreateDomain {
        name: String,
        data_type: DataType,
    },
    DeleteDomain {
        domain: DomainId,
    },
    RenameDomain {
        domain: DomainId,
        name: String,
    },
    SetDomainType {
        domain: DomainId,
        data_type: DataType,
    },

    // Attribute lifecycle
    AddAttribute {
        table: TableId,
        attribute: Attribute,
    },
    DeleteAttribute {
        table: TableId,
        attr: AttrId,
    },
    RenameAttribute {
        table: TableId,
        attr: AttrId,
        name: String,
    },
    ReorderAttributes {
        table: TableId,
        from_index: usize,
        to_index: usize,
    },
    SetAttributeType {
        table: TableId,
        attr: AttrId,
        attribute_type: AttributeType,
    },
    SetAttributeNotNull {
        table: TableId,
        attr: AttrId,
        value: bool,
    },
    SetAttributeUnique {
        table: TableId,
        attr: AttrId,
        value: bool,
    },
    SetAttributePrimaryKey {
        table: TableId,
        attr: AttrId,
        value: bool,
    },

    // Primary key
    RenamePrimaryKey {
        table: TableId,
        name: String,
    },

    // Foreign keys
    AddForeignKey {
        table: TableId,
        foreign_key: ForeignKey,
    },
    DeleteForeignKey {
        table: TableId,
        fk: FkId,
    },
    SetForeignKeyReference {
        table: TableId,
        fk: FkId,
        references: Option<TableId>,
    },

    // Unique constraints
    AddUnique {
        table: TableId,
        unique: Unique,
    },
    DeleteUnique {
        table: TableId,
        index: usize,
    },
    RenameUnique {
        table: TableId,
        index: usize,
        name: String,
    },
    AddUniqueAttribute {
        table: TableId,
        index: usize,
        attr: AttrId,
    },
    RemoveUniqueAttribute {
        table: TableId,
        index: usize,
        attr: AttrId,
    },

    // Check constraints
    AddTableCheck {
        table: TableId,
        check: Check,
    },
    DeleteTableCheck {
        table: TableId,
        index: usize,
    },
    AddDomainCheck {
        domain: DomainId,
        check: Check,
    },
    DeleteDomainCheck {
        domain: DomainId,
        index: usize,
    },
}

/// Queue for command batching per frame.
#[derive(Default, Debug, Clone)]
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

impl AppStella {
    /// Enqueues a command to be applied after the current UI frame.
    pub fn dispatch(&mut self, cmd: Command) {
        self.command_queue.push(cmd);
    }

    /// Applies all queued commands in FIFO order.
    pub fn flush_commands(&mut self) {
        if self.command_queue.is_empty() {
            return;
        }

        let commands: Vec<Command> = self.command_queue.drain().collect();
        for cmd in commands {
            self.execute(cmd);
        }
    }

    pub fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::Undo => {
                if let Some(item) = self.undo_history.pop() {
                    match item {
                        UndoItem::Inverse(inv_cmd) => {
                            if let Some(redo_item) = self.apply_inverse_without_recording(inv_cmd) {
                                self.redo_history.push(redo_item);
                            }
                        }
                        UndoItem::Snapshot(prev_state) => {
                            self.redo_history.push(UndoItem::Snapshot(Box::from(
                                self.clone_without_histories(),
                            )));
                            self.restore_snapshot(*prev_state);
                        }
                    }
                }
            }
            Command::Redo => {
                if let Some(item) = self.redo_history.pop() {
                    match item {
                        UndoItem::Inverse(cmd_to_apply) => {
                            if let Some(undo_item) =
                                self.apply_inverse_without_recording(cmd_to_apply)
                            {
                                self.undo_history.push(undo_item);
                            }
                        }
                        UndoItem::Snapshot(prev_state) => {
                            self.undo_history.push(UndoItem::Snapshot(Box::from(
                                self.clone_without_histories(),
                            )));
                            self.restore_snapshot(*prev_state);
                        }
                    }
                }
            }
            other => self.apply_command_with_history(other),
        }
    }

    fn apply_command_with_history(&mut self, cmd: Command) {
        self.redo_history.clear();
        if let Some(pre_inv) = self.compute_inverse_pre(&cmd) {
            self.apply_command(cmd);
            self.push_undo_item(UndoItem::Inverse(pre_inv));
        } else {
            let snapshot = self.clone_without_histories();
            self.apply_command(cmd);
            self.push_undo_item(UndoItem::Snapshot(Box::from(snapshot)));
        }
    }

    pub fn execute_command(&mut self, cmd: Command) {
        self.apply_command(cmd);
    }

    fn apply_command(&mut self, cmd: Command) {
        match cmd {
            Command::NewCanvas => {
                self.tables.clear();
                self.domains.clear();
                self.domain_order.clear();
                self.workbench_table_layout.clear();
                self.workbench_table_rects.clear();
                self.workbench_pan = egui::Vec2::ZERO;
                self.workbench_zoom = 1.0;
            }
            Command::CreateTable { title } => {
                self.tables.insert(Table {
                    title,
                    ..Default::default()
                });
            }
            Command::DeleteTable { table } => {
                self.tables.remove(table);
            }
            Command::RenameTable { table, title } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.title = title;
                }
            }
            Command::AddAttribute { table, attribute } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.add_field(attribute);
                }
            }
            Command::DeleteAttribute { table, attr } => {
                if let Some(t) = self.tables.get_mut(table) {
                    t.attributes.remove(attr);
                    t.pk.attributes.remove(&attr);
                }
            }
            Command::RenameAttribute { table, attr, name } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.name = name;
                }
            }
            Command::SetAttributeType {
                table,
                attr,
                attribute_type,
            } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.attribute_type = match attribute_type {
                        AttributeType::Logical(mut dt) => {
                            dt.normalize_params();
                            AttributeType::Logical(dt)
                        }
                        other => other,
                    };
                }
            }
            Command::SetAttributeNotNull { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.not_null = if a.pk { true } else { value };
                }
            }
            Command::SetAttributeUnique { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table)
                    && let Some(a) = t.attributes.get_mut(attr)
                {
                    a.unique = value;
                }
            }
            Command::SetAttributePrimaryKey { table, attr, value } => {
                if let Some(t) = self.tables.get_mut(table) {
                    if value {
                        t.pk.attributes.insert(attr);
                    } else {
                        t.pk.attributes.remove(&attr);
                    }

                    if let Some(a) = t.attributes.get_mut(attr) {
                        a.pk = value;
                        if value {
                            a.not_null = true;
                        }
                    }
                }
            }
            Command::CreateDomain { name, data_type } => {
                let mut data_type = data_type;
                data_type.normalize_params();
                let id = self.domains.insert(Domain {
                    name,
                    data_type,
                    check_constraints: vec![],
                });
                self.domain_order.push(id);
            }
            Command::DeleteDomain { domain } => {
                self.domains.remove(domain);
                self.domain_order.retain(|id| *id != domain);
            }
            Command::RenameDomain { domain, name } => {
                if let Some(d) = self.domains.get_mut(domain) {
                    d.name = name;
                }
            }
            Command::SetDomainType { domain, data_type } => {
                if let Some(d) = self.domains.get_mut(domain) {
                    let mut data_type = data_type;
                    data_type.normalize_params();
                    d.data_type = data_type;
                }
            }
            Command::AddForeignKey { table, foreign_key } => {
                if let Some(referenced_table) = foreign_key.references
                    && let Some(pk_snapshot) = self.tables.get(referenced_table).map(|referenced| {
                        referenced
                            .pk
                            .attributes
                            .iter()
                            .filter_map(|attr_id| {
                                referenced
                                    .attributes
                                    .get(*attr_id)
                                    .map(|attr| (*attr_id, attr.name.clone()))
                            })
                            .collect::<Vec<_>>()
                    })
                    && let Some(current_table) = self.tables.get_mut(table)
                {
                    let mut fk = foreign_key;
                    fk.local_attrs.clear();

                    for (other_attr_id, other_attr_name) in pk_snapshot {
                        let local_attr = Attribute {
                            name: format!("{}_{}", fk.name, other_attr_name),
                            attribute_type: AttributeType::ForeignKeyAttribute(other_attr_id),
                            pk: false,
                            not_null: false,
                            unique: false,
                        };
                        let local_attr_key = current_table.attributes.insert(local_attr);
                        fk.local_attrs.insert(local_attr_key);
                    }

                    current_table.add_foreign_key(fk);
                }
            }
            Command::DeleteForeignKey { table, fk } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_foreign_key(fk);
                }
            }
            Command::SetForeignKeyReference {
                table,
                fk,
                references,
            } => {
                if let Some(current_table) = self.tables.get_mut(table)
                    && let Some(foreign_key) = current_table.fks.get_mut(fk)
                {
                    foreign_key.references = references;
                }
            }
            Command::AddUnique { table, unique } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_unique(unique);
                }
            }
            Command::DeleteUnique { table, index } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_unique(index);
                }
            }
            Command::RenameUnique { table, index, name } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.rename_unique(index, name);
                }
            }
            Command::AddUniqueAttribute { table, index, attr } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_unique_attribute(index, attr);
                }
            }
            Command::RemoveUniqueAttribute { table, index, attr } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_unique_attribute(index, attr);
                }
            }
            Command::AddTableCheck { table, check } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.add_check(check);
                }
            }
            Command::DeleteTableCheck { table, index } => {
                if let Some(current_table) = self.tables.get_mut(table) {
                    current_table.remove_check(index);
                }
            }
            Command::AddDomainCheck { domain, check } => {
                if let Some(current_domain) = self.domains.get_mut(domain) {
                    current_domain.check_constraints.push(check);
                }
            }
            Command::DeleteDomainCheck { domain, index } => {
                if let Some(current_domain) = self.domains.get_mut(domain)
                    && index < current_domain.check_constraints.len()
                {
                    current_domain.check_constraints.remove(index);
                }
            }
            _ => {}
        }
    }

    fn clone_without_histories(&self) -> Self {
        AppStella {
            tables: self.tables.clone(),
            domains: self.domains.clone(),
            domain_order: self.domain_order.clone(),
            settings: Default::default(),
            preferences: Default::default(),
            workbench_table_layout: self.workbench_table_layout.clone(),
            command_queue: CommandQueue::default(),
            modals: Default::default(),
            workbench_table_rects: self.workbench_table_rects.clone(),
            workbench_pan: self.workbench_pan,
            workbench_zoom: self.workbench_zoom,
            dragged_domain: self.dragged_domain,
            dragged_domain_from_index: self.dragged_domain_from_index,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: AppStella) {
        self.tables = snapshot.tables;
        self.domains = snapshot.domains;
        self.domain_order = snapshot.domain_order;
        self.workbench_table_layout = snapshot.workbench_table_layout;
        self.workbench_table_rects = snapshot.workbench_table_rects;
        self.workbench_pan = snapshot.workbench_pan;
        self.workbench_zoom = snapshot.workbench_zoom;
        self.settings = snapshot.settings;
        self.preferences = snapshot.preferences;
        self.dragged_domain = None;
        self.dragged_domain_from_index = None;
    }

    fn push_undo_item(&mut self, item: UndoItem) {
        self.undo_history.push(item);
        if self.undo_history.len() > MAX_HISTORY_STATES {
            self.undo_history.remove(0);
        }
    }

    /// Applies a command without recording to undo_history. Returns an UndoItem that
    /// represents the inverse of the command applied (suitable for pushing to the other stack).
    fn apply_inverse_without_recording(&mut self, cmd: Command) -> Option<UndoItem> {
        if let Some(pre_inv) = self.compute_inverse_pre(&cmd) {
            self.apply_command(cmd);
            Some(UndoItem::Inverse(pre_inv))
        } else {
            let snapshot = self.clone_without_histories();
            self.apply_command(cmd);
            Some(UndoItem::Snapshot(Box::from(snapshot)))
        }
    }

    /// Compute an inverse command from the current state for commands that can be inverted
    /// cheaply. Returns None when snapshot fallback should be used.
    fn compute_inverse_pre(&self, cmd: &Command) -> Option<Command> {
        use Command::*;
        match cmd {
            RenameTable { table, .. } => self.tables.get(*table).map(|t| RenameTable {
                table: *table,
                title: t.title.clone(),
            }),
            RenameAttribute { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| RenameAttribute {
                        table: *table,
                        attr: *attr,
                        name: a.name.clone(),
                    })
                } else {
                    None
                }
            }
            SetAttributeType { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeType {
                        table: *table,
                        attr: *attr,
                        attribute_type: a.attribute_type.clone(),
                    })
                } else {
                    None
                }
            }
            SetAttributeNotNull { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeNotNull {
                        table: *table,
                        attr: *attr,
                        value: a.not_null,
                    })
                } else {
                    None
                }
            }
            SetAttributeUnique { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributeUnique {
                        table: *table,
                        attr: *attr,
                        value: a.unique,
                    })
                } else {
                    None
                }
            }
            SetAttributePrimaryKey { table, attr, .. } => {
                if let Some(t) = self.tables.get(*table) {
                    t.attributes.get(*attr).map(|a| SetAttributePrimaryKey {
                        table: *table,
                        attr: *attr,
                        value: a.pk,
                    })
                } else {
                    None
                }
            }
            RenamePrimaryKey { table, .. } => self.tables.get(*table).map(|t| RenamePrimaryKey {
                table: *table,
                name: t.pk.name.clone(),
            }),
            RenameDomain { domain, .. } => self.domains.get(*domain).map(|d| RenameDomain {
                domain: *domain,
                name: d.name.clone(),
            }),
            SetDomainType { domain, .. } => self.domains.get(*domain).map(|d| SetDomainType {
                domain: *domain,
                data_type: d.data_type.clone(),
            }),
            SetForeignKeyReference { table, fk, .. } => self
                .tables
                .get(*table)
                .and_then(|t| t.fks.get(*fk))
                .map(|f| SetForeignKeyReference {
                    table: *table,
                    fk: *fk,
                    references: f.references,
                }),
            RenameUnique { table, index, .. } => self
                .tables
                .get(*table)
                .and_then(|t| t.uniques.get(*index))
                .map(|u| RenameUnique {
                    table: *table,
                    index: *index,
                    name: u.name.clone(),
                }),
            AddUniqueAttribute { table, index, attr } => Some(RemoveUniqueAttribute {
                table: *table,
                index: *index,
                attr: *attr,
            }),
            RemoveUniqueAttribute { table, index, attr } => Some(AddUniqueAttribute {
                table: *table,
                index: *index,
                attr: *attr,
            }),
            _ => None,
        }
    }
}
