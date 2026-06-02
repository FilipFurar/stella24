use slotmap::SlotMap;
use stella24::app::exports::sql::sql_export::SqlDialect::Oracle;
use stella24::app::exports::sql::sql_export::{SqlDialect, SqlExportError, build_sql};
use stella24::app::{DomainId, TableId};
use stella24::model::attribute::{Attribute, AttributeType};
use stella24::model::constraints::check::Check;
use stella24::model::constraints::constraint::{ForeignKey, PrimaryKey};
use stella24::model::datatype::DataType;
use stella24::model::entities::domain::Domain;
use stella24::model::entities::table::Table;

fn mk_table(title: &str) -> Table {
    Table {
        title: title.to_string(),
        attributes: SlotMap::with_key(),
        pk: PrimaryKey::new(),
        fks: SlotMap::with_key(),
        uniques: vec![],
        not_nulls: vec![],
        checks: vec![],
        open_modal: false,
        current_fk: None,
        current_unique: None,
        attr_order: vec![],
        dragged_attr: None,
        dragged_from_index: None,
        is_being_dragged: false,
    }
}
#[test]
fn exports_sql_and_inlines_domain_checks_and_fk() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let mut domains: SlotMap<DomainId, Domain> = SlotMap::with_key();
    let dom_id = domains.insert(Domain {
        name: "dom_code".to_string(),
        data_type: DataType {
            dialect: SqlDialect::Oracle,
            base: 1,
            params: vec![10],
        },
        check_constraints: vec![Check {
            name: "ck_dom".to_string(),
            condition: "VALUE <> ''".to_string(),
        }],
    });
    let mut parent = mk_table("parent");
    let parent_attr = parent.attributes.insert(Attribute {
        name: "id".to_string(),
        attribute_type: AttributeType::Logical(DataType {
            dialect: SqlDialect::Oracle,
            base: 4,
            params: vec![5, 0],
        }),
        pk: true,
        not_null: true,
        unique: true,
    });
    parent.pk.attributes.insert(parent_attr);
    let parent_id = tables.insert(parent);
    let mut child = mk_table("child");
    let fk_attr = child.attributes.insert(Attribute {
        name: "parent_id".to_string(),
        attribute_type: AttributeType::ForeignKeyAttribute(parent_attr),
        pk: false,
        not_null: true,
        unique: false,
    });
    let mut fk = ForeignKey::new();
    fk.name = "fk_child_parent".to_string();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);
    child.fks.insert(fk);
    child.attributes.insert(Attribute {
        name: "code".to_string(),
        attribute_type: AttributeType::Domain(dom_id),
        pk: false,
        not_null: false,
        unique: false,
    });
    tables.insert(child);
    let sql = build_sql(SqlDialect::Oracle, &tables, &domains).expect("sql export");
    assert!(sql.contains("CREATE DOMAIN dom_code AS VARCHAR2(10)"));
    assert!(sql.contains("CONSTRAINT ck_dom CHECK (VALUE <> '')"));
    assert!(sql.contains("CREATE TABLE child"));
    assert!(sql.contains("code dom_code"));
    assert!(!sql.contains("CREATE DOMAIN '"));
    assert!(!sql.contains("CREATE TABLE '"));
    assert!(!sql.contains("CONSTRAINT '"));
    assert!(!sql.contains("REFERENCES '"));
    assert!(!sql.contains("DOMCHK_"));
}

#[test]
fn exports_sqlite_sql_with_inline_domain_checks_and_foreign_keys() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let mut domains: SlotMap<DomainId, Domain> = SlotMap::with_key();
    let dom_id = domains.insert(Domain {
        name: "dom_code".to_string(),
        data_type: DataType {
            dialect: SqlDialect::Sqlite,
            base: 1,
            params: vec![10],
        },
        check_constraints: vec![Check {
            name: "ck_dom".to_string(),
            condition: "VALUE <> ''".to_string(),
        }],
    });

    let mut parent = mk_table("parent");
    let parent_attr = parent.attributes.insert(Attribute {
        name: "id".to_string(),
        attribute_type: AttributeType::Logical(DataType {
            dialect: SqlDialect::Sqlite,
            base: 7,
            params: vec![],
        }),
        pk: true,
        not_null: true,
        unique: true,
    });
    parent.pk.attributes.insert(parent_attr);
    let parent_id = tables.insert(parent);

    let mut child = mk_table("child");
    let fk_attr = child.attributes.insert(Attribute {
        name: "parent_id".to_string(),
        attribute_type: AttributeType::ForeignKeyAttribute(parent_attr),
        pk: false,
        not_null: true,
        unique: false,
    });
    let mut fk = ForeignKey::new();
    fk.name = "fk_child_parent".to_string();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);
    child.fks.insert(fk);
    child.attributes.insert(Attribute {
        name: "code".to_string(),
        attribute_type: AttributeType::Domain(dom_id),
        pk: false,
        not_null: false,
        unique: false,
    });
    tables.insert(child);

    let sql = build_sql(SqlDialect::Sqlite, &tables, &domains).expect("sqlite export");
    assert!(sql.contains("PRAGMA foreign_keys = ON;"));
    assert!(sql.contains("CREATE TABLE child"));
    // SQLite may use TEXT or VARCHAR depending on the domain catalog; accept both.
    assert!(sql.contains("code TEXT") || sql.contains("code VARCHAR(10)"));
    assert!(sql.contains("CHECK (code <> '')"));
    assert!(sql.contains("FOREIGN KEY (parent_id) REFERENCES parent (id)"));
    assert!(!sql.contains("CREATE DOMAIN"));
}

#[test]
fn exports_postgres_sql_with_domains_and_foreign_keys() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let mut domains: SlotMap<DomainId, Domain> = SlotMap::with_key();
    let dom_id = domains.insert(Domain {
        name: "dom_code".to_string(),
        data_type: DataType {
            dialect: SqlDialect::Postgres,
            base: 1,
            params: vec![10],
        },
        check_constraints: vec![Check {
            name: "ck_dom".to_string(),
            condition: "VALUE <> ''".to_string(),
        }],
    });

    let mut parent = mk_table("parent");
    let parent_attr = parent.attributes.insert(Attribute {
        name: "id".to_string(),
        attribute_type: AttributeType::Logical(DataType {
            dialect: SqlDialect::Postgres,
            base: 5,
            params: vec![],
        }),
        pk: true,
        not_null: true,
        unique: true,
    });
    parent.pk.attributes.insert(parent_attr);
    let parent_id = tables.insert(parent);

    let mut child = mk_table("child");
    let fk_attr = child.attributes.insert(Attribute {
        name: "parent_id".to_string(),
        attribute_type: AttributeType::ForeignKeyAttribute(parent_attr),
        pk: false,
        not_null: true,
        unique: false,
    });
    let mut fk = ForeignKey::new();
    fk.name = "fk_child_parent".to_string();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);
    child.fks.insert(fk);
    child.attributes.insert(Attribute {
        name: "code".to_string(),
        attribute_type: AttributeType::Domain(dom_id),
        pk: false,
        not_null: false,
        unique: false,
    });
    tables.insert(child);

    let sql = build_sql(SqlDialect::Postgres, &tables, &domains).expect("postgres export");
    assert!(sql.contains("-- stella24 PostgreSQL SQL export"));
    assert!(sql.contains("CREATE DOMAIN dom_code AS VARCHAR(10);"));
    assert!(sql.contains("ALTER DOMAIN dom_code ADD CONSTRAINT ck_dom CHECK (VALUE <> '');"));
    assert!(sql.contains("CREATE TABLE child"));
    assert!(sql.contains("code dom_code"));
    assert!(sql.contains("FOREIGN KEY (parent_id) REFERENCES parent (id)"));
}

#[test]
fn rejects_duplicate_table_names() {
    let tables: SlotMap<TableId, Table> = {
        let mut t = SlotMap::with_key();
        t.insert(mk_table("dup"));
        t.insert(mk_table("dup"));
        t
    };
    let domains: SlotMap<DomainId, Domain> = SlotMap::with_key();
    let err = build_sql(SqlDialect::Oracle, &tables, &domains).unwrap_err();
    assert!(matches!(err, SqlExportError::DuplicateTableName { .. }));
}

#[test]
fn generates_unique_primary_key_constraint_names_per_table() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let domains: SlotMap<DomainId, Domain> = SlotMap::with_key();

    let mut t1 = mk_table("first");
    let a1 = t1.attributes.insert(Attribute {
        name: "id".to_string(),
        attribute_type: AttributeType::Logical(DataType {
            dialect: SqlDialect::Oracle,
            base: 4,
            params: vec![5, 0],
        }),
        pk: true,
        not_null: true,
        unique: true,
    });
    t1.pk.attributes.insert(a1);
    tables.insert(t1);

    let mut t2 = mk_table("second");
    let a2 = t2.attributes.insert(Attribute {
        name: "id".to_string(),
        attribute_type: AttributeType::Logical(DataType {
            dialect: SqlDialect::Oracle,
            base: 4,
            params: vec![5, 0],
        }),
        pk: true,
        not_null: true,
        unique: true,
    });
    t2.pk.attributes.insert(a2);
    tables.insert(t2);

    let sql = build_sql(Oracle, &tables, &domains).unwrap();
    assert!(sql.contains("CONSTRAINT PK_first PRIMARY KEY"));
    assert!(sql.contains("CONSTRAINT PK_second PRIMARY KEY"));
}
