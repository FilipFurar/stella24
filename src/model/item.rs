use super::{table::Table, domain::Domain, connector::Connector};
use crate::ui::node::Node;

pub enum ItemType {
    Table(Table),
    Domain(Domain),
    Connector(Connector),
}

impl ItemType {
    pub fn node(&self) -> &dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            ItemType::Connector(c) => c,
        }
    }

    pub fn node_mut(&mut self) -> &mut dyn Node {
        match self {
            ItemType::Table(t) => t,
            ItemType::Domain(d) => d,
            ItemType::Connector(c) => c,
        }
    }
}
