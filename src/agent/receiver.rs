/// Process all incoming UDP packets.
///
/// Possible messages:
/// - Bootstrap response;
/// - Location request (foreign);
/// - Location response (for the local request).

use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::{SocketAddr, UdpSocket};

use storage::SharedStorage;
use agent::{AgentType, BinarySerializable, MsgType, NodeInfo, GOSSIP_MAX_NEIGHBOURS_IN_MSG};
use agent::bootstrap::BootstrapResponse;
use agent::probe::{ProbeRequest, ProbeResponse};

const RCV_BUFF_SIZE: usize = 1500;

pub struct Receiver {
    agent_type: AgentType,
    name: String,
    store: SharedStorage,
    sock: UdpSocket,
    local_addr: SocketAddr,
}

impl Receiver {
    pub fn new(agent_type: AgentType, name: String, store: SharedStorage, sock: UdpSocket) -> Self {
        let local_addr = sock.local_addr().expect("couldn't obtain socket address");

        Receiver {
            agent_type,
            name,
            store,
            sock,
            local_addr,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        match self.agent_type {
            AgentType::Regular => self.run_regular(),
            AgentType::Landmark => self.run_landmark(),
        }
    }

    /// Fully functional agent responder
    fn run_regular(&self) -> io::Result<()> {
        let mut buff: [u8; RCV_BUFF_SIZE] = [0; RCV_BUFF_SIZE];

        loop {
            let (msg_len, sender) = self.sock.recv_from(&mut buff)?;
            let msg_data = &buff[1..msg_len];

            match MsgType::from_code(buff[0]) {
                Some(MsgType::BootstrapResp) => {
                    debug!("get bootstrap response");

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
                        debug!("detected probe from {}:{} (aka {})", sender.ip(), sender.port(), &request.sender_name);

                        // storage
                        let mut s = self.store.lock().unwrap();

                        // form response
                        let mut response = ProbeResponse::new(self.name.clone(), s.get_location());

                        // send back initial transmission time
                        response.copy_time(&request);

                        // add some neighbour's info
                        if let Some(neighbours) = s.get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG, &[sender, self.local_addr]).and_then(|nodes| Some(nodes.iter().map(|&n| n.clone()).collect()))
                        {
                            response.set_neighbours(neighbours);
                        }

                        // store information about sender
                        let sender_info = NodeInfo::new(sender.ip(), sender.port(), request.sender_name);
                        s.add_node(sender_info);

                        // save received information about nodes
                        if let Some(neighbours) = request.neighbours {
                            neighbours.into_iter().for_each(|n| s.add_node(n));
                        }

                        Some(response)
                    });

                    // send back response
                    if let Some(encoded) = response.and_then(|r| r.serialize()) {
                        self.sock.send_to(&encoded, sender)?;
                    } else {
                        error!("response serialization failed");
                    }
                }

                Some(MsgType::ProbeResponse) => {
                    // message reception time
                    let received_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    // decode and process
                    ProbeResponse::deserialize(msg_data).and_then(|response| {
                        debug!("probe response from {}:{} (aka {})", sender.ip(), sender.port(), &response.respondent_name);

                        // storage access
                        let mut s = self.store.lock().unwrap();

                        // recompute own location based on response's RTT
                        if let Some(rtt) = received_at
                            .checked_sub(Duration::new(response.sent_at_sec, response.sent_at_nsec))
                        {
                            s.update_location(&response.location, rtt);
                        }

                        // store information about respondent
                        let mut respondent_info = NodeInfo::new(sender.ip(), sender.port(), response.respondent_name);
                        respondent_info.set_coordinates(&response.location);
                        s.add_node(respondent_info);

                        // store info about its neighbours
                        if let Some(neighbours) = response.neighbours {
                            neighbours.into_iter().for_each(|n| s.add_node(n));
                        }

                        Some(())
                    });
                }

                _ => {
                    debug!("unexpected message: {:?}", msg_data);
                }
            }
        }
    }

    /// Only respond on initial requests
    fn run_landmark(&self) -> io::Result<()> {
        let mut buff: [u8; RCV_BUFF_SIZE] = [0; RCV_BUFF_SIZE];

        loop {
            let (msg_len, sender) = self.sock.recv_from(&mut buff)?;
            let msg_data = &buff[1..msg_len];

            match MsgType::from_code(buff[0]) {
                Some(MsgType::ProbeRequest) => {
                    // respond to foreign request
                    let response = ProbeRequest::deserialize(msg_data).and_then(|request| {
                        debug!("detected probe from {}:{} (aka {})", sender.ip(), sender.port(), &request.sender_name);

                        // storage access
                        let mut s = self.store.lock().unwrap();

                        // form response
                        let mut response = ProbeResponse::new(self.name.clone(), s.get_location());

                        // send back original transmission time
                        response.copy_time(&request);

                        // store information about sender
                        let sender_info = NodeInfo::new(sender.ip(), sender.port(), request.sender_name);
                        s.add_node(sender_info);

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

                _ => debug!("unexpected message: {:?}", msg_data),
            }
        }
    }
}
