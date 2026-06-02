use crate::AppStella;
use crate::app::{DomainId, Preferences, ProjectSettings, TableId, TableLayoutEntry};
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::fs;

const CURRENT_PROJECT_VERSION: u32 = 1;

#[derive(Serialize)]
struct ProjectFileRef<'a> {
    version: u32,
    tables: &'a SlotMap<TableId, Table>,
    domains: &'a SlotMap<DomainId, Domain>,
    domain_order: &'a [DomainId],
    settings: &'a ProjectSettings,
    preferences: &'a Preferences,
    workbench_table_layout: &'a [TableLayoutEntry],
}

#[derive(Serialize, Deserialize)]
struct ProjectFileV1 {
    version: u32,
    #[serde(default)]
    tables: SlotMap<TableId, Table>,
    #[serde(default)]
    domains: SlotMap<DomainId, Domain>,
    #[serde(default)]
    domain_order: Vec<DomainId>,
    #[serde(default)]
    settings: ProjectSettings,
    #[serde(default)]
    preferences: Preferences,
    #[serde(default)]
    workbench_table_layout: Vec<TableLayoutEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyProjectFile {
    #[serde(default)]
    tables: SlotMap<TableId, Table>,
    #[serde(default)]
    domains: SlotMap<DomainId, Domain>,
    #[serde(default)]
    domain_order: Vec<DomainId>,
    #[serde(default)]
    settings: ProjectSettings,
    #[serde(default)]
    preferences: Preferences,
    #[serde(default)]
    workbench_table_layout: Vec<TableLayoutEntry>,
}

impl<'a> From<&'a AppStella> for ProjectFileRef<'a> {
    fn from(app: &'a AppStella) -> Self {
        Self {
            version: CURRENT_PROJECT_VERSION,
            tables: &app.tables,
            domains: &app.domains,
            domain_order: &app.domain_order,
            settings: &app.settings,
            preferences: &app.preferences,
            workbench_table_layout: &app.workbench_table_layout,
        }
    }
}

impl ProjectFileV1 {
    fn into_app(self) -> AppStella {
        let mut app = AppStella {
            tables: self.tables,
            domains: self.domains,
            domain_order: self.domain_order,
            settings: self.settings,
            preferences: self.preferences,
            workbench_table_layout: self.workbench_table_layout,
            ..Default::default()
        };
        app.restore_workbench_rects_from_layout();
        app.normalize_datatypes();
        app
    }
}

impl LegacyProjectFile {
    fn into_app(self) -> AppStella {
        let mut app = AppStella {
            tables: self.tables,
            domains: self.domains,
            domain_order: self.domain_order,
            settings: self.settings,
            preferences: self.preferences,
            workbench_table_layout: self.workbench_table_layout,
            ..Default::default()
        };
        app.restore_workbench_rects_from_layout();
        app.normalize_datatypes();
        app
    }
}

fn load_project_file(json: &str) -> Option<AppStella> {
    if let Ok(project) = serde_json::from_str::<ProjectFileV1>(json)
        && project.version == CURRENT_PROJECT_VERSION
    {
        return Some(project.into_app());
    }

    serde_json::from_str::<LegacyProjectFile>(json)
        .ok()
        .map(LegacyProjectFile::into_app)
}

impl AppStella {
    /// Saves the current application state to a JSON file.
    pub fn handle_save(&mut self, path: std::path::PathBuf) {
        self.sync_layout_from_workbench_rects();
        let project = ProjectFileRef::from(&*self);
        if let Ok(json) = serde_json::to_string(&project)
            && let Err(err) = fs::write(&path, json)
        {
            eprintln!("Error saving file: {}", err);
        }
    }

    /// Loads application state from a JSON file.
    pub fn handle_open(&mut self, path: std::path::PathBuf) {
        if let Ok(json) = fs::read_to_string(path)
            && let Some(state) = load_project_file(&json)
        {
            self.tables = state.tables;
            self.domains = state.domains;
            self.workbench_table_layout = state.workbench_table_layout;
            self.restore_workbench_rects_from_layout();
            self.settings = state.settings;
            self.preferences = state.preferences;
            self.normalize_datatypes();
            self.undo_history.clear();
            self.redo_history.clear();
            self.command_queue = crate::app::CommandQueue::default();
        }
    }
}
