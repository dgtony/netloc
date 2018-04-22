//! Location agent
//!
//! Regular agents perform all communication between nodes:
//! - RTT probes
//! - computation of coordinates
//! - overlay network discovery (Gossip)
//!
//! Landmark agent always sustain zero coordinates,
//! only responding to foreign requests, collecting
//! and spreading information about new nodes.
//!
//! NB: there must be ONLY ONE landmark agent in the network!!!
//!

mod receiver;
mod transmitter;
mod proto;
pub mod vivaldi;

pub use self::proto::*;

use super::storage::Storage;
use self::transmitter::Transmitter;
use self::receiver::Receiver;

use std::io;
use std::thread;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Duration;

pub const GOSSIP_MAX_NEIGHBOURS_IN_MSG: usize = 4;

pub const LANDMARK_AGENT_NAME: &str = "landmark-agent";

pub enum AgentType {
    Regular,
    Landmark,
}

pub fn run_regular_agent() -> io::Result<()> {
    // todo read from config
    let node_name = String::from("test_node");
    let agent_ip_addr = "0.0.0.0";
    let agent_port: u16 = 3737;
    let bootstrap_ip_addr = "127.0.0.1";
    let bootstrap_port: u16 = 3739;
    let probe_interval = Duration::new(10, 0);

    // shared parameters
    let bootstrap_addr = SocketAddr::new(
        IpAddr::from_str(bootstrap_ip_addr)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        bootstrap_port,
    );
    let sock = UdpSocket::bind(format!("{}:{}", agent_ip_addr, agent_port))?;
    let store = Arc::new(Mutex::new(Storage::new()));

    // run transmitter in separate thread
    let _tx_thread = {
        let node_name = node_name.clone();
        let store = store.clone();
        let sock = sock.try_clone().expect("cannot clone socket");

        thread::spawn(move || {
            let t = transmitter::Transmitter::new(
                node_name,
                bootstrap_addr,
                store,
                sock,
                probe_interval,
            );
            if let Err(e) = t.run() {
                println!("ERROR | agent-transmitter failure: {}", e);
            }
        })
    };

    // run receiver in separate thread
    let rx_thread = {
        let node_name = node_name.clone();
        let store = store.clone();
        let sock = sock.try_clone().expect("cannot clone socket");

        thread::spawn(move || {
            let r = Receiver::new(AgentType::Regular, node_name, store, sock);
            if let Err(e) = r.run() {
                println!("ERROR | agent-receiver failure: {}", e);
            }
        })
    };

    // - run interface server

    // todo remove
    rx_thread.join();

    Ok(())
}

pub fn run_landmark_agent() -> io::Result<()> {
    // todo read from config
    let agent_ip_addr = "0.0.0.0";
    let agent_port: u16 = 3738;

    let store = Arc::new(Mutex::new(Storage::new()));

    // run receiver in separate thread
    let rx_thread = {
        let store = store.clone();
        let sock = UdpSocket::bind(format!("{}:{}", agent_ip_addr, agent_port))?;

        thread::spawn(move || {
            let r = Receiver::new(AgentType::Landmark, LANDMARK_AGENT_NAME.to_string(), store, sock);
            if let Err(e) = r.run() {
                println!("ERROR | landmark-agent failure: {}", e);
            }
        })
    };

    // - run interface server ?

    // todo remove
    rx_thread.join();

    Ok(())
}
