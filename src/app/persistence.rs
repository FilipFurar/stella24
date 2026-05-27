use crate::AppStella;
use std::fs;

impl AppStella {
    /// Saves the current application state to a JSON file.
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        self.sync_layout_from_workbench_rects();
        if let Ok(json) = serde_json::to_string(&self)
            && let Err(err) = fs::write(&path, json)
        {
            eprintln!("Error saving file: {}", err);
        }
    }

    /// Loads application state from a JSON file.
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path)
            && let Ok(state) = serde_json::from_str::<AppStella>(&json)
        {
            self.tables = state.tables;
            self.domains = state.domains;
            self.workbench_table_layout = state.workbench_table_layout;
            self.restore_workbench_rects_from_layout();
            self.selected_sql_dialect = state.selected_sql_dialect;
            self.undo_history.clear();
            self.redo_history.clear();
            self.command_queue = crate::app::CommandQueue::default();
        }
    }
}
