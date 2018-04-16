/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{seq, Isaac64Rng};

use agent::{NodeCoordinates, NodeFlags, NodeInfo, NodeList};

pub type SharedStorage = Arc<Mutex<Storage>>;

pub struct Node {
    info: NodeInfo,
    last_updated_sec: u64,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.ip.hash(state);
        self.info.port.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.info.ip == other.info.ip && self.info.port == other.info.port
    }
}

impl Eq for Node {}

pub struct Storage {
    location: NodeCoordinates,
    nodes: HashSet<Node>,
    rng: Isaac64Rng,
}

impl Storage {
    /// Create empty storage
    pub fn new() -> Self {
        Storage {
            location: NodeCoordinates::empty(),
            nodes: HashSet::new(),
            rng: Isaac64Rng::new_unseeded(),
        }
    }

    /// Add new or replace existing node's information
    pub fn add_node(&mut self, info: NodeInfo) {
        let record = Node {
            info,
            // set update ts to current
            last_updated_sec: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|t| t.as_secs())
                .unwrap_or(0),
        };

        self.nodes.replace(record);
    }

    /// Return 'max_nodes' randomly chosen from all currently known to local node.
    /// If number of nodes in the storage is N | N < max_nodes, than N informational
    /// records will be returned.
    /// Return None if storage is empty.
    pub fn get_random_nodes(&mut self, max_nodes: usize) -> Option<Vec<&NodeInfo>> {
        if self.nodes.is_empty() || max_nodes < 1 {
            return None;
        }

        let num_values_to_return: usize = if max_nodes < self.nodes.len() {
            max_nodes
        } else {
            self.nodes.len()
        };

        let nptr: Vec<&Node> = self.nodes.iter().collect();
        let rand_neighbours = seq::sample_slice_ref(&mut self.rng, &nptr, num_values_to_return)
            .iter()
            .map(|&&v| &v.info)
            .collect();

        Some(rand_neighbours)
    }

    /// Return local node's full view.
    pub fn get_all_nodes(&self) -> NodeList {
        self.nodes.iter().map(|n| n.info.clone()).collect()
    }

    /// Return position of local node in RTT-based coordinate space
    pub fn get_location(&self) -> NodeCoordinates {
        self.location.clone()
    }

    /// Update location parameters of local node
    pub fn set_location(&mut self, location: NodeCoordinates) {
        self.location = location;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_storage() {
        let mut s = Storage::new();

        assert_eq!(s.get_random_nodes(0), None);
        assert_eq!(s.get_random_nodes(1), None);
        assert_eq!(s.get_all_nodes().len(), 0);
    }

    // todo more tests

}
