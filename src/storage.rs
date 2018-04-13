/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{seq, thread_rng, ThreadRng};

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
    nodes: HashSet<Node>,
    rng: ThreadRng,
    // todo
}

impl Storage {
    /// Create empty storage
    pub fn new() -> Self {
        // todo
        Storage {
            nodes: HashSet::new(),
            rng: thread_rng(),
        }
    }

    /// Return 'max_nodes' randomly chosen from all currently known to local node.
    /// If number of nodes in the storage is N | N < max_nodes, than N informational
    /// records will be returned.
    /// Return None if storage is empty.
    pub fn get_random_nodes(&mut self, max_nodes: usize) -> Option<Vec<&NodeInfo>> {
        if self.nodes.is_empty() {
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

    /// ?
    pub fn add_node(&mut self, info: NodeInfo) {
        let record = Node {
            info,
            // set update time to current
            last_updated_sec: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|t| t.as_secs())
                .unwrap_or(0),
        };

        self.nodes.replace(record);
    }
}
