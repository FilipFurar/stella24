use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use stella24::app::exports::svg_export::{SvgExportOptions, SvgLayoutMode, SvgThemeChoice};
use stella24::app::TableId;
use stella24::AppStella;
use egui::Rect;

fn temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("stella24_{name}_{nanos}.json"))
}

#[test]
fn svg_export_uses_workbench_relation_notation() {
    let input_path = temp_file("input");
    let output_path = temp_file("output").with_extension("svg");

    let state = json!({
        "tables": [
            {"value": null, "version": 0},
            {
                "value": {
                    "title": "child",
                    "attributes": [
                        {"value": null, "version": 0},
                        {"value": {"name": "id", "attribute_type": {"Logical": {"base": 0, "params": [1]}}, "pk": true, "not_null": true, "unique": true}, "version": 1},
                        {"value": {"name": "parent_id", "attribute_type": {"ForeignKeyAttribute": {"idx": 1, "version": 1}}, "pk": false, "not_null": true, "unique": false}, "version": 1}
                    ],
                    "pk": {"name": "primary_key", "attributes": [{"idx": 1, "version": 1}]},
                    "fks": [
                        {"value": null, "version": 0},
                        {"value": {"name": "fk_parent", "references": {"idx": 2, "version": 1}, "local_attrs": [{"idx": 2, "version": 1}]}, "version": 1}
                    ],
                    "uniques": [],
                    "not_nulls": []
                },
                "version": 1
            },
            {
                "value": {
                    "title": "parent",
                    "attributes": [
                        {"value": null, "version": 0},
                        {"value": {"name": "id", "attribute_type": {"Logical": {"base": 0, "params": [1]}}, "pk": true, "not_null": true, "unique": true}, "version": 1}
                    ],
                    "pk": {"name": "primary_key", "attributes": [{"idx": 1, "version": 1}]},
                    "fks": [{"value": null, "version": 0}],
                    "uniques": [],
                    "not_nulls": []
                },
                "version": 1
            }
        ],
        "domains": [{"value": null, "version": 0}]
    });

    fs::write(&input_path, state.to_string()).expect("failed to write input state");

    let mut app = AppStella::default();
    app.handle_open(input_path.clone());
    app.to_svg(output_path.to_str().expect("invalid output path"));

    let svg = fs::read_to_string(&output_path).expect("failed to read exported svg");
    assert!(svg.contains("<svg "));
    assert!(svg.contains("data-kind=\"non-identifying\""));
    assert!(svg.contains("stroke-dasharray=\"4 6\""));
    assert!(svg.contains("data-from-cardinality=\"0..N\""));
    assert!(svg.contains("data-to-cardinality=\"1..1\""));
    assert!(svg.contains("Table constraints"));
    assert!(svg.contains("FK fk_parent (parent_id) -&gt; parent"));

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn svg_export_can_use_workbench_layout_rects() {
    let input_path = temp_file("input_layout");

    let state = json!({
        "tables": [
            {"value": null, "version": 0},
            {
                "value": {
                    "title": "child",
                    "attributes": [
                        {"value": null, "version": 0},
                        {"value": {"name": "id", "attribute_type": {"Logical": {"base": 0, "params": [1]}}, "pk": true, "not_null": true, "unique": true}, "version": 1}
                    ],
                    "pk": {"name": "primary_key", "attributes": [{"idx": 1, "version": 1}]},
                    "fks": [{"value": null, "version": 0}],
                    "uniques": [],
                    "not_nulls": []
                },
                "version": 1
            },
            {
                "value": {
                    "title": "parent",
                    "attributes": [
                        {"value": null, "version": 0},
                        {"value": {"name": "id", "attribute_type": {"Logical": {"base": 0, "params": [1]}}, "pk": true, "not_null": true, "unique": true}, "version": 1}
                    ],
                    "pk": {"name": "primary_key", "attributes": [{"idx": 1, "version": 1}]},
                    "fks": [{"value": null, "version": 0}],
                    "uniques": [],
                    "not_nulls": []
                },
                "version": 1
            }
        ],
        "domains": [{"value": null, "version": 0}]
    });

    fs::write(&input_path, state.to_string()).expect("failed to write input state");

    let mut app = AppStella::default();
    app.handle_open(input_path.clone());

    let mut by_title: HashMap<String, TableId> = HashMap::new();
    for (id, table) in app.tables() {
        by_title.insert(table.title.clone(), id);
    }

    let mut rects = HashMap::new();
    rects.insert(
        *by_title.get("child").expect("child table missing"),
        Rect::from_min_size(egui::pos2(100.0, 120.0), egui::vec2(300.0, 140.0)),
    );
    rects.insert(
        *by_title.get("parent").expect("parent table missing"),
        Rect::from_min_size(egui::pos2(640.0, 280.0), egui::vec2(300.0, 140.0)),
    );

    let svg = app.svg_string_with_options(
        SvgExportOptions {
            layout: SvgLayoutMode::Workbench,
            theme: SvgThemeChoice::Dark,
        },
        Some(&rects),
        true,
    );

    assert!(svg.contains("x=\"100\""));
    assert!(svg.contains("y=\"120\""));
    assert!(svg.contains("x=\"640\""));
    assert!(svg.contains("y=\"280\""));

    let _ = fs::remove_file(input_path);
}

#[test]
fn svg_export_theme_choice_changes_table_colors() {
    let input_path = temp_file("input_theme");
    let state = json!({
        "tables": [
            {"value": null, "version": 0},
            {
                "value": {
                    "title": "theme_table",
                    "attributes": [
                        {"value": null, "version": 0},
                        {"value": {"name": "id", "attribute_type": {"Logical": {"base": 0, "params": [1]}}, "pk": true, "not_null": true, "unique": true}, "version": 1}
                    ],
                    "pk": {"name": "primary_key", "attributes": [{"idx": 1, "version": 1}]},
                    "fks": [{"value": null, "version": 0}],
                    "uniques": [],
                    "not_nulls": []
                },
                "version": 1
            }
        ],
        "domains": [{"value": null, "version": 0}]
    });

    fs::write(&input_path, state.to_string()).expect("failed to write input state");
    let mut app = AppStella::default();
    app.handle_open(input_path.clone());

    let dark = app.svg_string_with_options(
        SvgExportOptions {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Dark,
        },
        None,
        true,
    );
    let light = app.svg_string_with_options(
        SvgExportOptions {
            layout: SvgLayoutMode::Automatic,
            theme: SvgThemeChoice::Light,
        },
        None,
        true,
    );

    assert!(dark.contains("fill=\"#2b2b2b\""));
    assert!(light.contains("fill=\"#f7f7f7\""));
    assert!(light.contains("stroke=\"#9b9b9b\""));

    let _ = fs::remove_file(input_path);
}


