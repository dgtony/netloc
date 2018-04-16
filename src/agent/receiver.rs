/// Process all incoming UDP packets.
///
/// Possible messages:
/// - Bootstrap response;
/// - Location request (foreign);
/// - Location response (for the local request).

use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket};

use storage::SharedStorage;

const RCV_BUFF_SIZE: usize = 1500;

pub struct Receiver {
    name: String,
    store: SharedStorage,
    sock: UdpSocket,
}

impl Receiver {
    pub fn new(name: String, store: SharedStorage, sock: UdpSocket) -> Self {
        Receiver { name, store, sock }
    }

    pub fn run(&self) -> io::Result<()> {
        let mut buff: [u8; RCV_BUFF_SIZE] = [0; RCV_BUFF_SIZE];

        loop {
            let (msg_len, sender) = self.sock.recv_from(&mut buff)?;

            // todo implement
            // let node_addr = sender.ip();
            // let node_port = sender.port();
            println!("get message about {} bytes from {}", msg_len, sender);
        }
    }
}
