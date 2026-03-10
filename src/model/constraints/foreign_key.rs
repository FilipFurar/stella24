use crate::app::TableId;
use crate::model::field::FieldId;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ForeignKey {
    references_table: Option<TableId>,
    references_field: Option<FieldId>,
}

impl ForeignKey {
    pub fn referenced_table(&self) -> Option<TableId> {
        self.references_table
    }

    pub fn referenced_field(&self) -> Option<FieldId> {
        self.references_field
    }

    pub fn set_referenced_table(&mut self, table: TableId) {
        self.references_table = Some(table);
    }

    pub fn set_referenced_field(&mut self, field: FieldId) {
        self.references_field = Some(field);
    }
}

impl Default for ForeignKey {
    fn default() -> Self {
        Self {
            references_table: None,
            references_field: None,
        }
    }
}

impl ForeignKey {
    pub fn new(references_table: TableId, references_field: FieldId) -> Self {
        Self {
            references_table: Some(references_table),
            references_field: Some(references_field),
        }
    }
}