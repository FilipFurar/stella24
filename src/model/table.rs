use super::field::Field;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub title: String,
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
