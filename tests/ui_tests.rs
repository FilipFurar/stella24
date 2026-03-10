use stella24::model::constraints::field::Field;
use stella24::model::table::Table;

#[test]
fn rename_table() {
    let mut tab1 = Table::default();

    let f1 = Field::default();
    let f2 = Field::default();
    let f3 = Field::default();

    tab1.add_field(f1);
    tab1.add_field(f2);
    tab1.add_field(f3);
}
