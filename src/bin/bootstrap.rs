//! Bootstrap server.
//!
//! Mandatory part of network location infrastructure.
//! Bootstrap server allows recently connected agents
//! to find its first neighbours to start communication.
//!

extern crate netloc;

use netloc::storage;
use netloc::agent::bootstrap;

fn main() {
    println!("bootstrap server not implemented!")

    // todo:
    // - use hardcoded address of landmark-agent with zero coordinates
    // - add it into the storage
    // - run UDP-server
}
