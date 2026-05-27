//! SQLite SQL datatypes (parameter names only, simplified).

/// Parameter definition (name only)
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: &'static str,
}

/// SQLite datatype definition: name and ordered parameter definitions.
#[derive(Debug, Clone)]
pub struct SqliteTypeDef {
    pub name: &'static str,
    pub params: &'static [ParamDef],
}

/// SQLite datatypes and their parameter names.
pub static SQLITE_TYPES: &[SqliteTypeDef] = &[
    SqliteTypeDef { name: "TEXT", params: &[] },
    SqliteTypeDef { name: "VARCHAR", params: &[ParamDef { name: "length" }] },
    SqliteTypeDef { name: "CHAR", params: &[ParamDef { name: "length" }] },
    SqliteTypeDef { name: "CLOB", params: &[] },

    SqliteTypeDef { name: "NUMERIC", params: &[] },
    SqliteTypeDef { name: "DECIMAL", params: &[ParamDef { name: "precision" }, ParamDef { name: "scale" }] },
    SqliteTypeDef { name: "NUMBER", params: &[] },

    SqliteTypeDef { name: "INTEGER", params: &[] },
    SqliteTypeDef { name: "INT", params: &[] },
    SqliteTypeDef { name: "TINYINT", params: &[] },
    SqliteTypeDef { name: "SMALLINT", params: &[] },
    SqliteTypeDef { name: "MEDIUMINT", params: &[] },
    SqliteTypeDef { name: "BIGINT", params: &[] },
    SqliteTypeDef { name: "UNSIGNED BIG INT", params: &[] },

    SqliteTypeDef { name: "REAL", params: &[] },
    SqliteTypeDef { name: "DOUBLE", params: &[] },
    SqliteTypeDef { name: "DOUBLE PRECISION", params: &[] },
    SqliteTypeDef { name: "FLOAT", params: &[] },

    SqliteTypeDef { name: "BLOB", params: &[] },
    SqliteTypeDef { name: "BINARY", params: &[ParamDef { name: "length" }] },
    SqliteTypeDef { name: "VARBINARY", params: &[ParamDef { name: "length" }] },

    SqliteTypeDef { name: "BOOLEAN", params: &[] },
    SqliteTypeDef { name: "DATE", params: &[] },
    SqliteTypeDef { name: "TIME", params: &[] },
    SqliteTypeDef { name: "TIMESTAMP", params: &[] },
    SqliteTypeDef { name: "DATETIME", params: &[] },
];

