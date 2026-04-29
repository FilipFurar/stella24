use egui::{Pos2, Rect};
use slotmap::SlotMap;
use std::collections::HashMap;
use stella24::app::TableId;
use stella24::model::attribute::Attribute;
use stella24::model::constraints::constraint::{ForeignKey, Unique};
use stella24::model::entities::table::Table;
use stella24::ui::widgets::crow_foot::{
    CardinalityMax, CardinalityMin, RelationshipKind, build_edges,
};

fn make_rect(x: f32, y: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), egui::vec2(200.0, 120.0))
}

#[test]
fn non_identifying_optional_many_relation_is_dotted_with_zero_one_on_parent() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();

    let parent_id = tables.insert(Table {
        title: "parent".to_string(),
        ..Table::default()
    });

    let mut child = Table {
        title: "child".to_string(),
        ..Table::default()
    };
    let local_fk_attr = child.attributes.insert(Attribute {
        not_null: false,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(local_fk_attr);
    child.fks.insert(fk);

    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];
    assert_eq!(edge.kind, RelationshipKind::NonIdentifying);
    assert_eq!(edge.from_cardinality.min, CardinalityMin::Zero);
    assert_eq!(edge.from_cardinality.max, CardinalityMax::Many);
    assert_eq!(edge.to_cardinality.min, CardinalityMin::Zero);
    assert_eq!(edge.to_cardinality.max, CardinalityMax::One);
}

#[test]
fn identifying_relation_is_solid_when_fk_attribute_is_in_pk() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();

    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_attr = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });
    let other_pk_attr = child.attributes.insert(Attribute::default());
    child.pk.attributes.insert(fk_attr);
    child.pk.attributes.insert(other_pk_attr);

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);
    child.fks.insert(fk);

    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];
    assert_eq!(edge.kind, RelationshipKind::Identifying);
    assert_eq!(edge.to_cardinality.min, CardinalityMin::One);
    assert_eq!(edge.to_cardinality.max, CardinalityMax::One);
    assert_eq!(edge.from_cardinality.max, CardinalityMax::Many);
}

#[test]
fn child_side_is_one_only_when_fk_matches_unique_exactly() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();

    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_attr = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);

    child.uniques.push(Unique {
        name: "uq_fk".to_string(),
        attributes: fk.local_attrs.clone(),
    });

    child.fks.insert(fk);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];
    assert_eq!(edge.from_cardinality.min, CardinalityMin::Zero);
    assert_eq!(edge.from_cardinality.max, CardinalityMax::One);
}

#[test]
fn child_side_is_one_when_single_fk_column_has_inline_unique() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();

    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_attr = child.attributes.insert(Attribute {
        not_null: true,
        unique: true,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);

    child.fks.insert(fk);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from_cardinality.max, CardinalityMax::One);
}

#[test]
fn parent_side_is_optional_when_any_fk_column_is_nullable() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_a = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });
    let fk_b = child.attributes.insert(Attribute {
        not_null: false,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_a);
    fk.local_attrs.insert(fk_b);
    child.fks.insert(fk);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].to_cardinality.min, CardinalityMin::Zero);
}

#[test]
fn parent_side_is_mandatory_when_all_fk_columns_are_not_null() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_a = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });
    let fk_b = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_a);
    fk.local_attrs.insert(fk_b);
    child.fks.insert(fk);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].to_cardinality.min, CardinalityMin::One);
}

#[test]
fn deleted_fk_attribute_does_not_crash_edge_builder_and_becomes_optional() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_attr = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);
    child.fks.insert(fk);

    child.remove_field(fk_attr);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].to_cardinality.min, CardinalityMin::Zero);
}

#[test]
fn duplicate_unique_constraints_keep_child_side_one() {
    let mut tables: SlotMap<TableId, Table> = SlotMap::with_key();
    let parent_id = tables.insert(Table::default());

    let mut child = Table::default();
    let fk_attr = child.attributes.insert(Attribute {
        not_null: true,
        ..Attribute::default()
    });

    let mut fk = ForeignKey::new();
    fk.references = Some(parent_id);
    fk.local_attrs.insert(fk_attr);

    let attrs = fk.local_attrs.clone();
    child.uniques.push(Unique {
        name: "uq_fk_1".to_string(),
        attributes: attrs.clone(),
    });
    child.uniques.push(Unique {
        name: "uq_fk_2".to_string(),
        attributes: attrs,
    });

    child.fks.insert(fk);
    let child_id = tables.insert(child);

    let mut rects = HashMap::new();
    rects.insert(child_id, make_rect(100.0, 120.0));
    rects.insert(parent_id, make_rect(420.0, 120.0));

    let edges = build_edges(&tables, &rects);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from_cardinality.max, CardinalityMax::One);
}
