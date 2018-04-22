/// Locator agent service executable.
///
/// Run UDP-based location agent as well
/// as JSON-over-TCP interface server.
extern crate netloc;

use std::thread;
use netloc::agent;

fn main() {
    // todo parse CLI args

    // todo use config to send parameters in agent
    if let Err(e) = agent::run_agent() {
        println!("ERROR | agent failure: {}", e);
    }
}
