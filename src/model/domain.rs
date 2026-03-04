use super::datatype::DataType;

/// SQL Domain
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Domain {
    pub name: String,
    pub data_type: DataType,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            name: "Domain".to_string(),
            data_type: DataType {
                base: 1,
                params: vec![5],
            },
        }
    }
}
