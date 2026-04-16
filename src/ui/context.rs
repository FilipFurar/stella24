use crate::app::{DomainId, TableId};
use crate::model::attribute::AttrId;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;

/// Owned read-only data used by table/attribute UI.
#[derive(Clone, Debug, Default)]
pub struct DomainLookup {
    pub id: DomainId,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct TableLookup {
    pub id: TableId,
    pub title: String,
    pub has_primary_key: bool,
    pub primary_key_attributes: Vec<(AttrId, String)>,
}

/// Read-only context shared by table/attribute UI for lookup-heavy rendering.
#[derive(Clone, Debug, Default)]
pub struct TableUiContext {
    pub tables: Vec<TableLookup>,
    pub domains: Vec<DomainLookup>,
    pub current_table: TableId,
    pub current_table_attributes: Vec<(AttrId, String)>,
}

impl TableUiContext {
    pub fn from_app(
        tables: &slotmap::SlotMap<TableId, Table>,
        domains: &slotmap::SlotMap<DomainId, Domain>,
        current_table: TableId,
    ) -> Self {
        let table_lookups = tables
            .iter()
            .map(|(id, table)| TableLookup {
                id,
                title: table.title.clone(),
                has_primary_key: !table.pk.attributes.is_empty(),
                primary_key_attributes: table
                    .pk
                    .attributes
                    .iter()
                    .filter_map(|attr_id| table.attributes.get(*attr_id).map(|attr| (*attr_id, attr.name.clone())))
                    .collect(),
            })
            .collect();

        let domain_lookups = domains
            .iter()
            .map(|(id, domain)| DomainLookup {
                id,
                name: domain.name.clone(),
            })
            .collect();

        let current_table_attributes = tables
            .get(current_table)
            .map(|table| {
                table
                    .attributes
                    .iter()
                    .map(|(id, attr)| (id, attr.name.clone()))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            tables: table_lookups,
            domains: domain_lookups,
            current_table,
            current_table_attributes,
        }
    }

    pub fn table_title(&self, id: TableId) -> Option<&str> {
        self.tables.iter().find(|t| t.id == id).map(|t| t.title.as_str())
    }

    pub fn table_has_pk(&self, id: TableId) -> bool {
        self.tables
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.has_primary_key)
            .unwrap_or(false)
    }

    pub fn table_pk_attributes(&self, id: TableId) -> Option<&[(AttrId, String)]> {
        self.tables
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.primary_key_attributes.as_slice())
    }

    pub fn domain_name(&self, id: DomainId) -> Option<&str> {
        self.domains.iter().find(|d| d.id == id).map(|d| d.name.as_str())
    }

    pub fn current_table_attribute_name(&self, id: AttrId) -> Option<&str> {
        self.current_table_attributes
            .iter()
            .find(|(attr_id, _)| *attr_id == id)
            .map(|(_, name)| name.as_str())
    }
}
