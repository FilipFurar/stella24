//! PostgreSQL datatypes (parameter names only).

/// Parameter definition (name only)
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: &'static str,
}

/// PostgreSQL datatype definition: name and ordered parameter definitions.
#[derive(Debug, Clone)]
pub struct PostgresTypeDef {
    pub name: &'static str,
    pub params: &'static [ParamDef],
}

/// PostgreSQL datatypes and their parameter names.
pub static POSTGRES_TYPES: &[PostgresTypeDef] = &[
    // Character
    PostgresTypeDef {
        name: "CHAR",
        params: &[ParamDef { name: "length" }],
    },
    PostgresTypeDef {
        name: "VARCHAR",
        params: &[ParamDef { name: "length" }],
    },
    PostgresTypeDef {
        name: "TEXT",
        params: &[],
    },
    // Binary
    PostgresTypeDef {
        name: "BYTEA",
        params: &[],
    },
    // Numeric
    PostgresTypeDef {
        name: "SMALLINT",
        params: &[],
    },
    PostgresTypeDef {
        name: "INTEGER",
        params: &[],
    },
    PostgresTypeDef {
        name: "BIGINT",
        params: &[],
    },
    PostgresTypeDef {
        name: "DECIMAL",
        params: &[ParamDef { name: "precision" }, ParamDef { name: "scale" }],
    },
    PostgresTypeDef {
        name: "NUMERIC",
        params: &[ParamDef { name: "precision" }, ParamDef { name: "scale" }],
    },
    PostgresTypeDef {
        name: "REAL",
        params: &[],
    },
    PostgresTypeDef {
        name: "DOUBLE PRECISION",
        params: &[],
    },
    PostgresTypeDef {
        name: "MONEY",
        params: &[],
    },
    // Date/Time
    PostgresTypeDef {
        name: "DATE",
        params: &[],
    },
    PostgresTypeDef {
        name: "TIME",
        params: &[ParamDef { name: "precision" }],
    },
    PostgresTypeDef {
        name: "TIMESTAMP",
        params: &[ParamDef { name: "precision" }],
    },
    PostgresTypeDef {
        name: "TIMESTAMP WITH TIME ZONE",
        params: &[ParamDef { name: "precision" }],
    },
    PostgresTypeDef {
        name: "INTERVAL",
        params: &[ParamDef { name: "precision" }],
    },
    // Boolean
    PostgresTypeDef {
        name: "BOOLEAN",
        params: &[],
    },
    // Geometric (examples)
    PostgresTypeDef {
        name: "POINT",
        params: &[],
    },
    PostgresTypeDef {
        name: "LINE",
        params: &[],
    },
    PostgresTypeDef {
        name: "LSEG",
        params: &[],
    },
    PostgresTypeDef {
        name: "BOX",
        params: &[],
    },
    PostgresTypeDef {
        name: "PATH",
        params: &[],
    },
    PostgresTypeDef {
        name: "POLYGON",
        params: &[],
    },
    PostgresTypeDef {
        name: "CIRCLE",
        params: &[],
    },
    // Network
    PostgresTypeDef {
        name: "CIDR",
        params: &[],
    },
    PostgresTypeDef {
        name: "INET",
        params: &[],
    },
    PostgresTypeDef {
        name: "MACADDR",
        params: &[],
    },
    PostgresTypeDef {
        name: "MACADDR8",
        params: &[],
    },
    // UUID / JSON
    PostgresTypeDef {
        name: "UUID",
        params: &[],
    },
    PostgresTypeDef {
        name: "JSON",
        params: &[],
    },
    PostgresTypeDef {
        name: "JSONB",
        params: &[],
    },
    // Arrays / ranges / misc
    PostgresTypeDef {
        name: "TEXT[]",
        params: &[],
    },
    PostgresTypeDef {
        name: "INTEGER[]",
        params: &[],
    },
    PostgresTypeDef {
        name: "INT4RANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "INT8RANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "NUMRANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "TSRANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "TSTZRANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "DATERANGE",
        params: &[],
    },
    PostgresTypeDef {
        name: "OID",
        params: &[],
    },
    PostgresTypeDef {
        name: "SERIAL",
        params: &[],
    },
    PostgresTypeDef {
        name: "BIGSERIAL",
        params: &[],
    },
    PostgresTypeDef {
        name: "SMALLSERIAL",
        params: &[],
    },
];
