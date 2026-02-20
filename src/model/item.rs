use super::{/*connector::Connector,*/ domain::Domain, table::Table};
use crate::ui::node::Node;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    Table(Table),
    Domain(Domain),
    //Connector(Connector),
}

impl ItemType {
    pub fn node(&self) -> &dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            //ItemType::Connector(c) => c,
        }
    }

    pub fn node_mut(&mut self) -> &mut dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            //ItemType::Connector(c) => c,
        }
    }
}
