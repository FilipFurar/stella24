//! Oracle SQL DDL exporter (Oracle 23c+ compatible).
//!
//! Two-phase generation:
//! 1. CREATE DOMAIN + CREATE TABLE (no FKs)
//! 2. ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY

use crate::app::exports::sql::sql_export::{
    Export, SqlDialect, SqlExportError, constraint_name_or_fallback, render_check_constraints,
    render_column_parts, resolve_referenced_attribute, sorted_attrs, sorted_domains, sorted_tables,
    validate_object_names,
};
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::datatype::DataType;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::SlotMap;
use std::collections::HashSet;
use std::fmt::Write;

/// Oracle SQL DDL exporter (Oracle 23c+ compatible).
///
/// Produces Oracle-specific CREATE DOMAIN / CREATE TABLE statements and
/// emits foreign keys in a second phase via ALTER TABLE statements.
#[derive(Debug, Clone, Copy, Default)]
pub struct OracleDialect;

impl Export for OracleDialect {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Oracle
    }

    fn build_sql(
        &self,
        tables: &SlotMap<TableId, Table>,
        domains: &SlotMap<DomainId, Domain>,
    ) -> Result<String, SqlExportError> {
        validate_object_names(tables, domains)?;
        let mut used_constraints = HashSet::new();
        let mut out = String::new();
        writeln!(out, "-- stella24 Oracle SQL export").map_err(|_| SqlExportError::WriteError {
            context: "writing Oracle header".to_string(),
        })?;
        writeln!(out).map_err(|_| SqlExportError::WriteError {
            context: "writing Oracle header newline".to_string(),
        })?;

        // Phase 1a: Domains
        if !domains.is_empty() {
            for (_, domain) in sorted_domains(domains) {
                render_domain(&mut out, domain, &mut used_constraints)?;
                writeln!(out).map_err(|_| SqlExportError::WriteError {
                    context: format!("writing newline after domain {}", domain.name),
                })?;
            }
        }

        // Phase 1b: Tables (no FKs)
        for (_, table) in sorted_tables(tables) {
            Self::render_table(&mut out, table, tables, domains, &mut used_constraints)?;
            writeln!(out).map_err(|_| SqlExportError::WriteError {
                context: format!("writing newline after CREATE TABLE {}", table.title),
            })?;
        }

        // Phase 2: Foreign keys via ALTER TABLE
        for (_, table) in sorted_tables(tables) {
            let fk_lines = crate::app::exports::sql::sql_export::render_foreign_keys(
                table,
                tables,
                &mut used_constraints,
            )?;
            let fk_len = fk_lines.len();
            for fk in fk_lines {
                writeln!(out, "ALTER TABLE {} ADD {};", table.title, fk).map_err(|_| {
                    SqlExportError::WriteError {
                        context: format!("writing ALTER TABLE for {}", table.title),
                    }
                })?;
            }
            if fk_len > 0 {
                writeln!(out).map_err(|_| SqlExportError::WriteError {
                    context: format!("writing newline after ALTER TABLEs for {}", table.title),
                })?;
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
            AttributeType::Logical(dt) => {
                oracle_type_sql(dt, &format!("column {}.{}", table.title, attr.name))
            }
            AttributeType::Domain(domain_id) => {
                let Some(domain) = domains.get(*domain_id) else {
                    return Ok("NUMBER".to_string());
                };
                Ok(domain.name.clone())
            }
            AttributeType::ForeignKeyAttribute(_) => {
                let (_fk, ref_table, referenced_attr) =
                    resolve_referenced_attribute(table, attr_id, tables)?;
                match &referenced_attr.attribute_type {
                    AttributeType::Logical(dt) => oracle_type_sql(
                        dt,
                        &format!(
                            "foreign key target {}.{}",
                            ref_table.title, referenced_attr.name
                        ),
                    ),
                    AttributeType::Domain(domain_id) => {
                        let Some(domain) = domains.get(*domain_id) else {
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
        for (i, line) in lines.iter().enumerate() {
            let comma = if i + 1 == lines.len() { "" } else { "," };
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
        let parts = render_column_parts(
            table,
            attr_id,
            attr,
            tables,
            domains,
            used_constraints,
            Self::attribute_type_sql,
        )?;
        Ok(parts.join(" "))
    }
}

fn render_domain(
    out: &mut String,
    domain: &Domain,
    used_constraints: &mut HashSet<String>,
) -> Result<(), SqlExportError> {
    writeln!(
        out,
        "CREATE DOMAIN {} AS {};",
        domain.name,
        oracle_type_sql(&domain.data_type, &format!("domain {}", domain.name))?,
    )
    .map_err(|_| SqlExportError::WriteError {
        context: format!("writing CREATE DOMAIN {}", domain.name),
    })?;

    for line in render_check_constraints(
        &domain.check_constraints,
        used_constraints,
        |_, _| format!("domain {}", domain.name),
        |idx, check| {
            constraint_name_or_fallback(
                &check.name,
                &format!("CHK_DOMAIN_{}_{}", domain.name, idx + 1),
            )
        },
        |check| Ok(check.condition.clone()),
    )? {
        writeln!(out, "ALTER DOMAIN {} ADD {};", domain.name, line).map_err(|_| {
            SqlExportError::WriteError {
                context: format!("writing ALTER DOMAIN {}", domain.name),
            }
        })?;
    }

    Ok(())
}

/// Map a model `DataType` to an Oracle SQL type string.
///
/// Returns an error when the `base` index does not exist in the current
/// Oracle datatype catalog. `context` is used in the error message to aid
/// diagnostics.
fn oracle_type_sql(dt: &DataType, context: &str) -> Result<String, SqlExportError> {
    let type_name = dt.type_name();
    if type_name == "UNKNOWN" {
        return Err(SqlExportError::UnsupportedDataType {
            context: context.to_string(),
            base: dt.base,
        });
    }
    let sql = match type_name {
        // Character types
        "CHAR" => match dt.params.as_slice() {
            [size, char_semantics] => format!(
                "CHAR({} {})",
                size,
                if *char_semantics == 1 { "CHAR" } else { "BYTE" }
            ),
            [size] => format!("CHAR({})", size),
            _ => "CHAR(1)".to_string(),
        },
        "NCHAR" => format!("NCHAR({})", dt.params.first().copied().unwrap_or(1)),
        "VARCHAR2" => match dt.params.as_slice() {
            [size, char_semantics] => format!(
                "VARCHAR2({} {})",
                size,
                if *char_semantics == 1 { "CHAR" } else { "BYTE" }
            ),
            [size] => format!("VARCHAR2({})", size),
            _ => "VARCHAR2(1)".to_string(),
        },
        "NVARCHAR2" => format!("NVARCHAR2({})", dt.params.first().copied().unwrap_or(1)),

        // Numeric types
        "NUMBER" => match dt.params.as_slice() {
            [precision, scale] => format!("NUMBER({}, {})", precision, scale),
            [precision] => format!("NUMBER({})", precision),
            _ => "NUMBER".to_string(),
        },
        "FLOAT" => format!("FLOAT({})", dt.params.first().copied().unwrap_or(126)),

        // Date/time types
        "DATE" => "DATE".to_string(),
        "TIMESTAMP" => match dt.params.first().copied().unwrap_or(6) {
            0 => "TIMESTAMP".to_string(),
            p => format!("TIMESTAMP({})", p),
        },
        "TIMESTAMP WITH TIME ZONE" => match dt.params.first().copied().unwrap_or(6) {
            0 => "TIMESTAMP WITH TIME ZONE".to_string(),
            p => format!("TIMESTAMP({}) WITH TIME ZONE", p),
        },
        "TIMESTAMP WITH LOCAL TIME ZONE" => match dt.params.first().copied().unwrap_or(6) {
            0 => "TIMESTAMP WITH LOCAL TIME ZONE".to_string(),
            p => format!("TIMESTAMP({}) WITH LOCAL TIME ZONE", p),
        },

        // Interval types
        "INTERVAL YEAR TO MONTH" => format!(
            "INTERVAL YEAR({}) TO MONTH",
            dt.params.first().copied().unwrap_or(2)
        ),
        "INTERVAL DAY TO SECOND" => match dt.params.as_slice() {
            [day_precision, second_precision] => format!(
                "INTERVAL DAY({}) TO SECOND({})",
                day_precision, second_precision
            ),
            [day_precision] => format!("INTERVAL DAY({}) TO SECOND", day_precision),
            _ => "INTERVAL DAY TO SECOND".to_string(),
        },

        // LOB and raw types
        "LONG" => "LONG".to_string(),
        "LONG RAW" => "LONG RAW".to_string(),
        "NCLOB" => "NCLOB".to_string(),
        "BLOB" => "BLOB".to_string(),
        "BFILE" => "BFILE".to_string(),

        // Binary float types
        "BINARY_FLOAT" => "BINARY_FLOAT".to_string(),
        "BINARY_DOUBLE" => "BINARY_DOUBLE".to_string(),

        // Rowid types
        "ROWID" => "ROWID".to_string(),
        "UROWID" => match dt.params.first().copied() {
            Some(size) => format!("UROWID({})", size),
            None => "UROWID".to_string(),
        },

        other => {
            return Err(SqlExportError::UnsupportedDataType {
                context: format!("{} ({other})", context),
                base: dt.base,
            });
        }
    };
    Ok(sql)
}
