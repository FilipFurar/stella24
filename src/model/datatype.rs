// model/datatype.rs

use crate::app::exports::sql::oracle_types::ORACLE_TYPES;
use crate::app::exports::sql::postgres_types::POSTGRES_TYPES;
use crate::app::exports::sql::sql_export::SqlDialect;
use crate::app::exports::sql::sqlite_types::SQLITE_TYPES;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CharOrByte {
    Char,
    Byte,
}

/// View over the datatype catalog for a concrete SQL dialect.
pub enum DialectTypes {
    Oracle(&'static [crate::app::exports::sql::oracle_types::OracleTypeDef]),
    Postgres(&'static [crate::app::exports::sql::postgres_types::PostgresTypeDef]),
    Sqlite(&'static [crate::app::exports::sql::sqlite_types::SqliteTypeDef]),
}

impl DialectTypes {
    pub fn len(&self) -> usize {
        match self {
            DialectTypes::Oracle(types) => types.len(),
            DialectTypes::Postgres(types) => types.len(),
            DialectTypes::Sqlite(types) => types.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            DialectTypes::Oracle(types) => types.is_empty(),
            DialectTypes::Postgres(types) => types.is_empty(),
            DialectTypes::Sqlite(types) => types.is_empty(),
        }
    }

    pub fn name(&self, base: usize) -> Option<&'static str> {
        match self {
            DialectTypes::Oracle(types) => types.get(base).map(|def| def.name),
            DialectTypes::Postgres(types) => types.get(base).map(|def| def.name),
            DialectTypes::Sqlite(types) => types.get(base).map(|def| def.name),
        }
    }

    pub fn param_count(&self, base: usize) -> usize {
        match self {
            DialectTypes::Oracle(types) => types.get(base).map(|def| def.params.len()).unwrap_or(0),
            DialectTypes::Postgres(types) => {
                types.get(base).map(|def| def.params.len()).unwrap_or(0)
            }
            DialectTypes::Sqlite(types) => types.get(base).map(|def| def.params.len()).unwrap_or(0),
        }
    }

    pub fn param_name(&self, base: usize, param_index: usize) -> Option<&'static str> {
        match self {
            DialectTypes::Oracle(types) => {
                types.get(base)?.params.get(param_index).map(|def| def.name)
            }
            DialectTypes::Postgres(types) => {
                types.get(base)?.params.get(param_index).map(|def| def.name)
            }
            DialectTypes::Sqlite(types) => {
                types.get(base)?.params.get(param_index).map(|def| def.name)
            }
        }
    }
}

pub fn dialect_types(dialect: SqlDialect) -> DialectTypes {
    match dialect {
        SqlDialect::Oracle => DialectTypes::Oracle(ORACLE_TYPES),
        SqlDialect::Postgres => DialectTypes::Postgres(POSTGRES_TYPES),
        SqlDialect::Sqlite => DialectTypes::Sqlite(SQLITE_TYPES),
    }
}

fn default_type_index(dialect: SqlDialect) -> usize {
    match dialect {
        SqlDialect::Oracle => 1,
        SqlDialect::Postgres => 1,
        SqlDialect::Sqlite => 0,
    }
}

fn default_param_value(type_name: &str, param_index: usize) -> u32 {
    match type_name {
        "CHAR" | "VARCHAR" | "VARCHAR2" | "NCHAR" | "NVARCHAR2" | "TEXT" | "CLOB" | "NCLOB"
        | "BINARY" | "VARBINARY" | "UROWID" => 1,
        "NUMBER" | "NUMERIC" | "DECIMAL" => {
            if param_index == 0 {
                1
            } else {
                0
            }
        }
        "FLOAT"
        | "TIMESTAMP"
        | "TIMESTAMP WITH TIME ZONE"
        | "TIMESTAMP WITH LOCAL TIME ZONE"
        | "TIME"
        | "INTERVAL YEAR TO MONTH"
        | "INTERVAL DAY TO SECOND" => 6,
        _ => 0,
    }
}

fn find_type_index(dialect: SqlDialect, type_name: &str) -> Option<usize> {
    let types = dialect_types(dialect);
    (0..types.len()).find(|idx| types.name(*idx) == Some(type_name))
}

fn translation_candidates(target: SqlDialect, source_name: &str) -> &'static [&'static str] {
    match target {
        SqlDialect::Oracle => match source_name {
            "TEXT" => &["VARCHAR2", "NCLOB", "CLOB"],
            "VARCHAR" => &["VARCHAR2"],
            "CHAR" => &["CHAR"],
            "NUMBER" | "NUMERIC" | "DECIMAL" | "INTEGER" | "INT" | "BIGINT" | "SMALLINT"
            | "REAL" | "DOUBLE PRECISION" | "MONEY" => &["NUMBER", "FLOAT"],
            "BYTEA" | "BLOB" => &["BLOB", "LONG RAW", "BFILE"],
            "BOOLEAN" => &["NUMBER"],
            "DATE"
            | "TIME"
            | "DATETIME"
            | "TIMESTAMP"
            | "TIMESTAMP WITH TIME ZONE"
            | "TIMESTAMP WITH LOCAL TIME ZONE" => &["TIMESTAMP", "DATE"],
            "INTERVAL" => &["INTERVAL DAY TO SECOND", "INTERVAL YEAR TO MONTH"],
            "ROWID" | "UROWID" => &["ROWID", "UROWID"],
            _ => &[],
        },
        SqlDialect::Postgres => match source_name {
            "VARCHAR2" => &["VARCHAR", "TEXT"],
            "NVARCHAR2" => &["VARCHAR", "TEXT"],
            "CHAR" | "NCHAR" => &["CHAR", "VARCHAR", "TEXT"],
            "NUMBER" => &["NUMERIC", "DECIMAL"],
            "FLOAT" | "BINARY_FLOAT" | "BINARY_DOUBLE" => &["REAL", "DOUBLE PRECISION"],
            "LONG" | "NCLOB" => &["TEXT"],
            "LONG RAW" | "BLOB" | "BFILE" => &["BYTEA"],
            "ROWID" | "UROWID" => &["TEXT"],
            "INTERVAL_YEAR" | "INTERVAL_DAY" => &["INTERVAL"],
            "TIMESTAMP WITH LOCAL TIME ZONE" => &["TIMESTAMP WITH TIME ZONE", "TIMESTAMP"],
            "BINARY" | "VARBINARY" => &["BYTEA"],
            "BOOLEAN" => &["BOOLEAN"],
            _ => &[],
        },
        SqlDialect::Sqlite => match source_name {
            "VARCHAR2" | "VARCHAR" | "CHAR" | "NCHAR" | "NVARCHAR2" => &["TEXT"],
            "NUMBER" | "NUMERIC" | "DECIMAL" => &["NUMERIC"],
            "FLOAT" | "BINARY_FLOAT" | "BINARY_DOUBLE" | "REAL" | "DOUBLE PRECISION" | "MONEY" => {
                &["REAL"]
            }
            "LONG"
            | "NCLOB"
            | "DATE"
            | "TIME"
            | "TIMESTAMP"
            | "TIMESTAMP WITH TIME ZONE"
            | "TIMESTAMP WITH LOCAL TIME ZONE"
            | "DATETIME"
            | "INTERVAL_YEAR"
            | "INTERVAL_DAY"
            | "ROWID"
            | "UROWID" => &["TEXT"],
            "LONG RAW" | "BLOB" | "BFILE" | "BYTEA" | "VARBINARY" | "BINARY" => &["BLOB"],
            "BOOLEAN" => &["INTEGER", "NUMERIC"],
            _ => &[],
        },
    }
}

fn translated_type_index(target: SqlDialect, source_name: &str) -> Option<usize> {
    if let Some(idx) = find_type_index(target, source_name) {
        return Some(idx);
    }

    for candidate in translation_candidates(target, source_name) {
        if let Some(idx) = find_type_index(target, candidate) {
            return Some(idx);
        }
    }

    None
}

/// Built-in data type selected from the active project's SQL dialect.
///
/// `dialect` selects the concrete datatype catalog, `base` stores the index in
/// that catalog, and `params` stores the concrete parameter values.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DataType {
    #[serde(default)]
    pub dialect: SqlDialect,
    pub base: usize,
    pub params: Vec<u32>,
}

impl DataType {
    pub fn new(dialect: SqlDialect, base: usize) -> Self {
        let mut value = Self {
            dialect,
            base,
            params: Vec::new(),
        };
        value.normalize_params();
        value
    }

    pub fn default_for_dialect(dialect: SqlDialect) -> Self {
        Self::new(dialect, default_type_index(dialect))
    }

    pub fn catalog(&self) -> DialectTypes {
        dialect_types(self.dialect)
    }

    pub fn type_name(&self) -> &'static str {
        self.catalog().name(self.base).unwrap_or("UNKNOWN")
    }

    pub fn expected_param_count(&self) -> usize {
        self.catalog().param_count(self.base)
    }

    pub fn param_name(&self, param_index: usize) -> Option<&'static str> {
        self.catalog().param_name(self.base, param_index)
    }

    pub fn normalize_params(&mut self) {
        let expected = self.expected_param_count();
        let type_name = self.type_name();
        let fill_value = |idx: usize| default_param_value(type_name, idx);

        if self.params.len() > expected {
            self.params.truncate(expected);
        } else if self.params.len() < expected {
            let start = self.params.len();
            self.params.extend((start..expected).map(fill_value));
        }
    }

    pub fn display_text(&self) -> String {
        let name = self.type_name();
        if self.params.is_empty() {
            name.to_string()
        } else {
            format!(
                "{}({})",
                name,
                self.params
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    pub fn translate_to(&mut self, target: SqlDialect) {
        if self.dialect == target {
            self.normalize_params();
            return;
        }

        let source_name = self.type_name();
        let source_params = self.params.clone();
        let target_base = translated_type_index(target, source_name)
            .unwrap_or_else(|| default_type_index(target));

        self.dialect = target;
        self.base = target_base;
        self.params = source_params;
        self.normalize_params();
    }
}

impl Default for DataType {
    fn default() -> Self {
        Self::default_for_dialect(SqlDialect::Oracle)
    }
}
