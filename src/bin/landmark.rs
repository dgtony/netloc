//! Landmark agent
//!
//! Simple agent that has hardcoded zero coordinates and never
//! recalculate it. This agent could be used as an anchor
//! for drifting network coordinates.
//!

extern crate clap;
extern crate netloc;

use std::net::IpAddr;
use std::str::FromStr;
use std::process;
use std::time::Duration;

use clap::{App, Arg};
use netloc::agent;

fn parse_args(agent_addr: IpAddr, agent_port: u16) -> agent::AgentConfig {
    let config = agent::AgentConfig {
        agent_addr,
        agent_port,
        agent_name: String::new(),
        probe_period: None,
        interface_addr: None,
        interface_port: None,
        bootstrap_addr: None,
        bootstrap_port: None,
    };

    config
}

fn main() {
    let args = clap::App::new("netloc-landmark")
        .version("0.1")
        .author("Anton Dort-Golts dortgolts@gmail.com")
        .about("Landmark agent for the Vivaldi network coordinate system")
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .help("IP-address to listen on")
                .takes_value(true)
                .default_value("0.0.0.0"),
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .help("UDP-port to listen on")
                .takes_value(true)
                .default_value("3738"),
        )
        .get_matches();

    let agent_addr = args.value_of("addr").and_then(|a| IpAddr::from_str(a).ok());
    let agent_port = args.value_of("port").unwrap().parse::<u16>();

    // validation
    if agent_addr.is_none() {
        println!("ERROR | bad IP address: {}", args.value_of("addr").unwrap());
        process::exit(1);
    }
    if let Err(_) = agent_port {
        println!("ERROR | bad port value: {}", args.value_of("port").unwrap());
        process::exit(1);
    }

    let config = parse_args(agent_addr.unwrap(), agent_port.unwrap());
    println!("INFO | starting landmark agent");

    if let Err(e) = agent::run_landmark_agent(&config) {
        println!("ERROR | agent failure: {}", e);
    }
}
