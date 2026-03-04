use super::field::Field;

/// SQL Table
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Table {
    /// Title (name) of the database
    pub title: String,
    /// Table rows
    fields: Vec<Field>,
}

impl Table {
    pub fn new_field(&mut self) {
        self.fields.push(Field::default());
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn remove_field(&mut self, id: usize) {
        self.fields.remove(id);
    }

    pub fn sort_by_key(&mut self) {
        self.fields.sort_by_key(|f| !f.primary_key());
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut Vec<Field> {
        &mut self.fields
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            title: "Table".to_string(),
            fields: vec![],
        }
    }
}
