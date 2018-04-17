/// Store and share all network coordinates info.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{seq, Isaac64Rng};

use agent::{NodeCoordinates, NodeFlags, NodeInfo, NodeList};

pub type SharedStorage = Arc<Mutex<Storage>>;

#[derive(Debug)]
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

    /// Return 'max_nodes' most recently updated nodes, sorted by last update time.
    pub fn get_most_recent(&self, max_nodes: usize) -> Option<Vec<&NodeInfo>> {
        if self.nodes.is_empty() || max_nodes < 1 {
            return None;
        }

        let num_values_to_return: usize = if max_nodes < self.nodes.len() {
            max_nodes
        } else {
            self.nodes.len()
        };

        let mut nptr: Vec<&Node> = self.nodes.iter().collect();
        nptr.sort_by(|&a, &b| b.last_updated_sec.cmp(&a.last_updated_sec));
        Some(
            nptr.iter()
                .map(|&n| &n.info)
                .take(num_values_to_return)
                .collect(),
        )
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
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::thread;
    use std::time;

    #[test]
    fn empty_storage() {
        let mut s = Storage::new();

        assert_eq!(s.get_random_nodes(0), None);
        assert_eq!(s.get_random_nodes(1), None);
        assert_eq!(s.get_all_nodes().len(), 0);
    }

    #[test]
    fn single_entry() {
        let mut s = Storage::new();
        let node_ipv4 = NodeInfo::new(
            IpAddr::from_str("1.2.3.4").unwrap(),
            11001,
            "test_node_v4".to_string(),
        );

        s.add_node(node_ipv4.clone());

        let res: Vec<NodeInfo> = s.get_random_nodes(2)
            .unwrap()
            .iter()
            .map(|&n| n.clone())
            .collect();

        assert_eq!(res, vec![node_ipv4]);
        assert_eq!(s.get_all_nodes().len(), 1);
    }

    #[test]
    fn more_than_one_entry() {
        let mut s = Storage::new();
        let node_ipv4 = NodeInfo::new(
            IpAddr::from_str("1.2.3.4").unwrap(),
            11001,
            "test_node_v4".to_string(),
        );
        let node_ipv6 = NodeInfo::new(
            IpAddr::from_str("1a:2b:3c:4d:5e:6f:70:80").unwrap(),
            11002,
            "test_node_v6".to_string(),
        );

        s.add_node(node_ipv4.clone());
        s.add_node(node_ipv6.clone());

        assert_eq!(s.get_random_nodes(1).unwrap().len(), 1);
        assert_eq!(s.get_random_nodes(2).unwrap().len(), 2);
        assert_eq!(s.get_random_nodes(3).unwrap().len(), 2);
        assert_eq!(s.get_all_nodes().len(), 2);
    }

    #[test] #[ignore]
    fn recently_updated() {
        let mut s = Storage::new();
        assert_eq!(s.get_most_recent(0), None);

        let node_1 = NodeInfo::new(
            IpAddr::from_str("1.2.3.4").unwrap(),
            11001,
            "test_node_v4".to_string(),
        );
        let node_2 = NodeInfo::new(
            IpAddr::from_str("1a:2b:3c:4d:5e:6f:70:80").unwrap(),
            11002,
            "test_node_v6".to_string(),
        );

        s.add_node(node_1.clone());
        thread::sleep_ms(1000); // time resolution is 1 sec
        s.add_node(node_2.clone());

        println!("DEBUG | nodes: {:?}", s.nodes);
        assert_eq!(s.get_most_recent(1).unwrap()[0], &node_2);
        assert_eq!(s.get_most_recent(2).unwrap(), vec![&node_2, &node_1]);
    }

    #[test]
    fn node_location() {
        let mut s = Storage::new();
        let coord = NodeCoordinates {
            x1: 12.45,
            x2: 76.001,
            height: 10.23,
            pos_err: 0.05,
        };

        s.set_location(coord.clone());
        assert_eq!(s.get_location(), coord);
    }
}
