//! SQLite SQL DDL exporter.

use crate::app::exports::sql::sql_export::{SqlDialect, Export, SqlExportError};
use crate::app::{DomainId, TableId};
use crate::model::attribute::{AttrId, Attribute, AttributeType};
use crate::model::constraints::constraint::ForeignKey;
use crate::model::datatype::DataType;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use slotmap::{Key, SlotMap};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

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
        build_sqlite_sql(tables, domains)
    }
}

pub fn build_sqlite_sql(
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
        render_table(&mut out, table, tables, domains, &mut used_constraints)?;
        writeln!(out).unwrap();
    }

    Ok(out)
}

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

fn render_table(
    out: &mut String,
    table: &Table,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
) -> Result<(), SqlExportError> {
    writeln!(out, "CREATE TABLE {} (", table.title).unwrap();
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

    lines.extend(render_table_constraints(table, used_constraints)?);
    lines.extend(render_foreign_key_constraints(
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

fn render_column(
    table: &Table,
    attr_id: AttrId,
    attr: &Attribute,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
) -> Result<String, SqlExportError> {
    let mut parts = vec![
        attr.name.clone(),
        attribute_type_sql(table, attr_id, attr, tables, domains)?,
    ];

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

    if let AttributeType::Domain(domain_id) = &attr.attribute_type
        && let Some(domain) = domains.get(*domain_id)
    {
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
            parts.push(format!(
                "CONSTRAINT {} CHECK ({})",
                name,
                rewrite_domain_check(&check.condition, &attr.name)
            ));
        }
    }

    Ok(parts.join(" "))
}

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

fn render_foreign_key_constraints(
    table: &Table,
    tables: &SlotMap<TableId, Table>,
    domains: &SlotMap<DomainId, Domain>,
    used_constraints: &mut HashSet<String>,
) -> Result<Vec<String>, SqlExportError> {
    let mut lines = Vec::new();
    for fk in table.fks.values() {
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

        let mut ref_to_local = HashMap::new();
        for &local_id in &fk.local_attrs {
            let Some((_, ref_id)) = fk_attr_target(table, local_id)? else {
                return Err(SqlExportError::MissingForeignKeyConstraint {
                    table: table.title.clone(),
                    column: table.attributes[local_id].name.clone(),
                });
            };
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

        let _ = domains; // kept for symmetry with other render helpers
    }
    Ok(lines)
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
            let (fk, ref_table, ref_attr_id) = resolve_fk_attr(table, attr_id, tables)?;
            let Some(referenced_attr) = ref_table.attributes.get(ref_attr_id) else {
                return Err(SqlExportError::MissingReferencedColumn {
                    table: table.title.clone(),
                    foreign_key: fk.name.clone(),
                    referenced_table: ref_table.title.clone(),
                });
            };
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

fn rewrite_domain_check(condition: &str, column_name: &str) -> String {
    condition.replace("VALUE", column_name)
}

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

fn exact_pk_name(table: &Table, attr_id: AttrId) -> Option<String> {
    let pk_attrs = sorted_attr_ids(table, &table.pk.attributes);
    (pk_attrs.len() == 1 && pk_attrs[0] == attr_id)
        .then(|| constraint_name_or_fallback(&table.pk.name, &format!("PK_{}", table.title)))
}

fn exact_unique_name(table: &Table, attr_id: AttrId) -> Option<String> {
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

fn exact_not_null_name(table: &Table, attr_id: AttrId) -> Option<String> {
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

fn sorted_tables(tables: &SlotMap<TableId, Table>) -> Vec<(TableId, &Table)> {
    let mut out: Vec<_> = tables.iter().collect();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.title
            .cmp(&b.title)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

fn sorted_domains(domains: &SlotMap<DomainId, Domain>) -> Vec<(DomainId, &Domain)> {
    let mut out: Vec<_> = domains.iter().collect();
    out.sort_by(|(a_id, a), (b_id, b)| {
        a.name
            .cmp(&b.name)
            .then_with(|| a_id.data().as_ffi().cmp(&b_id.data().as_ffi()))
    });
    out
}

fn sorted_attrs(table: &Table) -> Vec<(AttrId, &Attribute)> {
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

fn sorted_attr_ids(table: &Table, set: &HashSet<AttrId>) -> Vec<AttrId> {
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

fn join_attr_names(table: &Table, attrs: &[AttrId]) -> String {
    attrs
        .iter()
        .map(|id| table.attributes[*id].name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

fn referenced_columns_decl(cols: Vec<String>) -> String {
    format!("({})", cols.join(", "))
}

fn ensure_name(kind: &'static str, name: &str) -> Result<(), SqlExportError> {
    if name.trim().is_empty() {
        return Err(SqlExportError::EmptyName { kind });
    }
    Ok(())
}

fn add_constraint_name(used: &mut HashSet<String>, name: &str) -> Result<(), SqlExportError> {
    if !used.insert(name.to_string()) {
        return Err(SqlExportError::DuplicateConstraintName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn constraint_name_or_fallback(name: &str, fallback: &str) -> String {
    if name.trim().is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}
