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

pub struct AgentConfig {
    pub agent_addr: IpAddr,
    pub agent_port: u16,
    pub agent_name: String,
    pub probe_period: Option<Duration>,
    pub interface_addr: Option<IpAddr>,
    pub interface_port: Option<u16>,
    pub bootstrap_addr: Option<IpAddr>,
    pub bootstrap_port: Option<u16>,
    //pub log_level: // todo
}

pub fn run_regular_agent(config: &AgentConfig) -> io::Result<()> {
    let node_name = config.agent_name.clone();

    // shared parameters
    let bootstrap_addr = SocketAddr::new(
        config
            .bootstrap_addr
            .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
        config
            .bootstrap_port
            .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
    );

    let sock = UdpSocket::bind((config.agent_addr, config.agent_port))?;
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

        thread::spawn(move || {
            let t = transmitter::Transmitter::new(node_name, bootstrap_addr, store, sock, period);

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

pub fn run_landmark_agent(config: &AgentConfig) -> io::Result<()> {
    let store = Arc::new(Mutex::new(Storage::new()));

    // run receiver in separate thread
    let rx_thread = {
        let store = store.clone();
        let sock = UdpSocket::bind((config.agent_addr, config.agent_port))?;

        thread::spawn(move || {
            let r = Receiver::new(
                AgentType::Landmark,
                LANDMARK_AGENT_NAME.to_string(),
                store,
                sock,
            );
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
