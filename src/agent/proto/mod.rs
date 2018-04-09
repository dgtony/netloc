/// UDP-based communication protocol.
///
/// Exchange location info between nodes.

mod bootstrap;

pub enum MsgType {
    BootstrapReq,
    BootstrapResp,
}

impl MsgType {
    fn to_code(&self) -> u8 {
        match *self {
            MsgType::BootstrapReq => 1,
            MsgType::BootstrapResp => 2,
        }
    }

    fn from_code(code: u8) -> Option<MsgType> {
        match code {
            1 => Some(MsgType::BootstrapReq),
            2 => Some(MsgType::BootstrapResp),

            _ => None,
        }
    }
}

// fixme do we really need a trait?
trait BinarySerialized {
    // fixme mb change to Result<Vec<u8>, ? SomeErr ? >
    fn serialize(&self) -> Option<Vec<u8>>;
}

/// Serialize short strings.
///
/// Current protocol scheme serialize str as follows:
/// +---------+---------------+
/// | str_len |   byte array  |
/// +---------+---------------+
/// |    u8   | 1 - 254 bytes |
/// +---------+---------------+
///
/// Hence only short strings (0 < len < 255 bytes) could be serialized.
fn serialize_str(s: &str) -> Option<Vec<u8>> {
    // get byte representation of Unicode str
    let str_bytes = s.as_bytes();
    let str_bytes_len = str_bytes.len();

    if str_bytes_len < 1 || str_bytes_len > 254 {
        return None;
    }

    let mut buff = Vec::with_capacity(str_bytes_len + 1);
    // set length prefix in front of data
    buff.push(str_bytes_len as u8);
    buff.extend(str_bytes);
    Some(buff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_serialization() {
        // normal
        assert_eq!(serialize_str("test"), Some(vec![4, 116, 101, 115, 116]));

        // non-ascii
        assert_eq!(
            serialize_str("узел"),
            Some(vec![8, 209, 131, 208, 183, 208, 181, 208, 187])
        );

        // empty
        assert_eq!(serialize_str(""), None);

        // too long
        let long: String = ['x'; 300].iter().collect();
        assert_eq!(serialize_str(long.as_str()), None);
    }
}
