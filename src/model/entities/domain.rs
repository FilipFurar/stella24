// model/entities/domain.rs

use super::super::datatype::DataType;
use crate::app::exports::sql::sql_export::SqlDialect;
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
        Self {
            name: "Domain".to_string(),
            data_type: DataType::default_for_dialect(SqlDialect::Oracle),
            check_constraints: vec![],
        }
    }
}

impl Domain {
    pub fn default_for_dialect(dialect: SqlDialect) -> Self {
        Self {
            name: "Domain".to_string(),
            data_type: DataType::default_for_dialect(dialect),
            check_constraints: vec![],
        }
    }
}
