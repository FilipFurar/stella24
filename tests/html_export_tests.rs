use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use stella24::AppStella;

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

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}


