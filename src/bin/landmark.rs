//! Landmark agent
//!
//! Simple agent that has hardcoded zero coordinates and never
//! recalculate it. This agent could be used as an anchor
//! for drifting network coordinates.
//!

extern crate netloc;

use netloc::agent;

fn main() {

    // todo parse CLI args

    println!("INFO | starting landmark agent");

    // todo use config to send parameters in agent
    if let Err(e) = agent::run_landmark_agent() {
        println!("ERROR | agent failure: {}", e);
    }
}
