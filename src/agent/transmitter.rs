/// Periodically send probes.
///
/// If neighbour table is empty, send request to landmark node,
/// otherwise - send regular Location request.

use std::io;
use std::time::Duration;
use std::thread;
use std::net::{SocketAddr, UdpSocket};

use agent::{BinarySerializable, NodeInfo, NodeList, GOSSIP_MAX_NEIGHBOURS_IN_MSG};
use agent::probe::ProbeRequest;
use storage::SharedStorage;

pub struct Transmitter {
    name: String,
    landmark: SocketAddr,
    store: SharedStorage,
    transmission_interval: Duration,
    sock: UdpSocket,
    local_addr: SocketAddr,
}

impl Transmitter {
    /// Create new transmitter object
    pub fn new(
        name: String,
        landmark: SocketAddr,
        store: SharedStorage,
        sock: UdpSocket,
        transmission_interval: Duration,
    ) -> Self {
        let local_addr = sock.local_addr().expect("couldn't obtain socket address");
        Transmitter {
            name,
            landmark,
            store,
            transmission_interval,
            sock,
            local_addr,
        }
    }

    /// Start sending probes
    pub fn run(&self) -> io::Result<()> {
        loop {
            let (receiver, neighbours) = self.get_nodes();
            debug!("probing {}:{}", receiver.ip(), receiver.port());

            // create request
            let mut request = ProbeRequest::new(self.name.clone());
            if let Some(neighbours) = neighbours {
                request.set_neighbours(neighbours);
            }

            // set sending time immediately before serialization
            request.set_current_time();

            if let Some(encoded) = request.serialize() {
                self.sock.send_to(&encoded, receiver)?;
            }

            // wait
            thread::sleep(self.transmission_interval);
        }
    }

    fn get_nodes(&self) -> (SocketAddr, Option<NodeList>) {
        let mut store = self.store.lock().unwrap();
        let receiver = store.random_receiver(&self.landmark);

        let neighbours: Option<NodeList> =
            store
                .get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG, &[self.local_addr, receiver])
                .and_then(|nodes| Some(nodes.iter().map(|&n| (*n).clone()).collect()));

        (receiver, neighbours)
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
            if let (receiver, Some(nodes)) = trans.get_nodes() {
                assert!(!nodes.contains(&NodeInfo::new(
                    receiver.ip(),
                    receiver.port(),
                    String::new(),
                )));
            } else {
                panic!("get_nodes() failed");
            }
        }
    }
}
