//! Landmark node
//!
//! Simple server that has hardcoded zero coordinates and never
//! recalculates it. Landmark is a mandatory part of the network location
//! infrastructure and has following objectives:
//!
//! * performs as a bootstrap, allowing recently connected agents
//! to find its first neighbours and start communication;
//! * defines zero point in local system of network coordinates;
//! * works as an anchor avoiding network coordinates drift.
//!
extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate netloc;

use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;
use std::process;

use clap::{App, Arg};
use netloc::{agent, arg_validator::*};

fn parse_args() -> Option<agent::AgentConfig> {
    let args = App::new("netloc-landmark")
        .version("0.1")
        .author("Anton Dort-Golts <dortgolts@gmail.com>")
        .about("Landmark node for the Vivaldi network coordinate system")
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
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .value_name("address")
                .help("Address of agent's informational interface")
                .takes_value(true)
                .validator(validate_address)
                .default_value("127.0.0.1:4001"),
        )
        .get_matches();

    let agent_addr = IpAddr::from_str(args.value_of("addr")?).ok()?;
    let agent_port = args.value_of("port")?.parse::<u16>().ok()?;
    let agent_name = agent::LANDMARK_AGENT_NAME.to_string();
    let log_level = args.value_of("log_level").and_then(|l| parse_log_level(l))?;
    let interface_addr = args.value_of("interface")
        .and_then(|a| a.to_socket_addrs().ok())
        .and_then(|mut a| a.next());

    let config = agent::AgentConfig {
        agent_addr,
        agent_port,
        agent_name,
        interface_addr,
        log_level,
        landmark_addr: None,
        probe_period: None,
    };

    Some(config)
}

fn main() {
    match parse_args() {
        Some(config) => {
            // init logger
            loggerv::Logger::new()
                .max_level(config.log_level)
                .level(true)
                .separator(" | ")
                .colors(true)
                .no_module_path()
                .init()
                .unwrap();

            info!(
                "{} started at {}:{}",
                config.agent_name, config.agent_addr, config.agent_port
            );

            if let Err(e) = agent::run_landmark_agent(&config) {
                error!("agent failure: {}", e);
            }
        }

        None => {
            println!("ERROR | argument parsing failed");
            process::exit(1);
        }
    }
}
