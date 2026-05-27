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

impl DataType {
    /// Returns the number of parameters expected by the selected built-in type.
    pub fn expected_param_count(&self) -> usize {
        DATA_TYPES.get(self.base).map(|def| def.param_count).unwrap_or(0)
    }

    /// Adjust the parameter vector to match the selected built-in type.
    ///
    /// Extra values are removed, and missing values are filled with sensible
    /// defaults for the built-in type.
    pub fn normalize_params(&mut self) {
        let expected = self.expected_param_count();
        let fill_value = match DATA_TYPES.get(self.base).map(|def| def.name) {
            Some("CHAR") | Some("VARCHAR2") => 1,
            _ => 0,
        };

        if self.params.len() > expected {
            self.params.truncate(expected);
        } else if self.params.len() < expected {
            self.params.resize(expected, fill_value);
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        Self {
            base: 1,
            params: vec![1],
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CharOrByte {
    Char,
    Byte,
}

/// Possible built-in data types
pub static DATA_TYPES: &[DataTypeDef] = &[
    DataTypeDef {
        name: "NUMBER",
        param_count: 2,
    },
    DataTypeDef {
        name: "VARCHAR2",
        param_count: 2,
    },
    DataTypeDef {
        name: "CHAR",
        param_count: 2,
    },
    DataTypeDef {
        name: "DATE",
        param_count: 0,
    },
    DataTypeDef {
        name: "TIMESTAMP",
        param_count: 1,
    },
    DataTypeDef {
        name: "FLOAT",
        param_count: 1,
    },
    DataTypeDef {
        name: "LONG",
        param_count: 0,
    },
    DataTypeDef {
        name: "BINARY_FLOAT",
        param_count: 0,
    },
    DataTypeDef {
        name: "BINARY_DOUBLE",
        param_count: 0,
    },
    DataTypeDef {
        name: "TIMESTAMP WITH TIME ZONE",
        param_count: 1,
    },
    DataTypeDef {
        name: "TIMESTAMP WITH LOCAL TIME ZONE",
        param_count: 1,
    },
    DataTypeDef {
        name: "INTERVAL_YEAR",
        param_count: 1,
    },
    DataTypeDef {
        name: "INTERVAL_DAY",
        param_count: 2,
    },
    DataTypeDef {
        name: "LONG RAW",
        param_count: 0,
    },
    DataTypeDef {
        name: "ROWID",
        param_count: 0,
    },
    DataTypeDef {
        name: "UROWID",
        param_count: 1,
    },
    DataTypeDef {
        name: "NVARCHAR2",
        param_count: 1,
    },
    DataTypeDef {
        name: "NCHAR",
        param_count: 1,
    },
    DataTypeDef {
        name: "NCLOB",
        param_count: 0,
    },
    DataTypeDef {
        name: "BLOB",
        param_count: 0,
    },
    DataTypeDef {
        name: "BFILE",
        param_count: 0,
    },
];
