// model/entities/domain.rs

use super::super::datatype::DataType;
use crate::model::constraints::check::Check;

/// SQL Domain
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Domain {
    pub name: String,

    #[serde(default)]
    pub data_type: DataType,

    #[serde(default)]
    pub check_constraints: Vec<Check>,
}

impl Default for Domain {
    fn default() -> Self {
        let mut data_type = DataType {
            base: 1,
            params: vec![5],
        };
        data_type.normalize_params();

        Self {
            name: "Domain".to_string(),
            data_type,
            check_constraints: vec![],
        }
    }
}
