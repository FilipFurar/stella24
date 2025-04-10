// Enum wrapping all possible workbench item types
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum WorkbenchItemType {
    Table(Table),
    Domain(Domain),
    Connector(Box<Connector>),
}

// Implement shared functionality for WorkbenchItemType
impl WorkbenchItemType {
    // Get display name for the item based on its type
    pub fn display_name(&self) -> String {
        match self {
            WorkbenchItemType::Table(t) => t.title.clone() + " > id: " + &t.id.to_string(),
            WorkbenchItemType::Domain(d) => d.title.clone() + " > id: " + &d.id.to_string(),
            WorkbenchItemType::Connector(c) => c.id.to_string(),
        }
    }
}

// Table struct representing a table workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Table {
    pub id: usize,
    pub(crate) title: String, // Title of the table
}

// Domain struct representing a domain workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Domain {
    pub id: usize,            // Unique ID of the domain
    pub(crate) title: String, // Title of the domain
}

// Connector struct representing a connector workbench item
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Connector {
    pub id: usize, // Unique ID of the connector
                   /*pub first_point: WorkbenchItemType,
                   pub second_point: WorkbenchItemType,*/
}
