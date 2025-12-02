pub struct DataTypeDef {
    pub name: &'static str,
    pub param_count: usize,
}

pub struct FieldType {
    pub base: usize,
    pub params: Vec<u32>,
}

pub static DATA_TYPES: &[DataTypeDef] = &[
    DataTypeDef { name: "CHAR", param_count: 0 },
    DataTypeDef { name: "VARCHAR", param_count: 1 },
    DataTypeDef { name: "BOOL", param_count: 0 },
    DataTypeDef { name: "NUMBER", param_count: 2 },
    DataTypeDef { name: "DATE", param_count: 0 },
];
