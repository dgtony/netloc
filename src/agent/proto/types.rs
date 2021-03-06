/// Primitive and composite types and structures
/// used in protocol messages.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use super::byteorder::{BigEndian, ByteOrder};
use super::*;

pub enum MsgType {
    ProbeRequest,
    ProbeResponse,
}

impl MsgType {
    pub fn to_code(&self) -> u8 {
        match *self {
            MsgType::ProbeRequest => 1,
            MsgType::ProbeResponse => 2,
        }
    }

    pub fn from_code(code: u8) -> Option<MsgType> {
        match code {
            1 => Some(MsgType::ProbeRequest),
            2 => Some(MsgType::ProbeResponse),

            _ => None,
        }
    }
}

/* Node information */

pub type NodeList = Vec<NodeInfo>;

#[derive(Debug, PartialOrd, PartialEq, Clone, Serialize)]
pub struct NodeFlags {
    is_addr_ipv6: bool,
}

impl NodeFlags {
    #[allow(dead_code)]
    fn serialize(&self) -> u8 {
        let is_addr_ipv6_flag: u8 = if self.is_addr_ipv6 { 1 } else { 0 };

        // concat flags
        let flags: u8 = is_addr_ipv6_flag;
        flags
    }

    fn deserialize(data: u8) -> Self {
        let is_addr_ipv6 = if (data & 0x01) == 1 { true } else { false };

        NodeFlags { is_addr_ipv6 }
    }
}

#[derive(Debug, Default, PartialOrd, PartialEq, Clone, Serialize)]
pub struct NodeCoordinates {
    pub x1: f32,
    pub x2: f32,
    pub height: f32,
    pub pos_err: f32,
    pub iteration: u64,
}

impl NodeCoordinates {
    pub fn empty() -> Self {
        NodeCoordinates {
            pos_err: 1.0, // initial error
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Serialize)]
pub struct NodeInfo {
    pub flags: NodeFlags,
    pub ip: IpAddr,
    pub port: u16,
    pub name: String,
    pub location: NodeCoordinates,
    // todo: pub location: Option<NodeCoordinates>
}

impl NodeInfo {
    /// Create node info record with empty coordinates
    pub fn new(ip: IpAddr, port: u16, name: String) -> Self {
        // set initial flags
        let is_addr_ipv6 = match ip {
            IpAddr::V4(..) => false,
            IpAddr::V6(..) => true,
        };

        let flags = NodeFlags { is_addr_ipv6 };

        NodeInfo {
            flags,
            ip,
            port,
            name,
            location: NodeCoordinates::empty(),
        }
    }

    /// Get node coordinates
    pub fn get_coordinates(&self) -> &NodeCoordinates {
        &self.location
    }

    /// Set coordinates on existing node record
    pub fn set_coordinates(&mut self, coordinates: &NodeCoordinates) {
        self.location = NodeCoordinates {
            x1: coordinates.x1,
            x2: coordinates.x2,
            height: coordinates.height,
            pos_err: coordinates.pos_err,
            iteration: coordinates.iteration,
        }
    }
}

/// NodeInfo structure protocol layout
///
/// +---------------------------------------+------------------------+------+-------------+------------------------+
/// |                 Flags                 |       IP-address       | Port |  Node name  |       Coordinates      |
/// |---------------------------------------+                        |      +-----+-------+----+----+----+----+----+
/// | x | x | x | x | x | x | x | addr_type |                        |  u16 | len | bytes |  X |  Y |  H |  E |  I |
/// +---------------------------+-----------+------------------------+------+-----+-------+----+----+----+----+----+
/// | 1 | 1 | 1 | 1 | 1 | 1 | 1 |     1     |                        |      |  1  |  var  |  f |  f |  f |  f |  u |
/// +---------------------------------------+                        |      +-----+-------+----+----+----+----+----+
/// |                  8                    | 32 (IPv4) / 128 (IPv6) |  16  |     var     | 32 | 32 | 32 | 32 | 64 |
/// +---------------------------------------+------------------------+------+-------------+----+----+----+----+----+
///
/// Byte order is big-endian.
///
/// Informational flags used
/// +------------+---------------------------+---------------------+
/// |    Flag    |        Description        |        Values       |
/// +------------+---------------------------+---------------------+
/// |  addr_type |      IP-address type      | 0 - IPv4 / 1 - IPv6 |
/// +------------+---------------------------+---------------------+
///
impl NodeInfo {
    pub fn serialize(&self) -> Vec<u8> {
        // allocate maximum
        let mut msg_buff = Vec::with_capacity(19);
        let mut buff_2b: [u8; 2] = [0; 2];
        let mut buff_4b: [u8; 4] = [0; 4];
        let mut buff_8b: [u8; 8] = [0; 8];

        match self.ip {
            IpAddr::V4(addr) => {
                // set type as IPv4
                msg_buff.push(0);
                // set ip address
                msg_buff.extend(addr.octets().iter());
            }

            IpAddr::V6(addr) => {
                // set type as IPv4
                msg_buff.push(1);
                // set ip address
                msg_buff.extend(addr.octets().iter());
            }
        }

        // set port
        BigEndian::write_u16(&mut buff_2b, self.port);
        msg_buff.extend(buff_2b.iter());

        // if name is too long, it wouldn't be used
        let name = serialize_str(&self.name).map_or(vec![0], |b| b);
        msg_buff.extend(name);

        // set coordinates and error
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

        // position iteration
        BigEndian::write_u64(&mut buff_8b, self.location.iteration);
        msg_buff.extend(buff_8b.iter());

        msg_buff
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, &[u8])> {
        // flags + IPv4 + port + empty name + coordinates
        if data.len() < 12 {
            return None;
        }

        let flags = NodeFlags::deserialize(data[0]);
        let mut unparsed = &data[1..];

        let addr = if flags.is_addr_ipv6 {
            if unparsed.len() < 18 {
                return None;
            }

            let octets: &[u8; 16] = &[
                unparsed[0],
                unparsed[1],
                unparsed[2],
                unparsed[3],
                unparsed[4],
                unparsed[5],
                unparsed[6],
                unparsed[7],
                unparsed[8],
                unparsed[9],
                unparsed[10],
                unparsed[11],
                unparsed[12],
                unparsed[13],
                unparsed[14],
                unparsed[15],
            ];
            unparsed = &unparsed[16..];
            IpAddr::from(Ipv6Addr::from(*octets))
        } else {
            let octets: &[u8; 4] = &[unparsed[0], unparsed[1], unparsed[2], unparsed[3]];
            unparsed = &unparsed[4..];
            IpAddr::from(Ipv4Addr::from(*octets))
        };

        // it is safe because we ensure there are at least 2 unparsed bytes for port
        let port = BigEndian::read_u16(&unparsed[..2]);
        unparsed = &unparsed[2..];

        // obtain node's name
        let (name, unparsed) = deserialize_str(unparsed)?;
        let mut node_info = NodeInfo::new(addr, port, name.to_string());

        // bytes required to decode 4 x f32 + 1 x u64 values
        if unparsed.len() < 24 {
            return None;
        }

        // parse coordinates and error
        node_info.set_coordinates(&NodeCoordinates {
            x1: BigEndian::read_f32(&unparsed[..4]),
            x2: BigEndian::read_f32(&unparsed[4..8]),
            height: BigEndian::read_f32(&unparsed[8..12]),
            pos_err: BigEndian::read_f32(&unparsed[12..16]),
            iteration: BigEndian::read_u64(&unparsed[16..24]),
        });

        Some((node_info, &unparsed[24..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_info_serialization_zero_coordinates() {
        let addr = IpAddr::from(Ipv4Addr::new(1, 2, 3, 4));
        let info = NodeInfo::new(addr, 1028, "test".to_string());

        let encoded = info.serialize();

        if let Some((decoded, rest)) = NodeInfo::deserialize(&encoded) {
            assert_eq!(decoded, info);
        } else {
            panic!("deserialization failed");
        }
    }

    #[test]
    fn node_info_serialization_filled_coordinates() {
        let addr = IpAddr::from(Ipv4Addr::new(1, 2, 3, 4));
        let mut info = NodeInfo::new(addr, 1028, "test".to_string());
        info.set_coordinates(&NodeCoordinates {
            x1: 1.0,
            x2: 2.0,
            height: 3.0,
            pos_err: 0.5,
            iteration: 12,
        });

        let encoded = info.serialize();

        if let Some((decoded, rest)) = NodeInfo::deserialize(&encoded) {
            assert_eq!(decoded, info);
        } else {
            panic!("deserialization failed");
        }
    }
}
