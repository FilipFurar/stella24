// model/entities/domain.rs

use crate::model::constraints::check::Check;
use super::super::datatype::DataType;

/// SQL Domain
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Domain {
    pub name: String,
    pub data_type: DataType,
    #[serde(default)]
    pub check_constraints: Vec<Check>,
    pub not_null: bool,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            name: "Domain".to_string(),
            data_type: DataType {
                base: 1,
                params: vec![5],
            },
            check_constraints: vec![],
            not_null: false,
        }
    }
}
