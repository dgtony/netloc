/// UDP-based communication protocol.
///
/// Exchange location info between nodes.

extern crate byteorder;

mod bootstrap;
mod types;

use std::str::from_utf8;

use storage;

// fixme do we really need a trait?
trait BinarySerializable {
    // fixme mb change to Result<Vec<u8>, ? SomeErr ? >
    fn serialize(&self) -> Option<Vec<u8>>;
}

trait BinaryDeserializable {
    type Item;

    // fixme mb change to Result<Self::Item, ? SomeErr ? >
    fn deserialize(&self, data: &[u8]) -> Option<Self::Item>;
}

/* Strings */

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

    // empty string
    if str_bytes_len < 1 {
        return Some(vec![0]);
    }

    // too long
    if str_bytes_len > 254 {
        return None;
    }

    let mut buff = Vec::with_capacity(str_bytes_len + 1);
    // set length prefix in front of data
    buff.push(str_bytes_len as u8);
    buff.extend(str_bytes);
    Some(buff)
}

/// Consume and deserialize first string in data
fn deserialize_str(data: &[u8]) -> Option<(&str, &[u8])> {
    // empty buffer
    if data.len() < 1 {
        return None;
    }

    let str_len = *data.get(0)? as usize;

    // bad length prefix
    if data.len() < str_len + 1 {
        return None;
    }

    // move one byte
    let data = &data[1..];
    let (str_bytes, rest) = data.split_at(str_len as usize);
    // will break if not valid UTF-8
    let s = from_utf8(str_bytes).ok()?;
    Some((s, rest))
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
        assert_eq!(serialize_str(""), Some(vec![0]));

        // too long
        let long: String = ['x'; 300].iter().collect();
        assert_eq!(serialize_str(long.as_str()), None);
    }

    #[test]
    fn str_deserialization_exact() {
        let data = vec![4, 116, 101, 115, 116];
        if let Some((s, rest)) = deserialize_str(&data) {
            assert_eq!(s, "test");
            assert_eq!(rest, &[]);
        } else {
            panic!("cannot deserialize string");
        }
    }

    #[test]
    fn str_deserialization_redundant() {
        let data = vec![4, 116, 101, 115, 116, 112, 221, 12];
        if let Some((s, rest)) = deserialize_str(&data) {
            assert_eq!(s, "test");
            assert_eq!(rest, &[112, 221, 12]);
        } else {
            panic!("cannot deserialize string");
        }
    }

    #[test]
    fn str_deserialization_empty() {
        let data = vec![];
        assert_eq!(deserialize_str(&data), None);
    }

    #[test]
    fn str_deserialization_no_str() {
        // only length - must return rest without zero length prefix
        let data = &[0, 1, 2, 3];
        if let Some((s, rest)) = deserialize_str(data) {
            assert_eq!(s, "");
            assert_eq!(rest, &[1, 2, 3]);
        } else {
            panic!("cannot deserialize string");
        }
    }

    #[test]
    fn str_deserialization_bad_len_prefix() {
        assert_eq!(deserialize_str(&[12, 23, 32]), None);
    }
}
