/// Probe messages

use super::*;

// fixme: mb use monotonic Instant?
use std::time::{SystemTime, UNIX_EPOCH};

use super::byteorder::{BigEndian, ByteOrder};
use agent::proto::BinarySerializable;

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
#[derive(Debug, PartialOrd, PartialEq)]
pub struct ProbeRequest {
    pub sent_at_sec: u64,
    pub sent_at_nsec: u32,
    pub sender_name: String,
    pub neighbours: Option<NodeList>,
}

impl ProbeRequest {
    pub fn new(name: String) -> Self {
        ProbeRequest {
            sender_name: name, // fixme use &'a str
            sent_at_sec: 0,
            sent_at_nsec: 0,
            neighbours: None,
        }
    }

    /// Timestamp must be set immediately before
    /// message serialization and transmission.
    /// Use
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

impl<'a> BinarySerializable<'a> for ProbeRequest {
    type Item = Self;

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut msg_buff: Vec<u8> = Vec::new();
        let mut buff_4b: [u8; 4] = [0; 4];
        let mut buff_8b: [u8; 8] = [0; 8];

        // msg type
        msg_buff.push(types::MsgType::ProbeRequest.to_code());

        // sent_at
        BigEndian::write_u64(&mut buff_8b, self.sent_at_sec);
        msg_buff.extend(buff_8b.iter());
        BigEndian::write_u32(&mut buff_4b, self.sent_at_nsec);
        msg_buff.extend(buff_4b.iter());

        // probe initiator's name
        msg_buff.extend(serialize_str(&self.sender_name)?);

        // neighbours
        if let Some(ref neighbours) = self.neighbours {
            neighbours.iter().for_each(
                |n| msg_buff.extend(n.serialize()),
            );
        }

        Some(msg_buff)
    }

    fn deserialize(data: &'a [u8]) -> Option<Self> {
        let mut unparsed = &data[..];

        // time
        let secs = BigEndian::read_u64(&unparsed[..8]);
        let nsecs = BigEndian::read_u32(&unparsed[8..12]);
        unparsed = &unparsed[12..];

        // transmitter name
        let (transmitter_name, mut unparsed) = deserialize_str(unparsed)?;

        // create message
        let mut msg = ProbeRequest::new(transmitter_name.to_string());
        msg.sent_at_sec = secs;
        msg.sent_at_nsec = nsecs;

        while let Some((info, rest)) = NodeInfo::deserialize(unparsed) {
            if let Some(ref mut neighbours) = msg.neighbours {
                neighbours.push(info);
            } else {
                msg.neighbours = Some(vec![info]);
            }

            unparsed = rest;
        }

        Some(msg)
    }
}

/// Network RTT-probe response.
///
/// +----------+------------+-----------------+--------------------+-------------------------------------------+
/// | MSG_TYPE |   sent_at  | respondent name | node's coordinates |   information about 0-4 random neighbours |
/// +----------+-----+------+-----------------+--------------------+----------+----------+----------+----------+
/// |    u8    | sec | nsec |       str       |   NodeCoordinates  | NodeInfo | NodeInfo | NodeInfo | NodeInfo |
/// +----------+-----+------+-----------------+--------------------+----------+----------+----------+----------+
/// |    8     |  64 |  32  |     1 - 255     |         192        |                   var                     |
/// +----------+------------+-----------------+--------------------+-------------------------------------------+
///
/// Remote node's response includes as well information about up to 4 its neighbour nodes
///
#[derive(Debug, PartialOrd, PartialEq)]
pub struct ProbeResponse {
    pub sent_at_sec: u64,
    pub sent_at_nsec: u32,
    pub respondent_name: String,
    pub location: NodeCoordinates,
    pub neighbours: Option<NodeList>,
}

impl ProbeResponse {
    pub fn new(name: String, location: NodeCoordinates) -> Self {
        ProbeResponse {
            respondent_name: name,
            sent_at_sec: 0,
            sent_at_nsec: 0,
            location,
            neighbours: None,
        }
    }

    pub fn set_neighbours(&mut self, neighbours: NodeList) {
        self.neighbours = Some(neighbours);
    }

    pub fn copy_time(&mut self, request: &ProbeRequest) {
        self.sent_at_sec = request.sent_at_sec;
        self.sent_at_nsec = request.sent_at_nsec;
    }
}

impl<'de> BinarySerializable<'de> for ProbeResponse {
    type Item = ProbeResponse;

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut msg_buff: Vec<u8> = Vec::new();
        let mut buff_4b: [u8; 4] = [0; 4];
        let mut buff_8b: [u8; 8] = [0; 8];

        // msg type
        msg_buff.push(types::MsgType::ProbeResponse.to_code());

        // sent_at
        BigEndian::write_u64(&mut buff_8b, self.sent_at_sec);
        msg_buff.extend(buff_8b.iter());
        BigEndian::write_u32(&mut buff_4b, self.sent_at_nsec);
        msg_buff.extend(buff_4b.iter());

        // probe respondent's name
        msg_buff.extend(serialize_str(&self.respondent_name)?);

        // coordinates
        [
            self.location.x1,
            self.location.x2,
            self.location.height,
            self.location.pos_err,
        ].iter()
            .for_each(|e| {
                BigEndian::write_f32(&mut buff_4b, *e);
                msg_buff.extend(buff_4b.iter())
            });
        // iteration
        BigEndian::write_u64(&mut buff_8b, self.location.iteration);
        msg_buff.extend(buff_8b.iter());

        // neighbours
        if let Some(ref neighbours) = self.neighbours {
            neighbours.iter().for_each(
                |n| msg_buff.extend(n.serialize()),
            );
        }

        Some(msg_buff)
    }

    fn deserialize(data: &'de [u8]) -> Option<Self> {
        let mut unparsed = &data[..];

        // time
        let secs = BigEndian::read_u64(&unparsed[..8]);
        let nsecs = BigEndian::read_u32(&unparsed[8..12]);
        unparsed = &unparsed[12..];

        // transmitter name
        let (respondent_name, mut unparsed) = deserialize_str(unparsed)?;

        // bytes required to decode coordinates
        if unparsed.len() < 24 {
            return None;
        }

        // parse coordinates
        let respondent_location = NodeCoordinates {
            x1: BigEndian::read_f32(&unparsed[..4]),
            x2: BigEndian::read_f32(&unparsed[4..8]),
            height: BigEndian::read_f32(&unparsed[8..12]),
            pos_err: BigEndian::read_f32(&unparsed[12..16]),
            iteration: BigEndian::read_u64(&unparsed[16..24]),
        };

        unparsed = &unparsed[24..];

        // create message
        let mut msg = ProbeResponse::new(respondent_name.to_string(), respondent_location);
        // set time
        msg.sent_at_sec = secs;
        msg.sent_at_nsec = nsecs;

        while let Some((info, rest)) = NodeInfo::deserialize(unparsed) {
            if let Some(ref mut neighbours) = msg.neighbours {
                neighbours.push(info);
            } else {
                msg.neighbours = Some(vec![info]);
            }

            unparsed = rest;
        }

        Some(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_homomorphism_probe_request() {
        let mut req = ProbeRequest::new("test_node".to_string());
        req.set_current_time();

        let encoded = req.serialize().unwrap();
        let decoded = ProbeRequest::deserialize(&encoded[1..]).unwrap();

        assert_eq!(req, decoded);
    }

    #[test]
    fn codec_homomorphism_probe_response() {
        let mut req = ProbeRequest::new("test_node".to_string());
        req.set_current_time();

        let location = NodeCoordinates {
            x1: 1.5,
            x2: 23.65,
            height: 0.34,
            pos_err: 0.5,
            iteration: 127,
        };

        let mut resp = ProbeResponse::new("respondent_node".to_string(), location);
        resp.copy_time(&req);

        let encoded = resp.serialize().unwrap();
        let decoded = ProbeResponse::deserialize(&encoded[1..]).unwrap();

        assert_eq!(resp, decoded);
    }
}
