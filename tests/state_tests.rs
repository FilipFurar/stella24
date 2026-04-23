use stella24::app::Command;
use stella24::AppStella;
use stella24::model::attribute::Attribute;
use stella24::model::entities::table::Table;

#[test]
fn can_make_former_pk_attribute_nullable_when_pk_removed_first() {
    let mut app = AppStella::default();
    let table_id = app.tables.insert(Table::default());

    let attr_id = {
        let table = app.tables.get_mut(table_id).expect("table missing");
        let attr_id = table.attributes.insert(Attribute {
            pk: true,
            not_null: true,
            ..Attribute::default()
        });
        table.pk.attributes.insert(attr_id);
        attr_id
    };

    app.dispatch(Command::SetAttributePrimaryKey {
        table: table_id,
        attr: attr_id,
        value: false,
    });
    app.dispatch(Command::SetAttributeNotNull {
        table: table_id,
        attr: attr_id,
        value: false,
    });
    app.flush_commands();

    let attr = app
        .tables
        .get(table_id)
        .and_then(|t| t.attributes.get(attr_id))
        .expect("attribute missing");
    assert!(!attr.pk);
    assert!(!attr.not_null);
}

#[test]
fn setting_pk_does_not_force_unique() {
    let mut app = AppStella::default();
    let table_id = app.tables.insert(Table::default());
    let attr_id = {
        let table = app.tables.get_mut(table_id).expect("table missing");
        table.attributes.insert(Attribute::default())
    };

    app.dispatch(Command::SetAttributePrimaryKey {
        table: table_id,
        attr: attr_id,
        value: true,
    });
    app.flush_commands();

    let attr = app
        .tables
        .get(table_id)
        .and_then(|t| t.attributes.get(attr_id))
        .expect("attribute missing");
    assert!(attr.pk);
    assert!(attr.not_null);
    assert!(!attr.unique);
}

#[test]
fn can_clear_unique_on_pk_attribute() {
    let mut app = AppStella::default();
    let table_id = app.tables.insert(Table::default());
    let attr_id = {
        let table = app.tables.get_mut(table_id).expect("table missing");
        table.attributes.insert(Attribute {
            unique: true,
            ..Attribute::default()
        })
    };

    app.dispatch(Command::SetAttributePrimaryKey {
        table: table_id,
        attr: attr_id,
        value: true,
    });
    app.dispatch(Command::SetAttributeUnique {
        table: table_id,
        attr: attr_id,
        value: false,
    });
    app.flush_commands();

    let attr = app
        .tables
        .get(table_id)
        .and_then(|t| t.attributes.get(attr_id))
        .expect("attribute missing");
    assert!(attr.pk);
    assert!(!attr.unique);
}