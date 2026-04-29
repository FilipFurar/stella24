//! SQL export module for generating Oracle SQL DDL from the database schema.
//!
//! This module provides functionality to convert the in-memory database schema representation
//! into Oracle-compatible SQL DDL (Data Definition Language). It handles table creation,
//! domain definitions, constraints, and comprehensive validation of the schema.

use crate::app::{AppStella, DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::ForeignKey;
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::{Key, SlotMap};
use std::collections::HashSet;
use std::fmt::{self, Write};
use std::fs;

/// Maximum length for Oracle identifiers (table names, column names, constraint names, etc.)
const IDENTIFIER_LIMIT: usize = 128;
/// Errors that can occur during SQL export and validation.
///
/// This enum represents all possible validation and generation errors that may occur
/// when converting the schema to Oracle SQL DDL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlExportError {
    /// An empty name was provided for a named object (table, column, domain, etc.)
    EmptyName { kind: &'static str },
    /// Two or more tables have the same name
    DuplicateTableName { name: String },
    /// Two or more domains have the same name
    DuplicateDomainName { name: String },
    /// Two or more columns in the same table have the same name
    DuplicateColumnName { table: String, name: String },
    /// Two or more constraints have the same name
    DuplicateConstraintName { name: String },
    /// A CHECK constraint has an empty or whitespace-only condition
    EmptyCheckCondition { context: String },
    /// A foreign key references a table that doesn't exist
    MissingReferencedTable { table: String, foreign_key: String },
    /// A foreign key references a column that doesn't exist in the target table
    MissingReferencedColumn {
        table: String,
        foreign_key: String,
        referenced_table: String,
    },
    /// A column is marked as a foreign key attribute but has no matching FK constraint
    MissingForeignKeyConstraint { table: String, column: String },
    /// A column belongs to multiple foreign key constraints (ambiguous)
    AmbiguousForeignKey { table: String, column: String },
    /// A data type with an unsupported base type was encountered
    UnsupportedDataType { context: String, base: usize },
    /// An identifier exceeds the Oracle limit of 128 characters
    IdentifierTooLong { kind: &'static str, name: String },
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
            SqlExportError::MissingReferencedTable { table, foreign_key } => {
                write!(
                    f,
                    "foreign key {foreign_key} in table {table} references a missing table"
                )
            }
            SqlExportError::MissingReferencedColumn {
                table,
                foreign_key,
                referenced_table,
            } => {
                write!(
                    f,
                    "foreign key {foreign_key} in table {table} points to a missing column in {referenced_table}"
                )
            }
            SqlExportError::MissingForeignKeyConstraint { table, column } => {
                write!(
                    f,
                    "column {column} in table {table} is typed as FK but has no matching FK constraint"
                )
            }
            SqlExportError::AmbiguousForeignKey { table, column } => {
                write!(
                    f,
                    "column {column} in table {table} belongs to multiple FK constraints"
                )
            }
            SqlExportError::UnsupportedDataType { context, base } => {
                write!(f, "unsupported datatype base {base} ({context})")
            }
            SqlExportError::IdentifierTooLong { kind, name } => {
                write!(f, "{kind} identifier too long for Oracle: {name}")
            }
        }
    }
}
impl std::error::Error for SqlExportError {}
/// Generates Oracle SQL DDL from the given tables and domains.
///
/// This function validates the entire schema and generates a complete Oracle SQL script
/// that can be executed to create the database structure. It includes:
/// - Domain definitions with CHECK constraints
/// - Table definitions with all columns and constraints
/// - Primary keys, unique constraints, foreign keys, CHECK constraints, and NOT NULL constraints
///
/// # Arguments
/// * `tables` - The collection of tables to export
/// * `domains` - The collection of domains to export
///
/// # Returns
/// A complete Oracle SQL DDL script as a string, or an error if validation fails
///
/// # Errors
/// Returns `SqlExportError` if:
/// - Any table, column, or domain has an empty name
/// - Duplicate names are found at any level
/// - Identifiers exceed the 128-character limit
/// - Foreign keys reference missing tables or columns
/// - Any CHECK constraint has an empty condition
/// - Data types are not supported
pub fn build_oracle_sql(
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
) -> Result<String, SqlExportError> {
    validate_object_names(tables, domains)?;
    let mut used_constraints = HashSet::new();
    let mut out = String::new();
    writeln!(out, "-- stella24 Oracle SQL export").expect("write to string");
    writeln!(out).expect("write to string");

    if !domains.is_empty() {
        for (_, domain) in sorted_domains(domains) {
            render_domain(&mut out, domain, &mut used_constraints)?;
            writeln!(out).expect("write to string");
        }
    }

    for (table_id, table) in sorted_tables(tables) {
        render_table(
            &mut out,
            table_id,
            table,
            tables,
            domains,
            &mut used_constraints,
        )?;
        writeln!(out).expect("write to string");
    }
    Ok(out)
}

/// Renders a domain as an Oracle CREATE DOMAIN statement.
///
/// Generates the SQL for creating a domain with its data type and any CHECK constraints.
///
/// # Arguments
/// * `out` - The output string to append the SQL to
/// * `domain` - The domain to render
/// * `used_constraints` - Set of already-used constraint names to prevent duplicates
fn render_domain(
    out: &mut String,
    domain: &Domain,
    used_constraints: &mut HashSet<String>,
) -> Result<(), SqlExportError> {
    writeln!(
        out,
        "CREATE DOMAIN {} AS {}",
        quote_ident(&domain.name),
        oracle_type_sql(&domain.data_type, &format!("domain {}", domain.name))?,
    )
    .expect("write to string");

    if domain.check_constraints.is_empty() {
        writeln!(out, ";").expect("write to string");
        return Ok(());
    }

    for (idx, check) in domain.check_constraints.iter().enumerate() {
        if check.condition.trim().is_empty() {
            return Err(SqlExportError::EmptyCheckCondition {
                context: format!("domain {}", domain.name),
            });
        }
        let name = constraint_name_or_fallback(
            &check.name,
            &format!("CHK_DOMAIN_{}_{}", domain.name, idx + 1),
        );
        add_constraint_name(used_constraints, &name)?;
        writeln!(
            out,
            "    CONSTRAINT {} CHECK ({}){}",
            quote_ident(&name),
            check.condition,
            if idx + 1 == domain.check_constraints.len() {
                ";"
            } else {
                ""
            }
        )
        .expect("write to string");
    }

    Ok(())
}
/// Writes Oracle SQL DDL to a file.
///
/// This is a convenience function that combines `build_oracle_sql` with file writing.
/// The generated SQL is written directly to the specified path.
///
/// # Arguments
/// * `tables` - The collection of tables to export
/// * `domains` - The collection of domains to export
/// * `path` - The file path where the SQL will be written
///
/// # Returns
/// Ok if successful, or an error if SQL generation or file writing fails
///
/// # Errors
/// Returns the same errors as `build_oracle_sql`, or an error if the file cannot be written
pub fn write_oracle_sql(
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    path: &str,
) -> Result<(), SqlExportError> {
    let sql = build_oracle_sql(tables, domains)?;
    fs::write(path, sql).map_err(|_| SqlExportError::IdentifierTooLong {
        kind: "path",
        name: path.to_string(),
    })
}
impl AppStella {
    /// Exports the current schema to an Oracle SQL DDL file.
    ///
    /// This method generates Oracle SQL DDL from the current application state and writes it
    /// to a file. Any errors are printed to stderr rather than propagated.
    ///
    /// # Arguments
    /// * `path` - The file path where the SQL will be written
    pub fn to_oracle_sql(&self, path: &str) {
        match build_oracle_sql(self.tables(), self.domains()) {
            Ok(sql) => {
                if let Err(err) = fs::write(path, sql) {
                    eprintln!("Error exporting SQL: {err}");
                }
            }
            Err(err) => eprintln!("Error exporting SQL: {err}"),
        }
    }
}
/// Validates that all tables, columns, and domains have valid, unique names.
///
/// This function performs the following checks:
/// - All table names are non-empty and unique
/// - All column names within each table are non-empty and unique
/// - All domain names are non-empty and unique
/// - All names are within the Oracle identifier length limit
fn validate_object_names(
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
/// Renders a table as an Oracle CREATE TABLE statement.
///
/// Generates the complete CREATE TABLE statement including all columns, inline constraints,
/// and table-level constraints (primary key, foreign keys, unique, CHECK, NOT NULL).
///
/// # Arguments
/// * `out` - The output string to append the SQL to
/// * `table_id` - The ID of the table being rendered (for reference)
/// * `table` - The table to render
/// * `tables` - All tables (for foreign key resolution)
/// * `domains` - All domains (for type resolution)
/// * `used_constraints` - Set of already-used constraint names
fn render_table(
    out: &mut String,
    table_id: TableId,
    table: &Table,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
) -> Result<(), SqlExportError> {
    let _ = table_id;
    writeln!(out, "CREATE TABLE {} (", quote_ident(&table.title)).expect("write to string");
    let mut lines = Vec::new();
    for (attr_id, attr) in sorted_attrs(table) {
        lines.push(render_column(
            table,
            attr_id,
            attr,
            tables,
            domains,
            used_constraints,
        )?);
    }
    lines.extend(render_table_constraints(table, tables, used_constraints)?);
    for (i, line) in lines.iter().enumerate() {
        let comma = if i + 1 == lines.len() { "" } else { "," };
        writeln!(out, "    {}{}", line, comma).expect("write to string");
    }
    writeln!(out, ");").expect("write to string");
    Ok(())
}
/// Renders a column definition with its inline constraints.
///
/// Generates the SQL for a single column including its name, data type, and inline constraints:
/// - NOT NULL (if applicable and not part of a multi-column constraint)
/// - PRIMARY KEY (if single-column PK)
/// - UNIQUE (if applicable and not primary key)
/// - FOREIGN KEY (if inline FK possible)
///
/// # Arguments
/// * `table` - The table containing the column
/// * `attr_id` - The ID of the attribute/column
/// * `attr` - The attribute/column definition
/// * `tables` - All tables (for foreign key resolution)
/// * `domains` - All domains (for type resolution)
/// * `used_constraints` - Set of already-used constraint names
fn render_column(
    table: &Table,
    attr_id: AttrId,
    attr: &Attribute,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
) -> Result<String, SqlExportError> {
    let mut parts = vec![
        quote_ident(&attr.name),
        attribute_type_sql(table, attr_id, attr, tables, domains)?,
    ];
    let inline_pk = exact_pk_name(table, attr_id);
    if let Some(name) = exact_not_null_name(table, attr_id) {
        add_constraint_name(used_constraints, &name)?;
        parts.push(format!("CONSTRAINT {} NOT NULL", quote_ident(&name)));
    } else if attr.not_null || domain_not_null(attr, domains) {
        parts.push("NOT NULL".to_string());
    }
    if let Some(name) = inline_pk.clone() {
        add_constraint_name(used_constraints, &name)?;
        parts.push(format!("CONSTRAINT {} PRIMARY KEY", quote_ident(&name)));
    }
    if inline_pk.is_none() {
        if let Some(name) = exact_unique_name(table, attr_id) {
            add_constraint_name(used_constraints, &name)?;
            parts.push(format!("CONSTRAINT {} UNIQUE", quote_ident(&name)));
        } else if attr.unique && !attr.pk {
            parts.push("UNIQUE".to_string());
        }
    }
    if let Some((fk, ref_table, ref_attr)) = inline_fk(table, attr_id, tables)? {
        let name =
            constraint_name_or_fallback(&fk.name, &format!("FK_{}_{}", table.title, attr.name));
        add_constraint_name(used_constraints, &name)?;
        parts.push(format!(
            "CONSTRAINT {} REFERENCES {} ({})",
            quote_ident(&name),
            quote_ident(&ref_table.title),
            quote_ident(&ref_attr.name)
        ));
    }
    Ok(parts.join(" "))
}
/// Renders table-level constraints that cannot be inlined on a column.
///
/// Generates SQL for:
/// - Multi-column PRIMARY KEY constraints
/// - Multi-column UNIQUE constraints
/// - Multi-column NOT NULL constraints
/// - Multi-column FOREIGN KEY constraints
/// - All CHECK constraints (both single and multi-column)
///
/// # Arguments
/// * `table` - The table containing the constraints
/// * `tables` - All tables (for foreign key resolution)
/// * `used_constraints` - Set of already-used constraint names
fn render_table_constraints(
    table: &Table,
    tables: &SlotMap<TableId, Table>,
    used_constraints: &mut HashSet<String>,
) -> Result<Vec<String>, SqlExportError> {
    let mut lines = Vec::new();
    let pk_attrs = sorted_attr_ids(&table.pk.attributes);
    if pk_attrs.len() > 1 {
        let name = constraint_name_or_fallback(&table.pk.name, &format!("PK_{}", table.title));
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!(
            "CONSTRAINT {} PRIMARY KEY ({})",
            quote_ident(&name),
            join_attr_names(table, &pk_attrs)
        ));
    }
    for (idx, unique) in table.uniques.iter().enumerate() {
        let attrs = sorted_attr_ids(&unique.attributes);
        if attrs.is_empty() || attrs.len() == 1 {
            continue;
        }
        let name =
            constraint_name_or_fallback(&unique.name, &format!("UQ_{}_{}", table.title, idx + 1));
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!(
            "CONSTRAINT {} UNIQUE ({})",
            quote_ident(&name),
            join_attr_names(table, &attrs)
        ));
    }
    for (idx, not_null) in table.not_nulls.iter().enumerate() {
        let attrs = sorted_attr_ids(&not_null.attributes);
        if attrs.is_empty() || attrs.len() == 1 {
            continue;
        }
        let name =
            constraint_name_or_fallback(&not_null.name, &format!("NN_{}_{}", table.title, idx + 1));
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!(
            "CONSTRAINT {} CHECK ({})",
            quote_ident(&name),
            attrs
                .iter()
                .map(|id| format!("{} IS NOT NULL", quote_ident(&table.attributes[*id].name)))
                .collect::<Vec<_>>()
                .join(" AND ")
        ));
    }
    for fk in table.fks.values() {
        let (ref_table, local_cols, ref_cols) = fk_columns(table, fk, tables)?;
        if local_cols.len() == 1 {
            continue;
        }
        let name = constraint_name_or_fallback(
            &fk.name,
            &format!("FK_{}_{}", table.title, ref_table.title),
        );
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!(
            "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
            quote_ident(&name),
            local_cols
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", "),
            quote_ident(&ref_table.title),
            ref_cols
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for (idx, check) in table.checks.iter().enumerate() {
        if check.condition.trim().is_empty() {
            return Err(SqlExportError::EmptyCheckCondition {
                context: format!("table {} check {}", table.title, idx + 1),
            });
        }
        let name = constraint_name_or_fallback(
            &format!("CHK_{}_{}", table.title, idx + 1),
            &format!("CHK_{}_{}", table.title, idx + 1),
        );
        add_constraint_name(used_constraints, &name)?;
        lines.push(format!(
            "CONSTRAINT {} CHECK ({})",
            quote_ident(&name),
            check.condition
        ));
    }
    Ok(lines)
}
/// Determines the SQL data type for an attribute.
///
/// Resolves the attribute's type based on whether it's:
/// - A logical type (directly specified data type)
/// - A domain type (resolves to the domain's data type)
/// - A foreign key attribute (resolves to the referenced column's type)
///
/// # Arguments
/// * `table` - The table containing the attribute
/// * `attr_id` - The ID of the attribute
/// * `attr` - The attribute definition
/// * `tables` - All tables (for foreign key resolution)
/// * `domains` - All domains (for type resolution)
fn attribute_type_sql(
    table: &Table,
    attr_id: AttrId,
    attr: &Attribute,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
) -> Result<String, SqlExportError> {
    match attr.attribute_type.clone() {
        AttributeType::Logical(dt) => {
            oracle_type_sql(&dt, &format!("column {}.{}", table.title, attr.name))
        }
        AttributeType::Domain(domain_id) => {
            let Some(domain) = domains.get(domain_id) else {
                return Ok("NUMBER".to_string());
            };
            Ok(quote_ident(&domain.name))
        }
        AttributeType::ForeignKeyAttribute(_) => {
            let (fk, ref_table, ref_attr_id) = resolve_fk_attr(table, attr_id, tables)?;
            let Some(referenced_attr) = ref_table.attributes.get(ref_attr_id) else {
                return Err(SqlExportError::MissingReferencedColumn {
                    table: table.title.clone(),
                    foreign_key: fk.name.clone(),
                    referenced_table: ref_table.title.clone(),
                });
            };
            match referenced_attr.attribute_type.clone() {
                AttributeType::Logical(dt) => oracle_type_sql(
                    &dt,
                    &format!(
                        "foreign key target {}.{}",
                        ref_table.title, referenced_attr.name
                    ),
                ),
                AttributeType::Domain(domain_id) => {
                    let Some(domain) = domains.get(domain_id) else {
                        return Ok("NUMBER".to_string());
                    };
                    oracle_type_sql(
                        &domain.data_type,
                        &format!("foreign key target domain {}", domain.name),
                    )
                }
                AttributeType::ForeignKeyAttribute(_) => Ok("NUMBER".to_string()),
            }
        }
    }
}
/// Determines if a foreign key constraint can be inlined on a column definition.
///
/// A foreign key can be inlined only if:
/// - The column belongs to a foreign key constraint
/// - The constraint references exactly one local column
/// - The referenced table's primary key has exactly one column
///
/// Returns the FK constraint, referenced table, and referenced attribute if eligible.
fn inline_fk<'a>(
    table: &'a Table,
    attr_id: AttrId,
    tables: &'a SlotMap<TableId, Table>,
) -> Result<Option<(&'a ForeignKey, &'a Table, &'a Attribute)>, SqlExportError> {
    let Some(fk) = fk_for_attr(table, attr_id)? else {
        return Ok(None);
    };
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
    let Some((_, ref_attr_id)) = fk_attr_target(table, attr_id)? else {
        return Err(SqlExportError::MissingForeignKeyConstraint {
            table: table.title.clone(),
            column: table.attributes[attr_id].name.clone(),
        });
    };
    let Some(ref_attr) = ref_table.attributes.get(ref_attr_id) else {
        return Err(SqlExportError::MissingReferencedColumn {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
            referenced_table: ref_table.title.clone(),
        });
    };
    if fk.local_attrs.len() == 1 && ref_table.pk.attributes.len() == 1 {
        Ok(Some((fk, ref_table, ref_attr)))
    } else {
        Ok(None)
    }
}
/// Extracts the local and referenced column names for a foreign key constraint.
///
/// Resolves the column names for both the local table and the referenced table,
/// verifying that the counts match and the references are valid.
///
/// Returns a tuple of (referenced table, local column names, referenced column names).
fn fk_columns<'a>(
    table: &Table,
    fk: &ForeignKey,
    tables: &'a SlotMap<TableId, Table>,
) -> Result<(&'a Table, Vec<String>, Vec<String>), SqlExportError> {
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
    let local_pairs = fk_attr_pairs(table, fk)?;
    let ref_ids = sorted_attr_ids(&ref_table.pk.attributes);
    if local_pairs.len() != ref_ids.len() {
        return Err(SqlExportError::MissingReferencedColumn {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
            referenced_table: ref_table.title.clone(),
        });
    }
    let local_cols = local_pairs
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let ref_cols = ref_ids
        .iter()
        .map(|id| ref_table.attributes[*id].name.clone())
        .collect::<Vec<_>>();
    Ok((ref_table, local_cols, ref_cols))
}
/// Gets the local and referenced attribute ID pairs for a foreign key constraint.
///
/// Returns a sorted list of (local column name, referenced attribute ID) pairs.
fn fk_attr_pairs(table: &Table, fk: &ForeignKey) -> Result<Vec<(String, AttrId)>, SqlExportError> {
    let mut pairs = Vec::new();
    for attr_id in sorted_attr_ids(&fk.local_attrs) {
        let Some((_, ref_attr_id)) = fk_attr_target(table, attr_id)? else {
            return Err(SqlExportError::MissingForeignKeyConstraint {
                table: table.title.clone(),
                column: table.attributes[attr_id].name.clone(),
            });
        };
        pairs.push((table.attributes[attr_id].name.clone(), ref_attr_id));
    }
    pairs.sort_by_key(|(_, ref_id)| ref_id.data().as_ffi());
    Ok(pairs)
}
/// Finds the foreign key constraint and target attribute for a column marked as a foreign key attribute.
///
/// Returns the referenced table ID and attribute ID if found.
/// Errors if the column belongs to multiple FK constraints (ambiguous).
fn fk_attr_target(
    table: &Table,
    attr_id: AttrId,
) -> Result<Option<(TableId, AttrId)>, SqlExportError> {
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
    let Some(ref_table_id) = fk.references else {
        return Err(SqlExportError::MissingReferencedTable {
            table: table.title.clone(),
            foreign_key: fk.name.clone(),
        });
    };
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
    Ok(Some((ref_table_id, ref_attr_id)))
}
/// Resolves a foreign key attribute to its complete definition and target.
///
/// Returns the FK constraint, referenced table, and referenced attribute ID.
fn resolve_fk_attr<'a>(
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
    let Some((_, ref_attr_id)) = fk_attr_target(table, attr_id)? else {
        return Err(SqlExportError::MissingForeignKeyConstraint {
            table: table.title.clone(),
            column: table.attributes[attr_id].name.clone(),
        });
    };
    Ok((fk, ref_table, ref_attr_id))
}
/// Finds the foreign key constraint that contains the given attribute.
///
/// Returns None if the attribute is not a foreign key column.
/// Errors if the attribute belongs to multiple FK constraints (ambiguous).
fn fk_for_attr(table: &Table, attr_id: AttrId) -> Result<Option<&ForeignKey>, SqlExportError> {
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
/// Gets the primary key constraint name if this attribute is the only column in the PK.
fn exact_pk_name(table: &Table, attr_id: AttrId) -> Option<String> {
    let attrs = sorted_attr_ids(&table.pk.attributes);
    (attrs.len() == 1 && attrs[0] == attr_id)
        .then(|| constraint_name_or_fallback(&table.pk.name, &format!("PK_{}", table.title)))
}
/// Gets the unique constraint name if this attribute is the only column in the constraint.
fn exact_unique_name(table: &Table, attr_id: AttrId) -> Option<String> {
    table.uniques.iter().find_map(|u| {
        let attrs = sorted_attr_ids(&u.attributes);
        (attrs.len() == 1 && attrs[0] == attr_id).then(|| {
            constraint_name_or_fallback(
                &u.name,
                &format!("UQ_{}_{}", table.title, table.attributes[attr_id].name),
            )
        })
    })
}
/// Gets the NOT NULL constraint name if this attribute is the only column in the constraint.
fn exact_not_null_name(table: &Table, attr_id: AttrId) -> Option<String> {
    table.not_nulls.iter().find_map(|n| {
        let attrs = sorted_attr_ids(&n.attributes);
        (attrs.len() == 1 && attrs[0] == attr_id).then(|| {
            constraint_name_or_fallback(
                &n.name,
                &format!("NN_{}_{}", table.title, table.attributes[attr_id].name),
            )
        })
    })
}
/// Checks if the domain containing this attribute specifies NOT NULL.
/// Currently always returns false as domains don't have NOT NULL constraints.
fn domain_not_null(attr: &Attribute, domains: &SlotMap<DomainId, Domain>) -> bool {
    let _ = (attr, domains);
    false
}
/// Returns sorted tables by name (then by insertion order for stability).
fn sorted_tables(tables: &SlotMap<TableId, Table>) -> Vec<(TableId, &Table)> {
    let mut out = tables.iter().collect::<Vec<_>>();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.title
            .cmp(&b.title)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

/// Returns sorted domains by name (then by insertion order for stability).
fn sorted_domains(domains: &SlotMap<DomainId, Domain>) -> Vec<(DomainId, &Domain)> {
    let mut out = domains.iter().collect::<Vec<_>>();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.name
            .cmp(&b.name)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

/// Returns sorted attributes by insertion order (their ID).
fn sorted_attrs(table: &Table) -> Vec<(AttrId, &Attribute)> {
    let mut out = table.attributes.iter().collect::<Vec<_>>();
    out.sort_by_key(|(id, _)| id.data().as_ffi());
    out
}

/// Returns sorted attribute IDs from a set by insertion order.
fn sorted_attr_ids(set: &std::collections::HashSet<AttrId>) -> Vec<AttrId> {
    let mut out = set.iter().copied().collect::<Vec<_>>();
    out.sort_by_key(|id| id.data().as_ffi());
    out
}
/// Joins attribute names into a comma-separated, quoted list suitable for SQL.
fn join_attr_names(table: &Table, attrs: &[AttrId]) -> String {
    attrs
        .iter()
        .map(|id| quote_ident(&table.attributes[*id].name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Validates that a name is non-empty and within the identifier length limit.
fn ensure_name(kind: &'static str, name: &str) -> Result<(), SqlExportError> {
    if name.trim().is_empty() {
        return Err(SqlExportError::EmptyName { kind });
    }
    if name.len() > IDENTIFIER_LIMIT {
        return Err(SqlExportError::IdentifierTooLong {
            kind,
            name: name.to_string(),
        });
    }
    Ok(())
}
/// Adds a constraint name to the used set, ensuring no duplicates and respecting identifier limits.
fn add_constraint_name(used: &mut HashSet<String>, name: &str) -> Result<(), SqlExportError> {
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
/// Returns the constraint name if non-empty, otherwise uses the fallback name.
fn constraint_name_or_fallback(name: &str, fallback: &str) -> String {
    if name.trim().is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}

/// Quotes an identifier with double quotes, escaping any embedded quotes by doubling them.
/// This follows Oracle's identifier quoting rules.
fn quote_ident(name: &str) -> String {
    format!("\'{}\'", name.replace('\'', "\'\'"))
}

/// Converts a DataType to its Oracle SQL representation.
///
/// Handles the following base types:
/// - CHAR: CHAR(length)
/// - VARCHAR: VARCHAR2(length)
/// - BOOL: NUMBER(1)
/// - NUMBER(precision) or NUMBER(precision, scale)
/// - DATE: DATE
fn oracle_type_sql(dt: &DataType, context: &str) -> Result<String, SqlExportError> {
    let Some(def) = DATA_TYPES.get(dt.base) else {
        return Err(SqlExportError::UnsupportedDataType {
            context: context.to_string(),
            base: dt.base,
        });
    };
    let sql = match def.name {
        "CHAR" => format!("CHAR({})", dt.params.get(0).copied().unwrap_or(1)),
        "VARCHAR" => format!("VARCHAR2({})", dt.params.get(0).copied().unwrap_or(1)),
        "BOOL" => "NUMBER(1)".to_string(),
        "NUMBER" => match dt.params.as_slice() {
            [precision, scale] => format!("NUMBER({}, {})", precision, scale),
            [precision] => format!("NUMBER({})", precision),
            _ => "NUMBER".to_string(),
        },
        "DATE" => "DATE".to_string(),
        other => {
            return Err(SqlExportError::UnsupportedDataType {
                context: format!("{} ({other})", context),
                base: dt.base,
            });
        }
    };
    Ok(sql)
}
