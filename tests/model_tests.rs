use stella24::model::field::Field;
use stella24::model::table::Table;

#[test]
fn table_fields() {
    let mut tab1 = Table::default();

    let f1 = Field::default();
    let f2 = Field::default();
    let f3 = Field::default();


    tab1.add_field(f1);
    tab1.add_field(f2);
    tab1.add_field(f3);

    assert_eq!(tab1.fields().len(), 3);
    assert!(!tab1.fields().get(0).unwrap().nullable)
}

#[test]
fn it_works() {

}