//! Simple JSON-based protocol for retrieving agent's information
//!
//! Encoded messages are delimited with newline symbols
//!

use agent::NodeInfo;
use storage::Node;

/* Error reasons */
pub const REASON_BAD_MESSAGE: &str = "bad message";

/* Messages */

// todo use enum of requests

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actions {
    GetLocation,
    GetNodeInfo,
    GetRecentNodes,
    GetFullMap,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub action: Actions,
    pub node_addr: Option<String>,
    pub max_nodes: Option<usize>,
}

// todo use enum of various responses

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag="type")]
pub enum Response {
    Location {
        // todo add payload
        msg: String,
    },

    NodeInfo {
        // todo add payload
        msg: String,
    },

    // todo add all variants

    Failure {
        reason: Option<&'static str>,
    }
}