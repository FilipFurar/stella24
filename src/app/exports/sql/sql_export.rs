//! Shared SQL export abstractions and dispatcher.

use std::collections::HashSet;
use crate::app::{DomainId, TableId};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use crate::model::constraints::check::Check;
use slotmap::{Key, SlotMap};
use std::fmt;
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::ForeignKey;

const IDENTIFIER_LIMIT: usize = 128;

/// SQL dialect selection used for SQL DDL generation.
///
/// Export implementations generate different DDL depending on the target
/// database engine. This enum selects the concrete exporter to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub enum SqlDialect {
    /// Oracle-compatible DDL (Oracle 23c style features).
    #[default]
    Oracle,
    /// SQLite-compatible DDL.
    Sqlite,
    /// PostgreSQL-compatible DDL.
    Postgres,
}

impl SqlDialect {
    /// Const array of all SqlDialect variants so we can iterate through them.
    pub const ALL: [SqlDialect; 3] = [
        SqlDialect::Oracle,
        SqlDialect::Sqlite,
        SqlDialect::Postgres,
    ];

    /// Human-readable label for the dialect used in UI controls.
    pub const fn label(self) -> &'static str {
        match self {
            SqlDialect::Oracle => "Oracle",
            SqlDialect::Sqlite => "SQLite",
            SqlDialect::Postgres => "PostgreSQL",
        }
    }
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Trait implemented by concrete SQL exporters (Oracle, SQLite, ...).
///
/// Implementors produce DDL text for a complete schema and provide a set of
/// helper functions used during rendering (column type resolution, per-table
/// rendering and constraint collection).
pub trait Export {
    /// Returns the dialect handled by this exporter.
    fn dialect(&self) -> SqlDialect;

    /// Builds a complete SQL script of the data model.
    ///
    /// The returned string should contain any required statements (domains,
    /// CREATE TABLE, ALTER TABLE for FKs, etc.) or an error if generation
    /// failed (invalid names, missing references, unsupported types).
    fn build_sql(
        &self,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError>;

    /// Resolve an attribute's SQL type representation for this dialect.
    ///
    /// Uses the attribute, table and domain information to produce a
    /// dialect-specific column type string (for example, "VARCHAR2(50)"
    /// on Oracle or "TEXT" on SQLite).
    fn attribute_type_sql(
        table: &Table,
        attr_id: AttrId,
        attr: &Attribute,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError>;

    /// Render a CREATE TABLE statement (without final trailing blank line).
    fn render_table(
        out: &mut String,
        table: &Table,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
        used_constraints: &mut HashSet<String>,
    ) -> Result<(), SqlExportError>;

    /// Render a single column definition used inside `CREATE TABLE`.
    fn render_column(
        table: &Table,
        attr_id: AttrId,
        attr: &Attribute,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
        used_constraints: &mut HashSet<String>,
    ) -> Result<String, SqlExportError>;

    /// Collects multi-column table-level constraints (PK, UQ, NN, CHECK).
    fn render_table_constraints(
        table: &Table,
        used_constraints: &mut HashSet<String>,
    ) -> Result<Vec<String>, SqlExportError> {
        let mut lines = Vec::new();

        let pk_attrs = sorted_attr_ids(table, &table.pk.attributes);
        if pk_attrs.len() > 1 {
            let name = constraint_name_or_fallback(&table.pk.name, &format!("PK_{}", table.title));
            add_constraint_name(used_constraints, &name)?;
            lines.push(format!(
                "CONSTRAINT {} PRIMARY KEY ({})",
                name,
                join_attr_names(table, &pk_attrs)
            ));
        }

        for (idx, unique) in table.uniques.iter().enumerate() {
            let attrs = sorted_attr_ids(table, &unique.attributes);
            if attrs.len() <= 1 {
                continue;
            }
            let name =
                constraint_name_or_fallback(&unique.name, &format!("UQ_{}_{}", table.title, idx + 1));
            add_constraint_name(used_constraints, &name)?;
            lines.push(format!(
                "CONSTRAINT {} UNIQUE ({})",
                name,
                join_attr_names(table, &attrs)
            ));
        }

        for (idx, not_null) in table.not_nulls.iter().enumerate() {
            let attrs = sorted_attr_ids(table, &not_null.attributes);
            if attrs.len() <= 1 {
                continue;
            }
            let name =
                constraint_name_or_fallback(&not_null.name, &format!("NN_{}_{}", table.title, idx + 1));
            add_constraint_name(used_constraints, &name)?;
            lines.push(format!(
                "CONSTRAINT {} CHECK ({})",
                name,
                attrs
                    .iter()
                    .map(|id| format!("{} IS NOT NULL", table.attributes[*id].name))
                    .collect::<Vec<_>>()
                    .join(" AND ")
            ));
        }

        for (idx, check) in table.checks.iter().enumerate() {
            if check.condition.trim().is_empty() {
                return Err(SqlExportError::EmptyCheckCondition {
                    context: format!("table {} check {}", table.title, idx + 1),
                });
            }
            let name =
                constraint_name_or_fallback(&check.name, &format!("CHK_{}_{}", table.title, idx + 1));
            add_constraint_name(used_constraints, &name)?;
            lines.push(format!("CONSTRAINT {} CHECK ({})", name, check.condition));
        }

        Ok(lines)
    }

    fn render_foreign_keys(
        table: &Table,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
        used_constraints: &mut HashSet<String>,
    ) -> Result<Vec<String>, SqlExportError>;
}

/// Errors that can occur while generating SQL exports.
///
/// Enumeration contains precise diagnostics for invalid model state and
/// unsupported scenarios encountered during DDL generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlExportError {
    /// A required name (table, column, domain) was empty.
    EmptyName {
        kind: &'static str,
    },
    /// Two tables share the same name.
    DuplicateTableName {
        name: String,
    },
    /// Two domains share the same name.
    DuplicateDomainName {
        name: String,
    },
    /// Duplicate column name inside a table.
    DuplicateColumnName {
        table: String,
        name: String,
    },
    /// Duplicate constraint name detected while rendering.
    DuplicateConstraintName {
        name: String,
    },
    /// A CHECK constraint was present but its condition was empty.
    EmptyCheckCondition {
        context: String,
    },
    /// A foreign key references a table that does not exist in the model.
    MissingReferencedTable {
        table: String,
        foreign_key: String,
    },
    /// A foreign key references a column that does not exist in the referenced table.
    MissingReferencedColumn {
        table: String,
        foreign_key: String,
        referenced_table: String,
    },
    /// A column marked as FK has no corresponding FK constraint.
    MissingForeignKeyConstraint {
        table: String,
        column: String,
    },
    /// A column participates in multiple FK constraints making resolution ambiguous.
    AmbiguousForeignKey {
        table: String,
        column: String,
    },
    /// The model uses a data type base that the exporter does not support.
    UnsupportedDataType {
        context: String,
        base: usize,
    },
    /// An identifier (table/column/constraint) exceeds the dialect's length limit.
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
            },
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
        SqlDialect::Postgres => {
            crate::app::exports::sql::postgres::PostgresDialect.build_sql(tables, domains)
        }
    }
}

/// Validate that tables, columns and domains have non-empty and unique names.
///
/// Returns `Err(SqlExportError::EmptyName)` when a name is empty or a
/// `Duplicate*` variant when duplicates are found.
pub fn validate_object_names(
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
) -> Result<(), SqlExportError> {
    let mut seen_tables = HashSet::new();
    for (_, table) in sorted_tables(tables) {
        ensure_name("table", &table.title)?;
        if !seen_tables.insert(table.title.clone()) {
            return Err(SqlExportError::DuplicateTableName {
                name: table.title.clone(),
            });
        }
        let mut seen_columns = HashSet::new();
        for (_, attr) in sorted_attrs(table) {
            ensure_name("column", &attr.name)?;
            if !seen_columns.insert(attr.name.clone()) {
                return Err(SqlExportError::DuplicateColumnName {
                    table: table.title.clone(),
                    name: attr.name.clone(),
                });
            }
        }
    }

    let mut seen_domains = HashSet::new();
    for (_, domain) in sorted_domains(domains) {
        ensure_name("domain", &domain.name)?;
        if !seen_domains.insert(domain.name.clone()) {
            return Err(SqlExportError::DuplicateDomainName {
                name: domain.name.clone(),
            });
        }
    }

    Ok(())
}

/// Ensure a single object name is not empty.
///
/// `kind` is included in error diagnostics to identify whether the missing
/// name belonged to a table, column or domain.
pub fn ensure_name(kind: &'static str, name: &str) -> Result<(), SqlExportError> {
    if name.trim().is_empty() {
        return Err(SqlExportError::EmptyName { kind });
    }
    Ok(())
}

/// Return tables in a stable, display-friendly order.
///
/// Sorts by table title and then by internal id to produce deterministic output.
pub fn sorted_tables(tables: &SlotMap<TableId, Table>) -> Vec<(TableId, &Table)> {
    let mut out: Vec<_> = tables.iter().collect();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.title
            .cmp(&b.title)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

/// Return domains in a stable, display-friendly order.
pub fn sorted_domains(domains: &SlotMap<DomainId, Domain>) -> Vec<(DomainId, &Domain)> {
    let mut out: Vec<_> = domains.iter().collect();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.name
            .cmp(&b.name)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

/// Return attributes for a table in either explicit attribute order or sorted by id.
pub fn sorted_attrs(table: &Table) -> Vec<(AttrId, &Attribute)> {
    if table.attr_order.is_empty() {
        let mut out: Vec<_> = table.attributes.iter().collect();
        out.sort_by_key(|(id, _)| id.data().as_ffi());
        out
    } else {
        table
            .attr_order
            .iter()
            .filter_map(|&id| table.attributes.get(id).map(|attr| (id, attr)))
            .collect()
    }
}

/// Return attribute ids from `set` in table display order or sorted by id.
pub fn sorted_attr_ids(table: &Table, set: &HashSet<AttrId>) -> Vec<AttrId> {
    if table.attr_order.is_empty() {
        let mut out: Vec<_> = set.iter().copied().collect();
        out.sort_by_key(|id| id.data().as_ffi());
        out
    } else {
        table
            .attr_order
            .iter()
            .copied()
            .filter(|id| set.contains(id))
            .collect()
    }
}

/// Join attribute names into a comma-separated list for SQL fragments.
pub fn join_attr_names(table: &Table, attrs: &[AttrId]) -> String {
    attrs
        .iter()
        .map(|id| table.attributes[*id].name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Return the provided constraint `name` or a generated `fallback` when empty.
pub fn constraint_name_or_fallback(name: &str, fallback: &str) -> String {
    if name.trim().is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}

/// Resolve the target (table id, attribute id) for a FK-typed attribute.
///
/// Returns `Ok(Some((table_id, attr_id)))` when a single matching FK target
/// is found, `Ok(None)` when no FK constraint applies to the attribute, or
/// an error when ambiguous or missing information is encountered.
pub fn fk_attr_target(
    table: &Table,
    attr_id: AttrId,
) -> Result<Option<(TableId, AttrId)>, SqlExportError> {
    let Some(fk) = fk_for_attr(table, attr_id)? else {
        return Ok(None);
    };
    let ref_attr_id = get_referenced_attr_id(table, attr_id)?;
    // Ensure the FK specifies a referenced table id (table existence is not
    // resolved here; callers that need the table object may resolve it
    // separately).
    let Some(ref_table_id) = fk.references else {
        return Err(SqlExportError::MissingReferencedTable {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
        });
    };
    Ok(Some((ref_table_id, ref_attr_id)))
}

/// Resolve the foreign key definition, referenced table and referenced attribute
/// for a given local attribute. Returns an error when the FK cannot be resolved.
pub fn resolve_fk_attr<'a>(
    table: &'a Table,
    attr_id: AttrId,
    tables: &'a SlotMap<TableId, Table>,
) -> Result<(&'a ForeignKey, &'a Table, AttrId), SqlExportError> {
    let Some(fk) = fk_for_attr(table, attr_id)? else {
        return Err(SqlExportError::MissingForeignKeyConstraint {
            table: table.title.clone(),
            column: table.attributes[attr_id].name.clone(),
        });
    };
    let ref_table = get_referenced_table(table, fk, tables)?;
    let ref_attr_id = get_referenced_attr_id(table, attr_id)?;
    Ok((fk, ref_table, ref_attr_id))
}

/// Find the FK constraint that contains the given local attribute, if any.
pub fn fk_for_attr(table: &Table, attr_id: AttrId) -> Result<Option<&ForeignKey>, SqlExportError> {
    let mut matches = table
        .fks
        .values()
        .filter(|fk| fk.local_attrs.contains(&attr_id));
    let Some(fk) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(SqlExportError::AmbiguousForeignKey {
            table: table.title.clone(),
            column: table.attributes[attr_id].name.clone(),
        });
    }
    Ok(Some(fk))
}

/// Resolve and return the referenced table object for an FK, producing a
/// uniform diagnostic when the FK has no referenced table or the referenced
/// table id does not exist in `tables`.
pub fn get_referenced_table<'a>(
    table: &Table,
    fk: &ForeignKey,
    tables: &'a SlotMap<TableId, Table>,
) -> Result<&'a Table, SqlExportError> {
    let Some(ref_table_id) = fk.references else {
        return Err(SqlExportError::MissingReferencedTable {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
        });
    };
    let Some(ref_table) = tables.get(ref_table_id) else {
        return Err(SqlExportError::MissingReferencedTable {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
        });
    };
    Ok(ref_table)
}

/// Resolve the referenced attribute id for a local FK column and produce a
/// consistent diagnostic when the local attribute wasn't declared as an FK.
pub fn get_referenced_attr_id(table: &Table, attr_id: AttrId) -> Result<AttrId, SqlExportError> {
    let Some(AttributeType::ForeignKeyAttribute(ref_attr_id)) = table
        .attributes
        .get(attr_id)
        .map(|a| a.attribute_type.clone())
    else {
        return Err(SqlExportError::MissingForeignKeyConstraint {
            table: table.title.clone(),
            column: table.attributes[attr_id].name.clone(),
        });
    };
    Ok(ref_attr_id)
}

/// Resolve the referenced foreign key attribute for a local FK column.
pub fn resolve_referenced_attribute<'a>(
    table: &'a Table,
    attr_id: AttrId,
    tables: &'a SlotMap<TableId, Table>,
) -> Result<(&'a ForeignKey, &'a Table, &'a Attribute), SqlExportError> {
    let (fk, ref_table, ref_attr_id) = resolve_fk_attr(table, attr_id, tables)?;
    let Some(referenced_attr) = ref_table.attributes.get(ref_attr_id) else {
        return Err(SqlExportError::MissingReferencedColumn {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
            referenced_table: ref_table.title.clone(),
        });
    };
    Ok((fk, ref_table, referenced_attr))
}

/// Render a sequence of CHECK constraints using a shared code path.
pub fn render_check_constraints<F, G>(
    checks: &[Check],
    used_constraints: &mut HashSet<String>,
    mut empty_context: F,
    mut fallback_name: impl FnMut(usize, &Check) -> String,
    mut render_condition: G,
) -> Result<Vec<String>, SqlExportError>
where
    F: FnMut(usize, &Check) -> String,
    G: FnMut(&Check) -> Result<String, SqlExportError>,
{
    let mut lines = Vec::new();

    for (idx, check) in checks.iter().enumerate() {
        let condition = render_condition(check)?;
        if condition.trim().is_empty() {
            return Err(SqlExportError::EmptyCheckCondition {
                context: empty_context(idx, check),
            });
        }
        let name = fallback_name(idx, check);
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!("CONSTRAINT {} CHECK ({})", name, condition));
    }

    Ok(lines)
}

/// Build the shared prefix of a column definition.
pub fn render_column_parts(
    table: &Table,
    attr_id: AttrId,
    attr: &Attribute,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
    type_sql: impl FnOnce(&Table, AttrId, &Attribute, &SlotMap<TableId, Table>, &SlotMap<DomainId, Domain>) -> Result<String, SqlExportError>,
) -> Result<Vec<String>, SqlExportError> {
    let mut parts = vec![attr.name.clone(), type_sql(table, attr_id, attr, tables, domains)?];

    let inline_pk = exact_pk_name(table, attr_id);
    if let Some(name) = exact_not_null_name(table, attr_id) {
        add_constraint_name(used_constraints, &name)?;
        parts.push(format!("CONSTRAINT {} NOT NULL", name));
    } else if attr.not_null {
        parts.push("NOT NULL".to_string());
    }

    if let Some(name) = inline_pk.clone() {
        add_constraint_name(used_constraints, &name)?;
        parts.push(format!("CONSTRAINT {} PRIMARY KEY", name));
    }

    if inline_pk.is_none() {
        if let Some(name) = exact_unique_name(table, attr_id) {
            add_constraint_name(used_constraints, &name)?;
            parts.push(format!("CONSTRAINT {} UNIQUE", name));
        } else if attr.unique && !attr.pk {
            parts.push("UNIQUE".to_string());
        }
    }

    Ok(parts)
}

/// Render FK constraints shared by both SQL dialects.
pub fn render_foreign_keys(
    table: &Table,
    tables: &SlotMap<TableId, Table>,
    used_constraints: &mut HashSet<String>,
) -> Result<Vec<String>, SqlExportError> {
    let mut lines = Vec::new();

    for fk in table.fks.values() {
        let ref_table = get_referenced_table(table, fk, tables)?;

        let mut ref_to_local = std::collections::HashMap::new();
        for &local_id in &fk.local_attrs {
            let ref_id = get_referenced_attr_id(table, local_id)?;
            ref_to_local.insert(ref_id, local_id);
        }

        let ref_ids = sorted_attr_ids(ref_table, &ref_table.pk.attributes);
        if ref_ids.is_empty() {
            return Err(SqlExportError::MissingReferencedColumn {
                table: table.title.clone(),
                foreign_key: fk.name.clone(),
                referenced_table: ref_table.title.clone(),
            });
        }

        let mut local_cols = Vec::new();
        let mut ref_cols = Vec::new();
        for ref_id in &ref_ids {
            let Some(&local_id) = ref_to_local.get(ref_id) else {
                return Err(SqlExportError::MissingReferencedColumn {
                    table: table.title.clone(),
                    foreign_key: fk.name.clone(),
                    referenced_table: ref_table.title.clone(),
                });
            };
            local_cols.push(table.attributes[local_id].name.clone());
            ref_cols.push(ref_table.attributes[*ref_id].name.clone());
        }

        let name = constraint_name_or_fallback(
            &fk.name,
            &format!("FK_{}_{}", table.title, ref_table.title),
        );
        add_constraint_name(used_constraints, &name)?;
        let target_decl = referenced_columns_decl(ref_cols);
        lines.push(format!(
            "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} {}",
            name,
            local_cols.join(", "),
            ref_table.title,
            target_decl
        ));
    }

    Ok(lines)
}

/// If the attribute exactly matches a single-column PK, return the PK name.
pub fn exact_pk_name(table: &Table, attr_id: AttrId) -> Option<String> {
    let pk_attrs = sorted_attr_ids(table, &table.pk.attributes);
    (pk_attrs.len() == 1 && pk_attrs[0] == attr_id)
        .then(|| constraint_name_or_fallback(&table.pk.name, &format!("PK_{}", table.title)))
}

/// If the attribute exactly matches a single-column UNIQUE constraint, return its name.
pub fn exact_unique_name(table: &Table, attr_id: AttrId) -> Option<String> {
    table.uniques.iter().find_map(|u| {
        let attrs = sorted_attr_ids(table, &u.attributes);
        (attrs.len() == 1 && attrs[0] == attr_id).then(|| {
            constraint_name_or_fallback(
                &u.name,
                &format!("UQ_{}_{}", table.title, table.attributes[attr_id].name),
            )
        })
    })
}

/// If the attribute exactly matches a single-column NOT NULL constraint, return its name.
pub fn exact_not_null_name(table: &Table, attr_id: AttrId) -> Option<String> {
    table.not_nulls.iter().find_map(|n| {
        let attrs = sorted_attr_ids(table, &n.attributes);
        (attrs.len() == 1 && attrs[0] == attr_id).then(|| {
            constraint_name_or_fallback(
                &n.name,
                &format!("NN_{}_{}", table.title, table.attributes[attr_id].name),
            )
        })
    })
}

/// Format referenced column list as SQL declaration: "(col1, col2)".
pub fn referenced_columns_decl(cols: Vec<String>) -> String {
    format!("({})", cols.join(", "))
}

/// Register a constraint name and fail if it duplicates or exceeds limits.
pub fn add_constraint_name(used: &mut HashSet<String>, name: &str) -> Result<(), SqlExportError> {
    if name.len() > IDENTIFIER_LIMIT {
        return Err(SqlExportError::IdentifierTooLong {
            kind: "constraint",
            name: name.to_string(),
        });
    }
    if !used.insert(name.to_string()) {
        return Err(SqlExportError::DuplicateConstraintName {
            name: name.to_string(),
        });
    }
    Ok(())
}