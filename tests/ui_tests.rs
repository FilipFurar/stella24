use stella24::model::attribute::Attribute;
use stella24::model::entities::table::Table;

#[test]
fn rename_table() {
    let mut tab1 = Table::default();

    let f1 = Attribute::default();
    let f2 = Attribute::default();
    let f3 = Attribute::default();

    tab1.add_field(f1);
    tab1.add_field(f2);
    tab1.add_field(f3);
}
