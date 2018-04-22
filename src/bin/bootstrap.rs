//! Bootstrap server.
//!
//! Mandatory part of network location infrastructure.
//! Bootstrap server allows recently connected agents
//! to find its first neighbours to start communication.
//!

extern crate netloc;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::io;

use netloc::storage;
use netloc::agent::bootstrap::{BootstrapRequest, BootstrapResponse};
use netloc::agent::{BinarySerializable, MsgType, NodeInfo, GOSSIP_MAX_NEIGHBOURS_IN_MSG, LANDMARK_AGENT_NAME};

const RCV_BUFF_SIZE: usize = 1500;

fn run_server(addr: SocketAddr, store: &mut storage::Storage) -> io::Result<()> {
    let mut buff: [u8; RCV_BUFF_SIZE] = [0; RCV_BUFF_SIZE];
    let sock = UdpSocket::bind(addr)?;

    loop {
        let (msg_len, sender) = sock.recv_from(&mut buff)?;
        let msg_data = &buff[1..msg_len];

        match MsgType::from_code(buff[0]) {
            Some(MsgType::BootstrapReq) => {
                BootstrapRequest::deserialize(msg_data).and_then(|msg| {
                    // store requesting node
                    store.add_node(NodeInfo::new(sender.ip(), sender.port(), msg.local_name));

                    // construct response
                    let response = {
                        // there must be always at least one node (landmark) in storage
                        let nodes = store
                            .get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG, &[sender])
                            .unwrap();

                        let mut response = BootstrapResponse::empty();
                        for n in nodes {
                            response.neighbours.push(n.clone());
                        }

                        response
                    };

                    // send back response
                    response
                        .serialize()
                        .and_then(|encoded| Some(sock.send_to(&encoded, sender)))
                });
            }

            _ => println!("DEBUG | received unknown message: {:?}", msg_data),
        }
    }
}

fn main() {
    // todo:
    // - use predefined (CLI args?) address of landmark-agent with zero coordinates
    let addr = IpAddr::from_str("0.0.0.0").expect("bad address to listen on");
    let port = 3739;

    // landmark node with zero coordinates
    let landmark_node_ip = IpAddr::from_str("127.0.0.1").expect("bad landmark node IP");
    let landmark_node_port = 3738;
    let landmark_info = NodeInfo::new(landmark_node_ip, landmark_node_port, LANDMARK_AGENT_NAME.to_string());

    // init storage
    let mut store = storage::Storage::new();
    store.add_node(landmark_info);

    println!("INFO | starting bootstrap server on {:?}:{}", &addr, port);

    run_server(SocketAddr::new(addr, port), &mut store);
}
