/// Process all incoming UDP packets.
///
/// Possible messages:
/// - Bootstrap response;
/// - Location request (foreign);
/// - Location response (for the local request).

use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket};

use storage::SharedStorage;
use super::{BinarySerializable, MsgType};
use super::bootstrap::BootstrapResponse;
use super::probe::{ProbeRequest, ProbeResponse};

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

            match MsgType::from_code(buff[0]) {
                Some(MsgType::BootstrapResp) => {
                    // process bootstrap response
                    BootstrapResponse::deserialize(&buff[..msg_len]).and_then(|msg| {
                        let mut s = self.store.lock().unwrap(); // should never fail!
                        msg.neighbours.into_iter().for_each(|n| s.add_node(n));
                        Some(())
                    });
                }

                Some(MsgType::ProbeRequest) => {
                    // respond to foreign request
                }

                Some(MsgType::ProbeResponse) => {
                    // process probe response
                }

                _ => {
                    println!("DEBUG | unknown message received");
                }
            }

            // todo implement
            // let node_addr = sender.ip();
            // let node_port = sender.port();
            //println!("get message about {} bytes from {}", msg_len, sender);
        }
    }
}
