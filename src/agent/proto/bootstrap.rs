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
    // todo ?
}

impl<'a> BootstrapRequest<'a> {
    pub fn new(local_name: &'a str) -> Self {
        BootstrapRequest { local_name }
    }
}

impl<'a> BinarySerializable for BootstrapRequest<'a> {
    fn serialize(&self) -> Option<Vec<u8>> {
        let mut name_serialized = serialize_str(self.local_name)?;

        // insert msg code to create request payload
        name_serialized.insert(0, types::MsgType::BootstrapReq.to_code());

        Some(name_serialized)
    }
}

pub struct BootstrapResponse {
    neighbours: storage::NodeList,
}

impl BootstrapResponse {
    fn empty() -> Self {
        BootstrapResponse {
            neighbours: storage::NodeList(Vec::new()),
        }
    }
}

impl BinaryDeserializable for BootstrapResponse {
    type Item = Self;

    fn deserialize(&self, data: &[u8]) -> Option<Self> {
        //        let mut cursor = 0;
        //        let data_len = data.len();

        let msg = BootstrapResponse::empty();

        let mut unparsed = &data[..];

        while unparsed.len() > 0 {
            //let next_str_size = data.get(cursor)?;

            let (name, rest) = deserialize_str(unparsed)?;

            // todo follow up
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_bootstrap_request() {
        let req = BootstrapRequest::new("test");
        assert_eq!(req.serialize(), Some(vec![1, 4, 116, 101, 115, 116]));

        let empty = BootstrapRequest::new("");
        assert_eq!(empty.serialize(), Some(vec![1, 0]));
    }

    #[test]
    fn deserialize_bootstrap_response() {

        // todo

    }
}
