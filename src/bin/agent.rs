extern crate clap;
/// Locator agent service executable.
///
/// Run UDP-based location agent as well
/// as JSON-over-TCP interface server.
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate netloc;

use std::process;
use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

use clap::{App, Arg};

use netloc::{agent, arg_validator::*};

// fixme: parse for real and use errors (failure crate?)
fn parse_args() -> Option<agent::AgentConfig> {
    let args = App::new("netloc-agent")
        .version("0.1")
        .author("Anton Dort-Golts dortgolts@gmail.com")
        .about("Regular positioning agent for the Vivaldi network coordinate system")
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
                .validator(validate_port)
                .default_value("3737"),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("name")
                .help("Name of the agent")
                .takes_value(true)
                .validator(validate_name)
                .default_value(""),
        )
        .arg(
            Arg::with_name("period")
                .short("r")
                .long("probe")
                .value_name("period")
                .help("Probe period in seconds")
                .takes_value(true)
                .validator(validate_interval)
                .default_value("20"),
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
            Arg::with_name("bootstrap")
                .short("b")
                .long("bootstrap")
                .value_name("address")
                .help("Address of bootstrap server")
                .takes_value(true)
                .required(true)
                .validator(validate_address),
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

    // argument conversions
    let agent_addr = IpAddr::from_str(args.value_of("addr")?).ok()?;
    let agent_port = args.value_of("port")?.parse::<u16>().ok()?;
    let agent_name = args.value_of("name")?.to_string();
    let probe_period = args.value_of("period")
        .and_then(|p| p.parse::<u16>().ok())
        .and_then(|t| Some(Duration::new(t as u64, 0)));
    let bootstrap_addr = args.value_of("bootstrap")
        .and_then(|a| a.to_socket_addrs().ok())
        .and_then(|mut a| a.next())?;
    let interface_addr = args.value_of("interface")
        .and_then(|a| a.to_socket_addrs().ok())
        .and_then(|mut a| a.next());

    let log_level = args.value_of("log_level").and_then(|l| parse_log_level(l))?;

    let config = agent::AgentConfig {
        agent_addr,
        agent_port,
        agent_name,
        probe_period,
        interface_addr,
        bootstrap_addr: Some(bootstrap_addr),
        log_level,
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
                "regular agent started at {}:{}",
                config.agent_addr, config.agent_port
            );

            if let Err(e) = agent::run_regular_agent(&config) {
                panic!("agent failure: {}", e);
            }
        }

        None => {
            println!("ERROR | cannot parse config options");
            process::exit(1);
        }
    }
}
