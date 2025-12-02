use super::datatype::FieldType;

pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub primary_key: bool,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            name: "name".to_string(),
            field_type: FieldType {
                base: 1,
                params: vec![5],
            },
            nullable: true,
            primary_key: false,
        }
    }
}
