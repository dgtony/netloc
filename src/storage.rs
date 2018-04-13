/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use rand::{seq, thread_rng, ThreadRng};

use agent::{NodeCoordinates, NodeFlags, NodeInfo, NodeList};

pub type SharedStorage = Arc<Mutex<Storage>>;

pub struct Neighbour {
    info: NodeInfo,
    last_seen_sec: u64,
}

impl Hash for Neighbour {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.ip.hash(state);
        self.info.port.hash(state);
    }
}

impl PartialEq for Neighbour {
    fn eq(&self, other: &Neighbour) -> bool {
        self.info.ip == other.info.ip && self.info.port == other.info.port
    }
}

impl Eq for Neighbour {}

pub struct Storage {
    neighbours: HashSet<Neighbour>,
    rng: ThreadRng,
    // todo
}

impl Storage {
    /// Create empty storage
    pub fn new() -> Self {
        // todo
        Storage {
            neighbours: HashSet::new(),
            rng: thread_rng(),
        }
    }

    /// Return 'max_nodes' randomly chosen from all currently known to local node.
    /// If number of nodes in the storage is N | N < max_nodes, than N informational
    /// records will be returned.
    /// Return None if storage is empty.
    pub fn get_random_nodes(&mut self, max_nodes: usize) -> Option<Vec<&NodeInfo>> {
        if self.neighbours.is_empty() {
            return None;
        }

        let num_values_to_return: usize = if max_nodes < self.neighbours.len() {
            max_nodes
        } else {
            self.neighbours.len()
        };

        let nptr: Vec<&Neighbour> = self.neighbours.iter().collect();
        let rand_neighbours = seq::sample_slice_ref(&mut self.rng, &nptr, num_values_to_return)
            .iter()
            .map(|&&v| &v.info)
            .collect();

        Some(rand_neighbours)
    }
}
