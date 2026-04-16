use crate::app::{DomainId, TableId};
use crate::model::attribute::AttrId;
use crate::model::entities::domain::Domain;
use crate::model::entities::table::Table;

/// Owned domain lookup data for UI rendering.
#[derive(Clone, Debug, Default)]
pub struct DomainLookup {
    /// The domain ID.
    pub id: DomainId,
    /// The domain name.
    pub name: String,
}

/// Owned table lookup data for UI rendering.
#[derive(Clone, Debug, Default)]
pub struct TableLookup {
    /// The table ID.
    pub id: TableId,
    /// The table title.
    pub title: String,
    /// Whether the table has a primary key.
    pub has_primary_key: bool,
    /// Snapshot of primary-key attributes as `(AttrId, name)` pairs.
    pub primary_key_attributes: Vec<(AttrId, String)>,
}

/// Owned read-only context shared by table and attribute UI.
#[derive(Clone, Debug, Default)]
pub struct TableUiContext {
    /// Snapshot of all tables available to the workbench.
    pub tables: Vec<TableLookup>,
    /// Snapshot of all domains available to the workbench.
    pub domains: Vec<DomainLookup>,
    /// The table currently being rendered.
    pub current_table: TableId,
    /// Snapshot of the current table's attributes as `(AttrId, name)` pairs.
    pub current_table_attributes: Vec<(AttrId, String)>,
}

impl TableUiContext {
    /// Builds a snapshot of the current workbench state for UI rendering.
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
                    .filter_map(|attr_id| {
                        table
                            .attributes
                            .get(*attr_id)
                            .map(|attr| (*attr_id, attr.name.clone()))
                    })
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

    /// Returns the title for the given table ID, if it exists in the snapshot.
    pub fn table_title(&self, id: TableId) -> Option<&str> {
        self.tables
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.title.as_str())
    }

    /// Returns whether the given table has a primary key in the snapshot.
    pub fn table_has_pk(&self, id: TableId) -> bool {
        self.tables
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.has_primary_key)
            .unwrap_or(false)
    }

    /// Returns the primary-key attributes for the given table, if present in the snapshot.
    pub fn table_pk_attributes(&self, id: TableId) -> Option<&[(AttrId, String)]> {
        self.tables
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.primary_key_attributes.as_slice())
    }

    /// Returns the name for the given domain ID, if it exists in the snapshot.
    pub fn domain_name(&self, id: DomainId) -> Option<&str> {
        self.domains
            .iter()
            .find(|d| d.id == id)
            .map(|d| d.name.as_str())
    }

    /// Returns the current table's attribute name for the given attribute ID, if present.
    pub fn current_table_attribute_name(&self, id: AttrId) -> Option<&str> {
        self.current_table_attributes
            .iter()
            .find(|(attr_id, _)| *attr_id == id)
            .map(|(_, name)| name.as_str())
    }
}
