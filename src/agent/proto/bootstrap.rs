/// Bootstrap messages

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::*;

/// First message while storage is empty
///
/// +----------+-------------------+
/// | MSG_TYPE | local node's name |
/// +----------+-------------------+
/// |    u8    |    >= [u8; 255]   |
/// +----------+-------------------+
///
/// Send it to bootstrap server in order to obtain some neighbour's addresses
#[derive(Debug, PartialOrd, PartialEq)]
pub struct BootstrapRequest {
    local_name: String,
}

impl BootstrapRequest {
    pub fn new(local_name: String) -> Self {
        BootstrapRequest { local_name }
    }
}

impl <'a> BinarySerializable<'a> for BootstrapRequest {
    type Item = Self;

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut name_serialized = serialize_str(&self.local_name)?;

        // insert msg code to create request payload
        name_serialized.insert(0, types::MsgType::BootstrapReq.to_code());

        Some(name_serialized)
    }

    fn deserialize(data: &'a [u8]) -> Option<Self> {
        let (name, _) = deserialize_str(data)?;
        Some(BootstrapRequest { local_name: name.to_string() })
    }
}

/// Bootstrap server response
///
/// +----------+-------------------------------------------+
/// | MSG_TYPE |    information about 0-4 random nodes     |
/// +----------+----------+----------+----------+----------+
/// |    u8    | NodeInfo | NodeInfo | NodeInfo | NodeInfo |
/// +----------+----------+----------+----------+----------+
///
/// Number of NodeInfo records in response depends on number of
/// active nodes in network and bootstrap sever awareness.
///
#[derive(Debug, PartialOrd, PartialEq)]
pub struct BootstrapResponse {
    neighbours: NodeList,
}

impl BootstrapResponse {
    pub fn empty() -> Self {
        BootstrapResponse {
            neighbours: Vec::new(),
        }
    }
}

impl<'de> BinarySerializable<'de> for BootstrapResponse {
    type Item = Self;

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut msg_buff = Vec::new();
        msg_buff.insert(0, types::MsgType::BootstrapResp.to_code());
        // serialize neighbours
        self.neighbours
            .iter()
            .for_each(|n| msg_buff.extend(n.serialize()));

        Some(msg_buff)
    }

    fn deserialize(data: &'de [u8]) -> Option<Self> {
        let mut msg = BootstrapResponse::empty();
        let mut unparsed = &data[..];

        while let Some((info, rest)) = NodeInfo::deserialize(unparsed) {
            // add node info
            msg.neighbours.push(info);
            // move buffer
            unparsed = rest;
        }

        Some(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_bootstrap_request() {
        let req = BootstrapRequest::new("test".to_string());
        assert_eq!(req.serialize(), Some(vec![1, 4, 116, 101, 115, 116]));

        let empty = BootstrapRequest::new("".to_string());
        assert_eq!(empty.serialize(), Some(vec![1, 0]));
    }

    #[test]
    fn codec_homomorphism_bootstrap_request() {
        let req = BootstrapRequest::new("test_node".to_string());

        let encoded = req.serialize().unwrap();
        let decoded = BootstrapRequest::deserialize(&encoded[1..]).unwrap();

        assert_eq!(req, decoded);
    }

    #[test]
    fn codec_homomorphism_bootstrap_response() {
        let nodes = vec![
            NodeInfo::new(
                IpAddr::from(Ipv4Addr::new(1, 2, 3, 4)),
                1001,
                "first".to_string(),
            ),
            NodeInfo::new(
                IpAddr::from(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8)),
                1002,
                "second".to_string(),
            ),
            NodeInfo::new(
                IpAddr::from(Ipv4Addr::new(10, 252, 33, 17)),
                1003,
                "third".to_string(),
            ),
        ];
        let resp = BootstrapResponse { neighbours: nodes };

        let encoded = resp.serialize().unwrap();
        let decoded = BootstrapResponse::deserialize(&encoded[1..]).unwrap();

        assert_eq!(resp, decoded);
    }
}
