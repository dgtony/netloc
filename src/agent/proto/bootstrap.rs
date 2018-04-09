/// Bootstrap messages

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
pub struct BootstrapRequest<'a> {
    local_name: &'a str,
    // todo
}

impl<'a> BinarySerialized for BootstrapRequest<'a> {
    fn serialize(&self) -> Option<Vec<u8>> {
        let mut name_serialized = serialize_str(self.local_name)?;

        // insert msg code to create request payload
        name_serialized.insert(0, MsgType::BootstrapReq.to_code());

        Some(name_serialized)
    }
}
