// model/datatype.rs

/// A struct containing a name and the count of built-in data types, which are defined as static `DATA_TYPES`
pub struct DataTypeDef {
    pub name: &'static str,
    pub param_count: usize,
}

/// Built-in data type
/// ## Base
/// The index of the data type in `DATA_TYPES`
/// ## Params
/// Parameter values
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DataType {
    pub base: usize,
    pub params: Vec<u32>,
}

/// Possible built-in data types
pub static DATA_TYPES: &[DataTypeDef] = &[
    DataTypeDef {
        name: "CHAR",
        param_count: 1,
    },
    DataTypeDef {
        name: "VARCHAR",
        param_count: 1,
    },
    DataTypeDef {
        name: "BOOL",
        param_count: 0,
    },
    DataTypeDef {
        name: "NUMBER",
        param_count: 2,
    },
    DataTypeDef {
        name: "DATE",
        param_count: 0,
    },
];
