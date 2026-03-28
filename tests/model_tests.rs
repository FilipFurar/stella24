use stella24::model::datatype::DATA_TYPES;
use stella24::model::entities::domain::Domain;
use stella24::model::field::Attribute;
use stella24::model::entities::table::Table;

#[test]
fn table_fields() {
    let mut tab1 = Table::default();

    let f1 = Attribute::default();
    let f2 = Attribute::default();
    let f3 = Attribute::default();

    tab1.add_field(f1);
    tab1.add_field(f2);
    tab1.add_field(f3);

    assert_eq!(tab1.fields().len(), 3);
    //assert!(!tab1.fields().get(0).unwrap().nullable)
}

#[test]
fn built_in_data_types() {
    let d1 = Domain::default();

    println!("{:?}", d1);
    println!("base 1: {}", DATA_TYPES[1].name);

    assert_eq!(d1.data_type.base, 1);
    assert_eq!(DATA_TYPES[1].name, "VARCHAR");
}
