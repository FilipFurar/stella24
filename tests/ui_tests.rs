use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::cell::RefCell;
use std::rc::Rc;
use stella24::model::attribute::Attribute;
use stella24::model::constraints::constraint::ForeignKey;
use stella24::model::entities::table::Table;

#[test]
fn checkbox_click_toggles_state() {
    let checked = Rc::new(RefCell::new(false));
    let checked_for_ui = Rc::clone(&checked);

    let mut harness = Harness::new_ui(move |ui| {
        let mut value = *checked_for_ui.borrow();
        if ui.checkbox(&mut value, "NN").changed() {
            *checked_for_ui.borrow_mut() = value;
        }
    });

    harness.get_by_label("NN").click();
    harness.run();

    assert!(*checked.borrow());
}

#[test]
fn button_click_invokes_action() {
    let pressed = Rc::new(RefCell::new(0usize));
    let pressed_for_ui = Rc::clone(&pressed);

    let mut harness = Harness::new_ui(move |ui| {
        if ui.button("Add").clicked() {
            *pressed_for_ui.borrow_mut() += 1;
        }
    });

    harness.get_by_label("Add").click();
    harness.run();

    assert_eq!(*pressed.borrow(), 1);
}

#[test]
fn deleting_fk_from_ui_removes_fk_and_its_local_attributes() {
    let table = Rc::new(RefCell::new(Table::default()));
    let table_for_ui = Rc::clone(&table);

    let fk_attr = {
        let mut t = table.borrow_mut();
        let fk_attr = t.attributes.insert(Attribute::default());

        let mut fk = ForeignKey::new();
        fk.local_attrs.insert(fk_attr);
        t.fks.insert(fk);
        fk_attr
    };

    let mut harness = Harness::new_ui(move |ui| {
        table_for_ui.borrow_mut().draw_fks(ui);
    });

    harness.get_by_label("🗑").click();
    harness.run();

    let t = table.borrow();
    assert!(t.fks.is_empty());
    assert!(t.attributes.get(fk_attr).is_none());
}
