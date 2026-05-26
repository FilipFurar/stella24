use crate::app::exports::sql::sql_export::{
    constraint_name_or_fallback,
    render_check_constraints,
    render_column_parts,
    resolve_referenced_attribute,
    sorted_attrs,
    sorted_domains,
    sorted_tables,
    validate_object_names,
    Export,
    SqlDialect,
    SqlExportError,
};
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::datatype::{DATA_TYPES, DataType};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::SlotMap;
use std::collections::HashSet;
use std::fmt::Write;

/// PostgreSQL SQL DDL exporter.
#[derive(Debug, Clone, Copy, Default)]
pub struct PostgresDialect;

impl Export for PostgresDialect {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Postgres
    }

    fn build_sql(&self, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>) -> Result<String, SqlExportError> {
        validate_object_names(tables, domains)?;

        let mut used_constraints = HashSet::new();
        let mut out = String::new();
        writeln!(out, "-- stella24 PostgreSQL SQL export").unwrap();
        writeln!(out).unwrap();

        if !domains.is_empty() {
            for (_, domain) in sorted_domains(domains) {
                render_domain(&mut out, domain, &mut used_constraints)?;
                writeln!(out).unwrap();
            }
        }

        for (_, table) in sorted_tables(tables) {
            Self::render_table(&mut out, table, tables, domains, &mut used_constraints)?;
            writeln!(out).unwrap();
        }

        Ok(out)
    }

    fn attribute_type_sql(table: &Table, attr_id: AttrId, attr: &Attribute, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>) -> Result<String, SqlExportError> {
        match &attr.attribute_type {
            AttributeType::Logical(dt) => {
                postgres_type_sql(dt, &format!("column {}.{}", table.title, attr.name))
            }
            AttributeType::Domain(domain_id) => {
                let Some(domain) = domains.get(*domain_id) else {
                    return Ok("NUMERIC".to_string());
                };
                Ok(domain.name.clone())
            }
            AttributeType::ForeignKeyAttribute(_) => {
                let (_fk, ref_table, referenced_attr) =
                    resolve_referenced_attribute(table, attr_id, tables)?;
                match &referenced_attr.attribute_type {
                    AttributeType::Logical(dt) => postgres_type_sql(
                        dt,
                        &format!("foreign key target {}.{}", ref_table.title, referenced_attr.name),
                    ),
                    AttributeType::Domain(domain_id) => {
                        let Some(domain) = domains.get(*domain_id) else {
                            return Ok("NUMERIC".to_string());
                        };
                        postgres_type_sql(
                            &domain.data_type,
                            &format!("foreign key target domain {}", domain.name),
                        )
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

    fn render_foreign_keys(table: &Table, tables: &SlotMap<TableId, Table>, domains: &SlotMap<DomainId, Domain>, used_constraints: &mut HashSet<String>) -> Result<Vec<String>, SqlExportError> {
        let _ = domains;
        crate::app::exports::sql::sql_export::render_foreign_keys(table, tables, used_constraints)
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
        postgres_type_sql(&domain.data_type, &format!("domain {}", domain.name))?,
    )
    .unwrap();

    for line in render_check_constraints(
        &domain.check_constraints,
        used_constraints,
        |_, _| format!("domain {}", domain.name),
        |idx, check| {
            constraint_name_or_fallback(&check.name, &format!("CHK_DOMAIN_{}_{}", domain.name, idx + 1))
        },
        |check| Ok(check.condition.clone()),
    )? {
        writeln!(out, "ALTER DOMAIN {} ADD {};", domain.name, line).unwrap();
    }

    Ok(())
}

fn postgres_type_sql(dt: &DataType, context: &str) -> Result<String, SqlExportError> {
    let Some(def) = DATA_TYPES.get(dt.base) else {
        return Err(SqlExportError::UnsupportedDataType {
            context: context.to_string(),
            base: dt.base,
        });
    };

    let sql = match def.name {
        "CHAR" | "NCHAR" => format!("CHAR({})", dt.params.first().copied().unwrap_or(1)),
        "VARCHAR2" | "NVARCHAR2" => {
            format!("VARCHAR({})", dt.params.first().copied().unwrap_or(1))
        }

        "NUMBER" => match dt.params.as_slice() {
            [precision, scale] => format!("NUMERIC({}, {})", precision, scale),
            [precision] => format!("NUMERIC({})", precision),
            _ => "NUMERIC".to_string(),
        },
        "FLOAT" => match dt.params.first().copied() {
            Some(p) => format!("FLOAT({})", p),
            None => "FLOAT".to_string(),
        },

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
            0 => "TIMESTAMP WITH TIME ZONE".to_string(),
            p => format!("TIMESTAMP({}) WITH TIME ZONE", p),
        },

        "INTERVAL_YEAR" | "INTERVAL_DAY" => "INTERVAL".to_string(),

        "LONG" | "NCLOB" => "TEXT".to_string(),
        "LONG RAW" | "BLOB" | "BFILE" => "BYTEA".to_string(),

        "BINARY_FLOAT" => "REAL".to_string(),
        "BINARY_DOUBLE" => "DOUBLE PRECISION".to_string(),

        "ROWID" => "TEXT".to_string(),
        "UROWID" => "TEXT".to_string(),

        other => {
            return Err(SqlExportError::UnsupportedDataType {
                context: format!("{} ({other})", context),
                base: dt.base,
            });
        }
    };

    Ok(sql)
}
