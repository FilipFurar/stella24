use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use slotmap::SlotMap;
use std::cell::RefCell;
use std::rc::Rc;
use stella24::app::TableId;
use stella24::app::exports::sql::sql_export::SqlDialect;
use stella24::model::attribute::Attribute;
use stella24::model::constraints::constraint::ForeignKey;
use stella24::model::constraints::constraint::Unique;
use stella24::model::entities::table::Table;
use stella24::ui::context::TableUiContext;
use stella24::ui::entities::table::TableChanges;

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

    let mut tables_map: SlotMap<TableId, Table> = SlotMap::with_key();
    let table_id = tables_map.insert(Table::default());

    let mut harness = Harness::new_ui(move |ui| {
        table_for_ui.borrow_mut().draw_fks(ui, table_id);
    });

    harness.get_by_label("🗑").click();
    harness.run();

    let t = table.borrow();
    assert!(t.fks.is_empty());
    assert!(t.attributes.get(fk_attr).is_none());
}

#[test]
fn deleting_unique_from_ui_removes_unique_and_unsets_attribute_flag() {
    let table = Rc::new(RefCell::new(Table::default()));
    let table_for_ui = Rc::clone(&table);

    let attr_id = {
        let mut t = table.borrow_mut();
        t.attributes.insert(Attribute::default())
    };

    // add a unique that references the attribute
    {
        let mut t = table.borrow_mut();
        let mut uq = Unique::new();
        uq.attributes.insert(attr_id);
        t.add_unique(uq);
    }

    // create a dummy tables snapshot to produce a TableUiContext and a TableId
    let mut tables_map: SlotMap<TableId, Table> = SlotMap::with_key();
    let table_id = tables_map.insert(Table::default());

    let mut harness = Harness::new_ui(move |ui| {
        // draw only uniques so the delete button corresponds to unique
        table_for_ui.borrow_mut().draw_uniques(ui, table_id);
    });

    harness.get_by_label("🗑").click();
    harness.run();

    let t = table.borrow();
    assert!(t.uniques.is_empty());
    assert!(t.attributes.get(attr_id).is_some());
    assert!(!t.attributes.get(attr_id).unwrap().unique);
}

#[test]
fn clicking_add_button_sets_add_attribute_change() {
    let mut tables_map: SlotMap<TableId, Table> = SlotMap::with_key();
    let table_id = tables_map.insert(Table::default());

    let ctx = TableUiContext::from_app(
        &tables_map,
        &SlotMap::with_key(),
        table_id,
        SqlDialect::Oracle,
    );

    let table = Rc::new(RefCell::new(Table::default()));
    let table_for_ui = Rc::clone(&table);

    let result: Rc<RefCell<Option<TableChanges>>> = Rc::new(RefCell::new(None));
    let result_for_ui = Rc::clone(&result);

    let mut harness = Harness::new_ui(move |ui| {
        let changes = table_for_ui.borrow_mut().draw(ui, &ctx, table_id);
        if changes.add_attribute {
            *result_for_ui.borrow_mut() = Some(changes);
        }
    });

    harness.get_by_label("Add").click();
    harness.run();

    let changes = result.borrow();
    assert!(changes.is_some());
    assert!(changes.as_ref().unwrap().add_attribute);
}
