/// Primitive and composite types and structures
/// used in protocol messages.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use super::byteorder::{BigEndian, ByteOrder};
use super::*;

pub enum MsgType {
    BootstrapReq,
    BootstrapResp,
}

impl MsgType {
    pub fn to_code(&self) -> u8 {
        match *self {
            MsgType::BootstrapReq => 1,
            MsgType::BootstrapResp => 2,
        }
    }

    pub fn from_code(code: u8) -> Option<MsgType> {
        match code {
            1 => Some(MsgType::BootstrapReq),
            2 => Some(MsgType::BootstrapResp),

            _ => None,
        }
    }
}

/* Node information */

pub struct NodeCoordinates {
    pub x1: f64,
    pub x2: f64,
    pub height: f64,
    pub pos_err: f64,
}

pub struct NodeInfo {
    ip: IpAddr,
    port: u16,
    name: String,
    location: NodeCoordinates,
}

impl NodeInfo {
    /// Create node info record with empty coordinates
    pub fn new(ip: IpAddr, port: u16, name: String) -> Self {
        NodeInfo {
            ip,
            port,
            name,
            location: NodeCoordinates {
                x1: 0.0,
                x2: 0.0,
                height: 0.0,
                pos_err: 0.0,
            },
        }
    }

    /// Get node coordinates
    pub fn get_coordinates(&self) -> &NodeCoordinates {
        &self.location
    }

    /// Set coordinates on existing node record
    pub fn set_coordinates(&mut self, coordinates: NodeCoordinates) {
        self.location = coordinates;
    }
}

/// NodeInfo structure protocol layout
///
/// +---------------------------------------+--------------------------------+--------------+-------------+-------------------+
/// |                 Flags                 |            IP-address          |     Port     |  Node name  |     Coordinates   |
/// |---------------------------------------+                                |              +-----+-------+----+----+----+----+
/// | x | x | x | x | x | x | x | addr_type |           (big-endian)         | (big-endian) | len | bytes |  X |  Y |  H |  E |
/// +---------------------------+-----------+--------------------------------+--------------+-----+-------+----+----+----+----+
/// | 1 | 1 | 1 | 1 | 1 | 1 | 1 |     1     |                                |              |  1  |  var  |  f |  f |  f |  f |
/// +---------------------------------------+                                |              +-----+-------+----+----+----+----+
/// |                  8                    |      32 (IPv4) / 128 (IPv6)    |      16      |      var    | 64 | 64 | 64 | 64 |
/// +---------------------------------------+--------------------------------+--------------+-------------+----+----+----+----+
///
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
        let mut buff = Vec::with_capacity(19);
        match self.ip {
            IpAddr::V4(addr) => {
                // set type as IPv4
                buff.push(0);
                // set ip address
                buff.extend(addr.octets().iter());
            }

            IpAddr::V6(addr) => {
                // set type as IPv4
                buff.push(1);
                // set ip address
                buff.extend(addr.octets().iter());
            }
        }

        // set port
        let port_high = (self.port & 0xff00 >> 8) as u8;
        let port_low = (self.port & 0x00ff) as u8;
        buff.push(port_high);
        buff.push(port_low);

        // if name is too long, it wouldn't be used
        let name = serialize_str(&self.name).map_or(vec![0], |b| b);
        buff.extend(name);

        // todo set coordinates

        buff
    }

    pub fn deserialize(data: &[u8]) -> Option<(NodeInfo, &[u8])> {
        let mut unparsed = &data[1..];

        let addr = match *data.get(0)? {
            // read IPv4
            0 => {
                if unparsed.len() < 6 {
                    return None;
                }

                let octets: &[u8; 4] = &[unparsed[0], unparsed[1], unparsed[2], unparsed[3]];
                unparsed = &unparsed[4..];
                Some(IpAddr::from(Ipv4Addr::from(*octets)))
            }

            // read IPv6
            1 => {
                if unparsed.len() < 18 {
                    return None;
                }

                //let octets = &unparsed[..16];
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
                Some(IpAddr::from(Ipv6Addr::from(*octets)))
            }

            _ => None,
        }?;

        // it is safe because we ensure there are at least 2 unparsed bytes
        let port = (*unparsed.get(0).unwrap() as u16) << 8 + (*unparsed.get(1).unwrap() as u16);
        unparsed = &unparsed[2..];

        // obtain node's name
        let (name, unparsed) = deserialize_str(unparsed)?;
        let mut node_info = NodeInfo::new(addr, port, name.to_string());

        // bytes required to decode 4 x f64 values
        if unparsed.len() < 4 * 8 {
            return None;
        }

        let x1 = BigEndian::read_f64(&unparsed[..8]);
        let x2 = BigEndian::read_f64(&unparsed[8..16]);
        let height = BigEndian::read_f64(&unparsed[16..24]);
        let pos_err = BigEndian::read_f64(&unparsed[24..32]);

        // todo parse coordinates and error
        node_info.set_coordinates(NodeCoordinates { x1, x2, height, pos_err });

        Some((node_info, &unparsed[32..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_info_serialization_zero_coordinates() {
        let addr = IpAddr::from(Ipv4Addr::new(1, 2, 3, 4));
        let expected: Vec<u8> = vec![0, 1, 2, 3, 4, 4, 4, 4, 116, 101, 115, 116, ];
        //                       flag^  ^ip-addr    ^port ^lp ^name              ^x1

        let info = NodeInfo::new(addr, 1028, "test".to_string());
        assert_eq!(info.serialize(), expected);
    }

    // todo more
}
