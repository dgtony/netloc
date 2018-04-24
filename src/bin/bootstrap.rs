//! Bootstrap server.
//!
//! Mandatory part of network location infrastructure.
//! Bootstrap server allows recently connected agents
//! to find its first neighbours to start communication.
//!
extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate netloc;

use std::io;
use std::process;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::str::FromStr;

use clap::{App, Arg};

use netloc::{storage, arg_validator::*};
use netloc::agent::bootstrap::{BootstrapRequest, BootstrapResponse};
use netloc::agent::{BinarySerializable, MsgType, NodeInfo, GOSSIP_MAX_NEIGHBOURS_IN_MSG,
                    LANDMARK_AGENT_NAME};

const RCV_BUFF_SIZE: usize = 1500;

struct Config {
    addr: SocketAddr,
    landmark_addr: SocketAddr,
    log_level: log::Level,
}

fn parse_args() -> Option<Config> {
    let args = App::new("netloc-bootstrap")
        .version("0.1")
        .author("Anton Dort-Golts dortgolts@gmail.com")
        .about("Bootstrap server for the Vivaldi network coordinate system")
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
                .default_value("3739"),
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
            Arg::with_name("landmark")
                .short("m")
                .long("landmark")
                .value_name("address")
                .help("Full address of landmark agent")
                .takes_value(true)
                .required(true)
                .validator(validate_address),
        )
        .get_matches();

    let agent_ip = IpAddr::from_str(args.value_of("addr")?).ok()?;
    let agent_port = args.value_of("port")?.parse::<u16>().ok()?;
    let log_level = args.value_of("log_level").and_then(|l| parse_log_level(l))?;
    let landmark_addr = args.value_of("landmark")
        .and_then(|a| a.to_socket_addrs().ok())
        .and_then(|mut a| a.next())?;

    let config = Config {
        addr: SocketAddr::new(agent_ip, agent_port),
        landmark_addr,
        log_level,
    };

    Some(config)
}

fn run_server(config: &Config, store: &mut storage::Storage) -> io::Result<()> {
    let mut buff: [u8; RCV_BUFF_SIZE] = [0; RCV_BUFF_SIZE];
    let sock = UdpSocket::bind(config.addr)?;

    loop {
        let (msg_len, sender) = sock.recv_from(&mut buff)?;
        let msg_data = &buff[1..msg_len];

        match MsgType::from_code(buff[0]) {
            Some(MsgType::BootstrapReq) => {
                BootstrapRequest::deserialize(msg_data).and_then(|msg| {
                    // store requesting node
                    store.add_node(NodeInfo::new(sender.ip(), sender.port(), msg.local_name));

                    // construct response
                    let response = {
                        // there must be always at least one node (landmark) in storage
                        let nodes = store
                            .get_random_nodes(GOSSIP_MAX_NEIGHBOURS_IN_MSG, &[sender])
                            .unwrap();

                        let mut response = BootstrapResponse::empty();
                        for n in nodes {
                            response.neighbours.push(n.clone());
                        }

                        response
                    };

                    // send back response
                    response
                        .serialize()
                        .and_then(|encoded| Some(sock.send_to(&encoded, sender)))
                });
            }

            _ => println!("DEBUG | received unknown message: {:?}", msg_data),
        }
    }
}

fn main() {
    match parse_args() {
        Some(config) => {
            let landmark_info = NodeInfo::new(
                config.landmark_addr.ip(),
                config.landmark_addr.port(),
                LANDMARK_AGENT_NAME.to_string(),
            );

            // init logger
            loggerv::Logger::new()
                .max_level(config.log_level)
                .level(true)
                .separator(" | ")
                .colors(true)
                .no_module_path()
                .init()
                .unwrap();

            // init storage
            let mut store = storage::Storage::new();
            store.add_node(landmark_info);

            info!("bootstrap server started at {}", config.addr);

            if let Err(e) = run_server(&config, &mut store) {
                error!("bootstrap server failure: {}", e);
                process::exit(1);
            }
        }

        None => {
            println!("ERROR | cannot parse config options");
            process::exit(1);
        }
    }
}
