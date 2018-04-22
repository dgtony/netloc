//! Landmark agent
//!
//! Simple agent that has hardcoded zero coordinates and never
//! recalculate it. This agent could be used as an anchor
//! for drifting network coordinates.
//!

extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;
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

fn parse_log_level(level: &str) -> Option<log::Level> {
    match level {
        "debug" => Some(log::Level::Debug),
        "info" => Some(log::Level::Info),
        "warn" => Some(log::Level::Warn),
        "error" => Some(log::Level::Error),
        _ => None,
    }
}

fn main() {
    let args = App::new("netloc-landmark")
        .version("0.1")
        .author("Anton Dort-Golts dortgolts@gmail.com")
        .about("Landmark agent for the Vivaldi network coordinate system")
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("IP-address")
                .help("IP-address to listen on")
                .takes_value(true)
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("UDP-port")
                .help("UDP-port to listen on")
                .takes_value(true)
                .default_value("3738"),
        )
        .arg(
            Arg::with_name("log_level")
                .short("l")
                .long("log")
                .value_name("level")
                .help("logging level")
                .takes_value(true)
                .possible_values(&["debug", "info", "warn", "error"])
                .default_value("info"),
        )
        .get_matches();

    let agent_addr = args.value_of("addr").and_then(|a| IpAddr::from_str(a).ok());
    let agent_port = args.value_of("port").unwrap().parse::<u16>();
    let log_level = args.value_of("log_level")
        .and_then(|l| parse_log_level(l))
        .unwrap();

    // init logger
    loggerv::Logger::new()
        .max_level(log_level)
        .level(true)
        .separator(" | ")
        .colors(true)
        .no_module_path()
        .init()
        .unwrap();

    // validation
    if agent_addr.is_none() {
        error!("bad IP address: {}", args.value_of("addr").unwrap());
        process::exit(1);
    }
    if let Err(_) = agent_port {
        error!("bad port value: {}", args.value_of("port").unwrap());
        process::exit(1);
    }
    if agent_addr.is_none() {
        error!("bad IP address: {}", args.value_of("addr").unwrap());
        process::exit(1);
    }

    let agent_addr = agent_addr.unwrap();
    let agent_port = agent_port.unwrap();

    let config = parse_args(agent_addr, agent_port);
    info!("landmark agent started at {}:{}", agent_addr, agent_port);

    if let Err(e) = agent::run_landmark_agent(&config) {
        error!("agent failure: {}", e);
    }
}
