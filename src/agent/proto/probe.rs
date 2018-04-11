/// Probe messages

use super::*;

// fixme: mb use monotonic Instant?
use std::time::{SystemTime, UNIX_EPOCH};

/// Periodic request sent to random neighbour in order
/// to measure its RTT.
///
/// +----------+------------+-------------+-------------------------------------------+
/// | MSG_TYPE |   sent_at  | sender name |   information about 0-4 random neighbours |
/// +----------+-----+------+-------------+           known to local node             |
/// |    u8    | sec | nsec |     str     |                                           |
/// +----------+-----+------+-------------+----------+----------+----------+----------+
/// |    8     |  64 |  32  |   1 - 255   | NodeInfo | NodeInfo | NodeInfo | NodeInfo |
/// +----------+------------+-------------+----------+----------+----------+----------+
///
pub struct ProbeRequest<'a> {
    transmitter_name: &'a str,
    sent_at_sec: u64,
    sent_at_nsec: u32,
    neighbours: Option<NodeList>,
}

impl <'a> ProbeRequest<'a> {
    pub fn new(name: &'a str) -> Self {
        ProbeRequest {
            transmitter_name: name,
            sent_at_sec: 0,
            sent_at_nsec: 0,
            neighbours: None,
        }
    }

    /// Timestamp must be set immediately before
    /// message serialization and transmission
    pub fn set_current_time(&mut self) {
        if let Ok(t) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.sent_at_sec = t.as_secs();
            self.sent_at_nsec = t.subsec_nanos();
        }
    }

    pub fn set_neighbours(&mut self, neighbours: NodeList) {
        self.neighbours = Some(neighbours);
    }
}

fn s() {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = t.as_secs(); // could work almost forever :)
    let nsec = t.subsec_nanos();
}

/// Network RTT-probe response.
///
/// +----------+------------+-----------------+--------------------+-------------------------------------------+
/// | MSG_TYPE |   sent_at  | respondent name | node's coordinates |   information about 0-4 random neighbours |
/// +----------+-----+------+-----------------+--------------------+----------+----------+----------+----------+
/// |    u8    | sec | nsec |       str       |   NodeCoordinates  | NodeInfo | NodeInfo | NodeInfo | NodeInfo |
/// +----------+-----+------+-----------------+--------------------+----------+----------+----------+----------+
/// |    8     |  64 |  32  |     1 - 255     |         256        |                   var                     |
/// +----------+------------+-----------------+--------------------+-------------------------------------------+
///
/// Remote node's response includes information about up to 4 neighbour nodes
///
pub struct ProbeResponse {

}