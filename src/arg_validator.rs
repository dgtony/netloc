//! Validate CLI arguments
//!

use std::net::ToSocketAddrs;
use log;

pub fn validate_name(name: String) -> Result<(), String> {
    if name.as_bytes().len() > 254 {
        return Err(String::from("Provided name is too long"));
    }

    Ok(())
}

pub fn validate_interval(interval: String) -> Result<(), String> {
    match interval.parse::<u16>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Bad probe interval provided")),
    }
}

pub fn validate_address(addr: String) -> Result<(), String> {
    match addr.to_socket_addrs() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Bad address provided")),
    }
}

pub fn validate_port(port: String) -> Result<(), String> {
    match port.parse::<u16>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Bad port provided")),
    }
}

pub fn parse_log_level(level: &str) -> Option<log::Level> {
    match level {
        "debug" => Some(log::Level::Debug),
        "info" => Some(log::Level::Info),
        "warn" => Some(log::Level::Warn),
        "error" => Some(log::Level::Error),
        _ => None,
    }
}
