/// Periodically send probes.
///
/// If neighbour table is empty, send Bootstrap request,
/// otherwise - send Location request.

use std::io;
use std::time::Duration;
use std::thread;
use std::net::{SocketAddr, UdpSocket};

use agent::{BinarySerializable, NodeInfo, NodeList, GOSSIP_MAX_NEIGHBOURS_IN_MSG};
use agent::bootstrap::BootstrapRequest;
use agent::probe::ProbeRequest;
use storage::SharedStorage;

pub struct Transmitter {
    name: String,
    bootstrap: SocketAddr,
    store: SharedStorage,
    transmission_interval: Duration,
    sock: UdpSocket,
    local_addr: SocketAddr,
}

impl Transmitter {
    /// Create new transmitter object
    pub fn new(
        name: String,
        bootstrap: SocketAddr,
        store: SharedStorage,
        sock: UdpSocket,
        transmission_interval: Duration,
    ) -> Self {
        let local_addr = sock.local_addr().expect("couldn't obtain socket address");
        Transmitter {
            name,
            bootstrap,
            store,
            transmission_interval,
            sock,
            local_addr,
        }
    }

    /// Start sending probes
    pub fn run(&self) -> io::Result<()> {
        loop {
            if let Some((receiver, neighbours)) = self.get_nodes() {
                debug!(
                    "probing {}:{} (aka {})",
                    receiver.ip, receiver.port, &receiver.name
                );

                // create request
                let mut request = ProbeRequest::new(self.name.clone());
                request.set_neighbours(neighbours);

                // set sending time immediately before serialization
                request.set_current_time();

                if let Some(encoded) = request.serialize() {
                    self.sock
                        .send_to(&encoded, SocketAddr::new(receiver.ip, receiver.port))?;
                }
            } else {
                let request = BootstrapRequest::new(self.name.clone());
                if let Some(encoded) = request.serialize() {
                    self.sock.send_to(&encoded, self.bootstrap)?;
                }
            }

            // wait
            thread::sleep(self.transmission_interval);
        }
    }

    fn get_nodes(&self) -> Option<(NodeInfo, Vec<NodeInfo>)> {
        let mut store = self.store.lock().unwrap();
        let nodes = store.get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG + 1, &[self.local_addr])?;
        let receiver = nodes[0].clone();
        let neighbours: NodeList = nodes.iter().skip(1).map(|&n| (*n).clone()).collect();

        Some((receiver, neighbours))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::{SharedStorage, Storage};

    use std::sync::{Arc, Mutex};
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn node_samples() {
        let mut store = Storage::new();

        // fill neighbours
        for i in 1..10 {
            let addr = SocketAddr::from_str(format!("127.0.0.1:{}", i).as_ref()).unwrap();
            store.add_node(NodeInfo::new(addr.ip(), addr.port(), format!("{}", i)));
        }
        let s: SharedStorage = Arc::new(Mutex::new(store));

        let sock = SocketAddr::from_str("127.0.0.1:12345").unwrap();
        let trans = Transmitter::new(
            "test".to_string(),
            SocketAddr::from_str("5.5.5.5:12345").unwrap(),
            s,
            UdpSocket::bind(sock).unwrap(),
            Duration::new(1, 0),
        );

        // ensure that receiver never appears in node list
        for i in 1..100 {
            if let Some((receiver, nodes)) = trans.get_nodes() {
                assert!(!nodes.contains(&receiver));
            } else {
                panic!("get_nodes() failed");
            }
        }
    }
}
