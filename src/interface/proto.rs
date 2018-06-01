//! Simple JSON-based protocol for retrieving agent's information
//!
//! Encoded messages are delimited with newline symbols
//!

use std::net::IpAddr;

use agent::{NodeInfo, NodeCoordinates, NodeList};
use storage::Node;

/* Error reasons */
pub const REASON_BAD_REQUEST: &str = "bad request";
pub const REASON_BAD_NODE_ADDR: &str = "bad node address";
pub const REASON_NODE_NOT_FOUND: &str = "node not found";
pub const REASON_NO_INFORMATION: &str = "no information";

/* Messages */

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
pub enum Request {
    GetLocation,
    GetFullMap,
    GetNodeInfo { node_addr: String },
    GetRecentNodes { max_nodes: Option<usize> },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Response {
    Location { loc: NodeCoordinates },
    FullMap { nodes: NodeList },
    NodeInfo { info: NodeInfoFull },
    RecentNodes { nodes: NodeList },

    // general unsuccessful response
    Failure { reason: &'static str },
}

/* Protocol specific structures */

#[derive(Debug, Serialize)]
pub struct NodeInfoFull {
    pub ip: IpAddr,
    pub port: u16,
    pub name: String,
    pub location: NodeCoordinates,
    pub updated_at: u64,
}

impl From<Node> for NodeInfoFull {
    fn from(node_info: Node) -> Self {
        NodeInfoFull {
            ip: node_info.info.ip,
            port: node_info.info.port,
            name: node_info.info.name,
            location: node_info.info.location,
            updated_at: node_info.last_updated_sec,
        }
    }
}
