use stella24::model::attribute::Attribute;
use stella24::model::constraints::constraint::ForeignKey;
use stella24::model::datatype::DATA_TYPES;
use stella24::model::entities::domain::Domain;
use stella24::model::entities::table::Table;

#[test]
fn table_add_and_remove_fields_updates_collection() {
    let mut table = Table::default();

    let a1 = table.attributes.insert(Attribute::default());
    let a2 = table.attributes.insert(Attribute::default());

    assert_eq!(table.fields().len(), 2);

    table.remove_field(a1);
    assert_eq!(table.fields().len(), 1);
    assert!(table.fields().get(a2).is_some());
}

#[test]
fn built_in_data_types() {
    let d1 = Domain::default();

    assert_eq!(d1.data_type.base, 1);
    assert_eq!(DATA_TYPES[1].name, "VARCHAR");
}

#[test]
fn add_and_remove_primary_key_updates_attribute_pk_flag() {
    let mut table = Table::default();
    let attr = table.attributes.insert(Attribute::default());

    table.add_pk(attr);
    assert!(table.pk.attributes.contains(&attr));
    assert!(table.attributes.get(attr).expect("attribute missing").pk);

    let removed = table.remove_pk(attr);
    assert!(removed);
    assert!(!table.pk.attributes.contains(&attr));
    assert!(!table.attributes.get(attr).expect("attribute missing").pk);
}

#[test]
fn adding_same_attribute_to_pk_twice_keeps_single_pk_member() {
    let mut table = Table::default();
    let attr = table.attributes.insert(Attribute::default());

    table.add_pk(attr);
    table.add_pk(attr);

    assert_eq!(table.pk.attributes.len(), 1);
    assert!(table.pk.attributes.contains(&attr));
}

#[test]
fn removing_same_pk_twice_returns_false_second_time() {
    let mut table = Table::default();
    let attr = table.attributes.insert(Attribute::default());

    table.add_pk(attr);
    assert!(table.remove_pk(attr));
    assert!(!table.remove_pk(attr));
}

#[test]
fn foreign_key_default_is_unbound_and_empty() {
    let fk = ForeignKey::new();
    assert!(fk.references.is_none());
    assert!(fk.local_attrs.is_empty());
}

#[test]
fn foreign_key_local_attrs_are_deduplicated() {
    let mut table = Table::default();
    let attr = table.attributes.insert(Attribute::default());

    let mut fk = ForeignKey::new();
    fk.local_attrs.insert(attr);
    fk.local_attrs.insert(attr);

    table.fks.insert(fk);
    let only_fk = table.fks.values().next().expect("fk missing");
    assert_eq!(only_fk.local_attrs.len(), 1);
}
