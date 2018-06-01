/// Request processing module
///
use std::net::SocketAddr;

use agent::NodeList;
use storage::SharedStorage;
use super::proto::{Request, Response, NodeInfoFull};
use super::proto::{REASON_NODE_NOT_FOUND, REASON_BAD_NODE_ADDR, REASON_NO_INFORMATION};

const NUM_RECENT_NODES_DEFAULT: usize = 10;
const ERR_LOCK_FAILED: &str = "lock failed";

pub fn process_request(request: Request, store: &mut SharedStorage) -> Response {
    debug!("get request: {:?}", request);
    match request {
        Request::GetLocation => {
            let location = store.lock().expect(ERR_LOCK_FAILED).get_location();
            Response::Location { loc: location }
        }

        Request::GetFullMap => {
            let nodes = store.lock().expect(ERR_LOCK_FAILED).get_all_nodes();
            Response::FullMap { nodes }
        }

        Request::GetNodeInfo { node_addr } => {
            node_addr
                .parse()
                .and_then(|addr| match store
                    .lock()
                    .expect(ERR_LOCK_FAILED)
                    .find_node(addr) {
                    Some(info) => Ok(Response::NodeInfo { info: NodeInfoFull::from(info) }),
                    None => Ok(Response::Failure { reason: REASON_NODE_NOT_FOUND }),
                })
                .unwrap_or(Response::Failure { reason: REASON_BAD_NODE_ADDR })
        }

        Request::GetRecentNodes { max_nodes } => {
            match store.lock().expect(ERR_LOCK_FAILED).get_most_recent(
                max_nodes.unwrap_or(
                    NUM_RECENT_NODES_DEFAULT,
                ),
            ) {
                Some(info) => Response::RecentNodes {
                    nodes: info.iter().map(|&n| n.clone()).collect(),
                },
                None => Response::Failure { reason: REASON_NO_INFORMATION },
            }
        }
    }
}
