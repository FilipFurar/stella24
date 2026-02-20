use super::datatype::FieldType;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Domain {
    pub title: String,
    pub field_type: FieldType,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            title: "Domain".to_string(),
            field_type: FieldType {
                base: 1,
                params: vec![5],
            },
        }
    }
}
