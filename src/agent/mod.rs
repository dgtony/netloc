//! Location nodes
//!
//! Agents perform all communication between nodes:
//! - RTT probes
//! - computation of coordinates
//! - overlay network discovery (Gossip)
//!
//! Landmark node always sustain zero coordinates,
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
use super::interface;
use self::transmitter::Transmitter;
use self::receiver::Receiver;

use log;
use std::io;
use std::thread;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Duration;

pub const GOSSIP_MAX_NEIGHBOURS_IN_MSG: usize = 4;
pub const LANDMARK_NODE_NAME: &str = "landmark";

pub enum NodeType {
    Regular,
    Landmark,
}

#[derive(Debug)]
pub struct NodeConfig {
    pub node_addr: IpAddr,
    pub node_port: u16,
    pub node_name: String,
    pub probe_period: Option<Duration>,
    pub interface_addr: Option<SocketAddr>,
    pub landmark_addr: Option<SocketAddr>,
    pub log_level: log::Level,
}

pub fn run_agent(config: &NodeConfig) -> io::Result<()> {
    check_interface_addr(config)?;

    let node_name = config.node_name.clone();

    // shared parameters
    let sock = UdpSocket::bind((config.node_addr, config.node_port))?;
    let store = Arc::new(Mutex::new(Storage::new()));

    // run transmitter in separate thread
    let _tx_thread = {
        let node_name = node_name.clone();
        let store = store.clone();
        let sock = sock.try_clone().expect("cannot clone socket");
        let period = config
            .probe_period
            .expect("probe period not specified")
            .clone();
        let landmark_addr = config.landmark_addr.unwrap().clone();

        thread::spawn(move || {
            let t = Transmitter::new(node_name, landmark_addr, store, sock, period);

            if let Err(e) = t.run() {
                panic!("agent-transmitter failure: {}", e);
            }
        })
    };

    // run receiver in separate thread
    let rx_thread = {
        let node_name = node_name.clone();
        let store = store.clone();
        let sock = sock.try_clone().expect("cannot clone socket");
        let landmark = config.landmark_addr.clone();

        thread::spawn(move || {
            let r = Receiver::new(NodeType::Regular, node_name, store, sock, landmark);
            if let Err(e) = r.run() {
                panic!("agent-receiver failure: {}", e);
            }
        })
    };

    // run interface server
    interface::run_server(config.interface_addr.unwrap(), store);

    Ok(())
}

pub fn run_landmark(config: &NodeConfig) -> io::Result<()> {
    check_interface_addr(config)?;

    let mut store = Storage::new();
    store.set_location(NodeCoordinates {
        pos_err: 0.0,
        ..Default::default()
    });
    let store = Arc::new(Mutex::new(store));

    // run receiver in separate thread
    let rx_thread = {
        let store = store.clone();
        let sock = UdpSocket::bind((config.node_addr, config.node_port))?;
        let name = config.node_name.clone();

        thread::spawn(move || {
            let r = Receiver::new(NodeType::Landmark, name, store, sock, None);
            if let Err(e) = r.run() {
                panic!("node failure: {}", e);
            }
        })
    };

    // run interface server
    interface::run_server(config.interface_addr.unwrap(), store);

    Ok(())
}


fn check_interface_addr(config: &NodeConfig) -> io::Result<()> {
    match config.interface_addr {
        Some(_) => Ok(()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            "bad interface address provided",
        )),
    }
}
