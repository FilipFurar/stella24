//! Oracle SQL datatypes (parameter names only).
//!
//! This file lists Oracle datatypes and the names of their parameters.

/// Parameter definition (name only)
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: &'static str,
}

/// Oracle datatype definition: name and ordered parameter definitions.
#[derive(Debug, Clone)]
pub struct OracleTypeDef {
    pub name: &'static str,
    pub params: &'static [ParamDef],
}

/// Oracle datatypes and their parameter names.
pub static ORACLE_TYPES: &[OracleTypeDef] = &[
    OracleTypeDef { name: "CHAR", params: &[ParamDef { name: "size" }, ParamDef { name: "length_semantics" }] },
    OracleTypeDef { name: "VARCHAR2", params: &[ParamDef { name: "size" }, ParamDef { name: "length_semantics" }] },
    OracleTypeDef { name: "NCHAR", params: &[ParamDef { name: "size" }] },
    OracleTypeDef { name: "NVARCHAR2", params: &[ParamDef { name: "size" }] },

    OracleTypeDef { name: "NUMBER", params: &[ParamDef { name: "precision" }, ParamDef { name: "scale" }] },
    OracleTypeDef { name: "FLOAT", params: &[ParamDef { name: "precision" }] },
    OracleTypeDef { name: "BINARY_FLOAT", params: &[] },
    OracleTypeDef { name: "BINARY_DOUBLE", params: &[] },

    OracleTypeDef { name: "DATE", params: &[] },
    OracleTypeDef { name: "TIMESTAMP", params: &[ParamDef { name: "fractional_seconds_precision" }] },
    OracleTypeDef { name: "TIMESTAMP WITH TIME ZONE", params: &[ParamDef { name: "fractional_seconds_precision" }] },
    OracleTypeDef { name: "TIMESTAMP WITH LOCAL TIME ZONE", params: &[ParamDef { name: "fractional_seconds_precision" }] },

    OracleTypeDef { name: "INTERVAL YEAR TO MONTH", params: &[ParamDef { name: "year_precision" }] },
    OracleTypeDef { name: "INTERVAL DAY TO SECOND", params: &[ParamDef { name: "day_precision" }, ParamDef { name: "fractional_seconds_precision" }] },

    OracleTypeDef { name: "BLOB", params: &[] },
    OracleTypeDef { name: "CLOB", params: &[] },
    OracleTypeDef { name: "NCLOB", params: &[] },
    OracleTypeDef { name: "BFILE", params: &[] },
    OracleTypeDef { name: "LONG", params: &[] },
    OracleTypeDef { name: "LONG RAW", params: &[] },

    OracleTypeDef { name: "ROWID", params: &[] },
    OracleTypeDef { name: "UROWID", params: &[ParamDef { name: "size" }] },
];

