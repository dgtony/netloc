/// Location agent
///
/// Perform all communication between nodes:
/// - RTT probes
/// - computation of coordinates
/// - overlay network discovery (Gossip)

mod receiver;
mod transmitter;
mod proto;

pub use self::proto::*;

use super::storage;
