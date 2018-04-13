/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};

use agent::{NodeCoordinates, NodeFlags, NodeInfo, NodeList};

use rand::{seq, thread_rng, ThreadRng};

pub type SharedStorage = Arc<Mutex<Storage>>;


pub struct Neighbour {
    info: NodeInfo,
    last_seen_sec: u64,
}


pub struct Storage {
    neighbours: Vec<Neighbour>,
    rng: ThreadRng,
    // todo
}

impl Storage {
    /// Create empty storage
    pub fn new() -> Self {
        // todo
        Storage {
            neighbours: Vec::new(),
            rng: thread_rng(),
        }
    }

    /// Return 'max_nodes' randomly chosen from all currently known to local node.
    /// If number of nodes in the storage is N | N < max_nodes, than N informational
    /// records will be returned.
    /// Return None if storage is empty.
    pub fn get_random_neighbours(&mut self, max_nodes: usize) -> Option<Vec<&NodeInfo>> {
        let num_neighbours = self.neighbours.len();

        if num_neighbours < 1 {
            return None;
        }

        let num_values: usize = if max_nodes < num_neighbours {
            max_nodes
        } else {
            num_neighbours
        };

        let rand_neighbours = seq::sample_slice_ref(&mut self.rng, &self.neighbours, num_values)
            .iter()
            .map(|&v| &v.info)
            .collect();

        Some(rand_neighbours)
    }
}
