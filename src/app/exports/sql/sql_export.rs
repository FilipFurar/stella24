//! Shared SQL export abstractions and dispatcher.

use crate::app::{DomainId, TableId};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::SlotMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub enum SqlDialect {
    #[default]
    Oracle,
    MySql,
    PostgreSql,
    Sqlite,
}

impl SqlDialect {
    pub const ALL: [SqlDialect; 4] = [
        SqlDialect::Oracle,
        SqlDialect::MySql,
        SqlDialect::PostgreSql,
        SqlDialect::Sqlite,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            SqlDialect::Oracle => "Oracle",
            SqlDialect::MySql => "MySQL",
            SqlDialect::PostgreSql => "PostgreSQL",
            SqlDialect::Sqlite => "SQLite",
        }
    }
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub trait SqlExport {
    fn dialect(&self) -> SqlDialect;
    fn build_sql(
        &self,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlExportError {
    EmptyName {
        kind: &'static str,
    },
    DuplicateTableName {
        name: String,
    },
    DuplicateDomainName {
        name: String,
    },
    DuplicateColumnName {
        table: String,
        name: String,
    },
    DuplicateConstraintName {
        name: String,
    },
    EmptyCheckCondition {
        context: String,
    },
    MissingReferencedTable {
        table: String,
        foreign_key: String,
    },
    MissingReferencedColumn {
        table: String,
        foreign_key: String,
        referenced_table: String,
    },
    MissingForeignKeyConstraint {
        table: String,
        column: String,
    },
    AmbiguousForeignKey {
        table: String,
        column: String,
    },
    UnsupportedDataType {
        context: String,
        base: usize,
    },
    DialectNotImplemented {
        dialect: SqlDialect,
    },
    IdentifierTooLong {
        kind: &'static str,
        name: String,
    },
}

impl fmt::Display for SqlExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlExportError::EmptyName { kind } => write!(f, "empty {kind} name"),
            SqlExportError::DuplicateTableName { name } => {
                write!(f, "duplicate table name: {name}")
            }
            SqlExportError::DuplicateDomainName { name } => {
                write!(f, "duplicate domain name: {name}")
            }
            SqlExportError::DuplicateColumnName { table, name } => {
                write!(f, "duplicate column name {name} in table {table}")
            }
            SqlExportError::DuplicateConstraintName { name } => {
                write!(f, "duplicate constraint name: {name}")
            }
            SqlExportError::EmptyCheckCondition { context } => {
                write!(f, "empty CHECK condition ({context})")
            }
            SqlExportError::MissingReferencedTable { table, foreign_key } => write!(
                f,
                "foreign key {foreign_key} in table {table} references a missing table"
            ),
            SqlExportError::MissingReferencedColumn {
                table,
                foreign_key,
                referenced_table,
            } => write!(
                f,
                "foreign key {foreign_key} in table {table} points to a missing column in {referenced_table}"
            ),
            SqlExportError::MissingForeignKeyConstraint { table, column } => write!(
                f,
                "column {column} in table {table} is typed as FK but has no matching FK constraint"
            ),
            SqlExportError::AmbiguousForeignKey { table, column } => write!(
                f,
                "column {column} in table {table} belongs to multiple FK constraints"
            ),
            SqlExportError::UnsupportedDataType { context, base } => {
                write!(f, "unsupported datatype base {base} ({context})")
            }
            SqlExportError::DialectNotImplemented { dialect } => {
                write!(f, "SQL dialect not implemented: {dialect}")
            }
            SqlExportError::IdentifierTooLong { kind, name } => {
                write!(f, "{kind} identifier too long: {name}")
            }
        }
    }
}
impl std::error::Error for SqlExportError {}

pub fn build_sql(
    dialect: SqlDialect,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
) -> Result<String, SqlExportError> {
    match dialect {
        SqlDialect::Oracle => {
            crate::app::exports::sql::oracle::OracleDialect.build_sql(tables, domains)
        }
        SqlDialect::Sqlite => {
            crate::app::exports::sql::sqlite::SqliteDialect.build_sql(tables, domains)
        }
        _ => Err(SqlExportError::DialectNotImplemented { dialect }),
    }
}
