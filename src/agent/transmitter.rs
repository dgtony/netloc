/// Periodically send probes.
///
/// If neighbour table is empty, send Bootstrap request,
/// otherwise - send Location request.

use std::io;
use std::time::Duration;
use std::thread;
use std::net::UdpSocket;

use storage::SharedStorage;

const GOSSIP_MAX_NEIGHBOURS_IN_MSG: usize = 4;

pub struct Transmitter {
    store: SharedStorage,
    transmission_interval: Duration,
    sock: UdpSocket,
    // todo
}

impl Transmitter {
    /// Create new transmitter object
    pub fn new(store: SharedStorage, sock: UdpSocket, transmission_interval: Duration) -> Self {
        Transmitter {
            store,
            transmission_interval,
            sock,
        }
    }

    /// Start sending probes
    pub fn run(&self) -> io::Result<()> {
        // fixme mb change to custom error?
        loop {
            let mut s = self.store.lock().unwrap();

            if let Some(neighbours) = s.get_random_neighbours(GOSSIP_MAX_NEIGHBOURS_IN_MSG) {

                // todo create message

                // todo: send request with neighbour list

            } else {

                // todo empty table - bootstrap

            }

            // wait
            thread::sleep(self.transmission_interval);
        }
    }
}
