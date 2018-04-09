/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};


pub type SharedStorage = Arc<Mutex<Storage>>;


pub struct NodeList(pub Vec<Node>);

impl Iterator for NodeList {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

/// Public node info, used to create probe requests
pub struct Node {

    // todo

}

/// Internal representation of node's info
struct NodeInfo {

}


pub struct Storage {

    // todo

}

impl Storage {
    /// Create empty storage
    pub fn new() -> Self {
        // todo
        Storage{}
    }

    /// Return 'max_nodes' randomly chosen from all currently known to local node.
    /// If number of nodes in the storage N is less than 'max_nodes', than N informational
    /// records will be returned.
    /// Return NOne ff storage is empty.
    pub fn get_random_neighbours(&self, max_nodes: u8) -> Option<NodeList> {

        // todo

        None
    }
}