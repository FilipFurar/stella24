//! SQLite SQL DDL exporter.

use crate::app::exports::sql::sql_export::{
    Export, SqlDialect, SqlExportError, constraint_name_or_fallback, render_check_constraints,
    render_column_parts, resolve_referenced_attribute, sorted_attrs, validate_object_names,
};
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::datatype::DataType;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::SlotMap;
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
        writeln!(out, "-- stella24 SQLite SQL export").map_err(|_| SqlExportError::WriteError {
            context: "writing SQLite header".to_string(),
        })?;
        writeln!(out, "PRAGMA foreign_keys = ON;").map_err(|_| SqlExportError::WriteError {
            context: "writing SQLite PRAGMA".to_string(),
        })?;
        writeln!(out).map_err(|_| SqlExportError::WriteError {
            context: "writing SQLite header newline".to_string(),
        })?;

        // Try to emit tables in dependency order so referenced tables are
        // created before tables that reference them. SQLite cannot add FKs
        // later via ALTER TABLE, so ordering is important.
        match crate::app::exports::sql::sql_export::topologically_sorted_tables(tables) {
            Ok(order) => {
                for (_id, table) in order {
                    Self::render_table(&mut out, table, tables, domains, &mut used_constraints)?;
                    writeln!(out).map_err(|_| SqlExportError::WriteError {
                        context: format!("writing newline after CREATE TABLE {}", table.title),
                    })?;
                }
            }
            Err(SqlExportError::CyclicForeignKeyDependencies { tables: cyclic }) => {
                // Cyclic dependencies detected; fall back to deterministic
                // alphabetical order and emit a warning comment.
                let names: Vec<String> = cyclic
                    .into_iter()
                    .filter_map(|id| tables.get(id).map(|t| t.title.clone()))
                    .collect();
                writeln!(
                    out,
                    "-- WARNING: cyclic foreign-key dependencies detected: {}",
                    names.join(", ")
                )
                .map_err(|_| SqlExportError::WriteError {
                    context: "writing sqlite cycle warning".to_string(),
                })?;
                writeln!(out, "-- Falling back to deterministic table order; some CREATE TABLE statements may reference tables not yet created").map_err(|_| SqlExportError::WriteError { context: "writing sqlite cycle fallback warning".to_string() })?;
                writeln!(out).map_err(|_| SqlExportError::WriteError {
                    context: "writing newline after sqlite warnings".to_string(),
                })?;
                for (_, table) in crate::app::exports::sql::sql_export::sorted_tables(tables) {
                    Self::render_table(&mut out, table, tables, domains, &mut used_constraints)?;
                    writeln!(out).map_err(|_| SqlExportError::WriteError {
                        context: format!("writing newline after CREATE TABLE {}", table.title),
                    })?;
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(out)
    }

    fn attribute_type_sql(
        table: &Table,
        attr_id: AttrId,
        attr: &Attribute,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError> {
        match &attr.attribute_type {
            AttributeType::Logical(dt) => Ok(sqlite_type_sql(dt)),
            AttributeType::Domain(domain_id) => {
                let Some(domain) = domains.get(*domain_id) else {
                    return Ok("NUMERIC".to_string());
                };
                Ok(sqlite_type_sql(&domain.data_type))
            }
            AttributeType::ForeignKeyAttribute(_) => {
                let (_fk, _ref_table, referenced_attr) =
                    resolve_referenced_attribute(table, attr_id, tables)?;
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

    fn render_table(
        out: &mut String,
        table: &Table,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
        used_constraints: &mut HashSet<String>,
    ) -> Result<(), SqlExportError> {
        writeln!(out, "CREATE TABLE {} (", table.title).map_err(|_| {
            SqlExportError::WriteError {
                context: format!("writing CREATE TABLE header for {}", table.title),
            }
        })?;
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
        lines.extend(crate::app::exports::sql::sql_export::render_foreign_keys(
            table,
            tables,
            used_constraints,
        )?);

        for (idx, line) in lines.iter().enumerate() {
            let comma = if idx + 1 == lines.len() { "" } else { "," };
            writeln!(out, "    {}{}", line, comma).map_err(|_| SqlExportError::WriteError {
                context: format!("writing column line for {}", table.title),
            })?;
        }
        writeln!(out, ");").map_err(|_| SqlExportError::WriteError {
            context: format!("writing end of CREATE TABLE {}", table.title),
        })?;
        Ok(())
    }

    fn render_column(
        table: &Table,
        attr_id: AttrId,
        attr: &Attribute,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
        used_constraints: &mut HashSet<String>,
    ) -> Result<String, SqlExportError> {
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
                    constraint_name_or_fallback(
                        &check.name,
                        &format!("CHK_DOMAIN_{}_{}", domain.name, idx + 1),
                    )
                },
                |check| Ok(rewrite_domain_check(&check.condition, &attr.name)),
            )?);
        }

        Ok(parts.join(" "))
    }
}

fn rewrite_domain_check(condition: &str, column_name: &str) -> String {
    condition.replace("VALUE", column_name)
}

fn sqlite_type_sql(dt: &DataType) -> String {
    if dt.type_name() == "UNKNOWN" {
        "NUMERIC".to_string()
    } else {
        dt.display_text()
    }
}
