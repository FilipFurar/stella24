//! SQLite SQL DDL exporter.

use crate::app::exports::sql::sql_export::{
    render_check_constraints,
    render_column_parts,
    resolve_referenced_attribute,
    sorted_attrs,
    sorted_tables,
    validate_object_names,
    Export,
    SqlDialect,
    SqlExportError,
};
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::datatype::DataType;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::{SlotMap};
use std::collections::HashSet;
use std::fmt::Write;

/// SQLite SQL DDL exporter.
///
/// Produces SQLite-compatible CREATE TABLE statements and enables PRAGMA
/// foreign_keys at the top of the generated script.
#[derive(Debug, Clone, Copy, Default)]
pub struct SqliteDialect;

impl Export for SqliteDialect {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Sqlite
    }

    fn build_sql(
        &self,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError> {
        validate_object_names(tables, domains)?;

        let mut used_constraints = HashSet::new();
        let mut out = String::new();
        writeln!(out, "-- stella24 SQLite SQL export").unwrap();
        writeln!(out, "PRAGMA foreign_keys = ON;").unwrap();
        writeln!(out).unwrap();

        for (_, table) in sorted_tables(tables) {
            Self::render_table(&mut out, table, tables, domains, &mut used_constraints)?;
            writeln!(out).unwrap();
        }

        Ok(out)
    }

    fn attribute_type_sql(table: &Table, attr_id: AttrId, attr: &Attribute, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>) -> Result<String, SqlExportError> {
        match &attr.attribute_type {
            AttributeType::Logical(dt) => Ok(sqlite_type_sql(dt)),
            AttributeType::Domain(domain_id) => {
                let Some(domain) = domains.get(*domain_id) else {
                    return Ok("NUMERIC".to_string());
                };
                Ok(sqlite_type_sql(&domain.data_type))
            }
            AttributeType::ForeignKeyAttribute(_) => {
                let (_fk, _ref_table, referenced_attr) = resolve_referenced_attribute(table, attr_id, tables)?;
                match &referenced_attr.attribute_type {
                    AttributeType::Logical(dt) => Ok(sqlite_type_sql(dt)),
                    AttributeType::Domain(domain_id) => {
                        let Some(domain) = domains.get(*domain_id) else {
                            return Ok("NUMERIC".to_string());
                        };
                        Ok(sqlite_type_sql(&domain.data_type))
                    }
                    AttributeType::ForeignKeyAttribute(_) => Ok("NUMERIC".to_string()),
                }
            }
        }
    }

    fn render_table(out: &mut String, table: &Table, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>, used_constraints: &mut HashSet<String>) -> Result<(), SqlExportError> {
        writeln!(out, "CREATE TABLE {} (", table.title).unwrap();
        let mut lines = Vec::new();

        for (attr_id, attr) in sorted_attrs(table) {
            lines.push(Self::render_column(
                table,
                attr_id,
                attr,
                tables,
                domains,
                used_constraints,
            )?);
        }

        lines.extend(Self::render_table_constraints(table, used_constraints)?);
        lines.extend(Self::render_foreign_keys(
            table,
            tables,
            domains,
            used_constraints,
        )?);

        for (idx, line) in lines.iter().enumerate() {
            let comma = if idx + 1 == lines.len() { "" } else { "," };
            writeln!(out, "    {}{}", line, comma).unwrap();
        }
        writeln!(out, ");").unwrap();
        Ok(())
    }

    fn render_column(table: &Table, attr_id: AttrId, attr: &Attribute, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>, used_constraints: &mut HashSet<String>) -> Result<String, SqlExportError> {
        let mut parts = render_column_parts(
            table,
            attr_id,
            attr,
            tables,
            domains,
            used_constraints,
            Self::attribute_type_sql,
        )?;

        if let AttributeType::Domain(domain_id) = &attr.attribute_type
            && let Some(domain) = domains.get(*domain_id)
        {
            parts.extend(render_check_constraints(
                &domain.check_constraints,
                used_constraints,
                |_, _| format!("domain {}", domain.name),
                |idx, check| {
                    crate::app::exports::sql::sql_export::constraint_name_or_fallback(
                        &check.name,
                        &format!("CHK_DOMAIN_{}_{}", domain.name, idx + 1),
                    )
                },
                |check| Ok(rewrite_domain_check(&check.condition, &attr.name)),
            )?);
        }

        Ok(parts.join(" "))
    }


    fn render_foreign_keys(table: &Table, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>, used_constraints: &mut HashSet<String>) -> Result<Vec<String>, SqlExportError> {
        let _ = domains;
        crate::app::exports::sql::sql_export::render_foreign_keys(table, tables, used_constraints)
    }
}

fn rewrite_domain_check(condition: &str, column_name: &str) -> String {
    condition.replace("VALUE", column_name)
}

/// Map a model `DataType` to a SQLite affinity/type string.
///
/// SQLite has a relaxed type system; this helper produces a best-effort
/// mapping to TEXT, NUMERIC, REAL or BLOB depending on the model base type.
fn sqlite_type_sql(dt: &DataType) -> String {
    let Some(def) = crate::model::datatype::DATA_TYPES.get(dt.base) else {
        return "NUMERIC".to_string();
    };

    match def.name {
        "CHAR"
        | "VARCHAR2"
        | "NCHAR"
        | "NVARCHAR2"
        | "LONG"
        | "LONG RAW"
        | "NCLOB"
        | "DATE"
        | "TIMESTAMP"
        | "TIMESTAMP WITH TIME ZONE"
        | "TIMESTAMP WITH LOCAL TIME ZONE"
        | "INTERVAL_YEAR"
        | "INTERVAL_DAY" => "TEXT".to_string(),
        "NUMBER" => "NUMERIC".to_string(),
        "FLOAT" | "BINARY_FLOAT" | "BINARY_DOUBLE" => "REAL".to_string(),
        "BLOB" | "BFILE" => "BLOB".to_string(),
        "ROWID" | "UROWID" => "TEXT".to_string(),
        _ => "NUMERIC".to_string(),
    }
}