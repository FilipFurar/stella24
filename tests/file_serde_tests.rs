use egui::Rect;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use stella24::AppStella;
use stella24::model::entities::table::Table;

fn temp_file(name: &str) -> PathBuf {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("clock before unix epoch")
		.as_nanos();
	std::env::temp_dir().join(format!("stella24_{name}_{nanos}.json"))
}

#[test]
fn save_and_open_preserves_workbench_table_positions() {
	let mut app = AppStella::default();
	let table_id = app.tables.insert(Table::default());

	app.workbench_table_rects.insert(
		table_id,
		Rect::from_min_size(egui::pos2(123.0, 456.0), egui::vec2(320.0, 180.0)),
	);

	let path = temp_file("layout_roundtrip");
	app.handle_save(path.clone());

	let mut loaded = AppStella::default();
	loaded.handle_open(path.clone());

	let rect = loaded
		.workbench_table_rects
		.get(&table_id)
		.copied()
		.expect("table position not restored");

	assert_eq!(rect.min.x, 123.0);
	assert_eq!(rect.min.y, 456.0);
	assert_eq!(rect.width(), 320.0);
	assert_eq!(rect.height(), 180.0);

	let _ = fs::remove_file(path);
}

