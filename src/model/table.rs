use super::field::Field;

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,
    /// Table rows
    pub fields: Vec<Field>,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            fields: vec![],
        }
    }
}
