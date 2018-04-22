/// Locator agent service executable.
///
/// Run UDP-based location agent as well
/// as JSON-over-TCP interface server.
extern crate netloc;

use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use netloc::agent;

// fixme: parse for real and use errors (failure crate?)
fn parse_args() -> Option<agent::AgentConfig> {
    let node_name = String::from("test_node");
    let agent_ip_addr = IpAddr::from_str("0.0.0.0").ok()?;
    let agent_port: u16 = 3737;
    let bootstrap_ip_addr = IpAddr::from_str("127.0.0.1").ok()?;
    let bootstrap_port: u16 = 3739;
    let probe_interval = Duration::new(10, 0);

    let config = agent::AgentConfig {
        agent_addr: agent_ip_addr,
        agent_port: agent_port,
        agent_name: node_name,
        probe_period: Some(probe_interval),
        interface_addr: None,
        interface_port: None,
        bootstrap_addr: Some(bootstrap_ip_addr),
        bootstrap_port: Some(bootstrap_port),
    };

    Some(config)
}

fn main() {
    // todo parse CLI args

    match parse_args() {
        Some(config) => {
            //
            println!("INFO | starting regular agent");

            if let Err(e) = agent::run_regular_agent(&config) {
                println!("ERROR | agent failure: {}", e);
            }
        }

        None => println!("ERROR | cannot parse config options"),
    }
}
