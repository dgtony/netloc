/// Periodically send probes.
///
/// If neighbour table is empty, send Bootstrap request,
/// otherwise - send Location request.

use std::io;
use std::time::Duration;
use std::thread;
use std::net::{SocketAddr, UdpSocket};

use agent::{BinarySerializable, NodeList, GOSSIP_MAX_NEIGHBOURS_IN_MSG};
use agent::bootstrap::BootstrapRequest;
use agent::probe::ProbeRequest;
use storage::SharedStorage;

pub struct Transmitter {
    name: String,
    bootstrap: SocketAddr,
    store: SharedStorage,
    transmission_interval: Duration,
    sock: UdpSocket,
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
        Transmitter {
            name,
            bootstrap,
            store,
            transmission_interval,
            sock,
        }
    }

    /// Start sending probes
    pub fn run(&self) -> io::Result<()> {
        // fixme mb change to custom error?
        loop {
            let mut store = self.store.lock().unwrap();

            if let Some(nodes) = store.get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG + 1, &[]) {
                let receiver = nodes[0];

                // create request
                let mut request = ProbeRequest::new(self.name.clone());
                let neighbours: NodeList = nodes.iter().skip(1).map(|&n| (*n).clone()).collect();
                request.set_neighbours(neighbours);

                // set sending time immediately before serialization
                request.set_current_time();
                if let Some(encoded) = request.serialize() {
                    // send request with neighbour list
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
}
