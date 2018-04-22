//! Landmark agent
//!
//! Simple agent that has hardcoded zero coordinates and never
//! recalculate it. This agent could be used as an anchor
//! for drifting network coordinates.
//!

extern crate netloc;

use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use netloc::agent;

// fixme: parse for real and use errors (failure crate?)
fn parse_args() -> Option<agent::AgentConfig> {
    let agent_ip_addr = IpAddr::from_str("127.0.0.1").ok()?;
    let agent_port: u16 = 3738;

    let config = agent::AgentConfig {
        agent_addr: agent_ip_addr,
        agent_port: agent_port,
        agent_name: String::new(),
        probe_period: None,
        interface_addr: None,
        interface_port: None,
        bootstrap_addr: None,
        bootstrap_port: None,
    };

    Some(config)
}

fn main() {
    // todo parse CLI args

    match parse_args() {
        Some(config) => {
            println!("INFO | starting landmark agent");

            // todo use config to send parameters in agent
            if let Err(e) = agent::run_landmark_agent(&config) {
                println!("ERROR | agent failure: {}", e);
            }
        }

        None => println!("ERROR | cannot parse config options"),
    }
}
