/// Process all incoming UDP packets.
///
/// Possible messages:
/// - Bootstrap response;
/// - Location request (foreign);
/// - Location response (for the local request).

use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::{IpAddr, SocketAddr, UdpSocket};

use storage::SharedStorage;
use agent::{BinarySerializable, MsgType, NodeCoordinates, NodeInfo, GOSSIP_MAX_NEIGHBOURS_IN_MSG};
use agent::bootstrap::BootstrapResponse;
use agent::probe::{ProbeRequest, ProbeResponse};

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
            let msg_data = &buff[1..msg_len];

            match MsgType::from_code(buff[0]) {
                Some(MsgType::BootstrapResp) => {
                    // process bootstrap response
                    BootstrapResponse::deserialize(msg_data).and_then(|msg| {
                        let mut s = self.store.lock().unwrap(); // should never fail!
                        msg.neighbours.into_iter().for_each(|n| s.add_node(n));
                        Some(())
                    });
                }

                Some(MsgType::ProbeRequest) => {
                    // respond to foreign request
                    let response = ProbeRequest::deserialize(msg_data).and_then(|request| {
                        let mut s = self.store.lock().unwrap();

                        // form response
                        let mut response = ProbeResponse::new(self.name.clone(), s.get_location());

                        // add some neighbour's info
                        if let Some(neighbours) =
                            s.get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG, &[sender])
                                .and_then(|nodes| Some(nodes.iter().map(|&n| n.clone()).collect()))
                        {
                            response.set_neighbours(neighbours);
                        }

                        // send back initial transmission time
                        response.copy_time(&request);

                        // save received information about nodes
                        if let Some(neighbours) = request.neighbours {
                            neighbours.into_iter().for_each(|n| s.add_node(n));
                        }

                        Some(response)
                    });

                    // send back response
                    if let Some(encoded) = response.and_then(|r| r.serialize()) {
                        self.sock.send_to(&encoded, sender)?;
                    }
                }

                Some(MsgType::ProbeResponse) => {
                    // message reception time
                    let received_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    // decode and process
                    ProbeResponse::deserialize(msg_data).and_then(|response| {
                        let mut s = self.store.lock().unwrap();

                        // recompute own location based on response's RTT
                        if let Some(rtt) = received_at
                            .checked_sub(Duration::new(response.sent_at_sec, response.sent_at_nsec))
                        {
                            s.update_location(&response.location, rtt);
                        }

                        // store information about respondent
                        let mut respondent =
                            NodeInfo::new(sender.ip(), sender.port(), self.name.clone());
                        respondent.set_coordinates(&response.location);
                        s.add_node(respondent);

                        // store info about its neighbours
                        if let Some(neighbours) = response.neighbours {
                            neighbours.into_iter().for_each(|n| s.add_node(n));
                        }

                        Some(())
                    });
                }

                _ => {
                    println!("DEBUG | unknown message received: {:?}", msg_data);
                }
            }
        }
    }
}
