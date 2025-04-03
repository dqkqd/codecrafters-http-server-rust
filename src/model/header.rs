use std::io::BufRead;

use crate::model::read_until_crlf;

use super::{Converter, CRLF};
use anyhow::Result;
use bytes::Bytes;
use indexmap::IndexMap;

#[derive(Debug, Hash, PartialEq, Eq)]
struct FieldName(Bytes);
#[derive(Debug, Hash, PartialEq, Eq)]
struct FieldValue(Bytes);

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct MessageHeader {
    data: IndexMap<FieldName, FieldValue>,
}

impl MessageHeader {
    pub fn new() -> MessageHeader {
        MessageHeader::default()
    }
}

impl Converter for MessageHeader {
    fn from_reader<R: BufRead>(reader: &mut R) -> Result<Self> {
        let mut header = MessageHeader::new();
        while let Some(buf) = read_until_crlf(reader)? {
            let mut split = buf
                .split(|v| v == &b':')
                .map(|v| v.trim_ascii())
                .filter(|v| !v.is_empty());

            let key = split.next().unwrap_or_default();
            let value = split.next().unwrap_or_default();
            if key.is_empty() {
                continue;
            }
            let key = FieldName(Bytes::copy_from_slice(key));
            let value = FieldValue(Bytes::copy_from_slice(value));
            header.data.insert(key, value);
        }

        Ok(header)
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        for (k, v) in &self.data {
            writer.write_all(&k.0)?;
            writer.write_all(b": ")?;
            writer.write_all(&v.0)?;
            writer.write_all(CRLF.as_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_from_bytes() {
        assert_eq!(
            MessageHeader::from_u8(b"Content-Type: text/plain\r\nContent-Length: 3\r\n").unwrap(),
            MessageHeader {
                data: IndexMap::from([
                    (
                        FieldName(Bytes::copy_from_slice(b"Content-Type")),
                        FieldValue(Bytes::copy_from_slice(b"text/plain"))
                    ),
                    (
                        FieldName(Bytes::copy_from_slice(b"Content-Length")),
                        FieldValue(Bytes::copy_from_slice(b"3"))
                    ),
                ]),
            }
        );
    }

    #[test]
    fn test_header_empty_from_bytes() {
        assert_eq!(MessageHeader::from_u8(b"").unwrap(), MessageHeader::new());
    }

    #[test]
    fn test_header_to_bytes() {
        let bytes = b"Content-Type: text/plain\r\nContent-Length: 3\r\n";
        let mut out = vec![];
        let header = MessageHeader::from_u8(&bytes.clone()).unwrap();
        header.to_writer(&mut out).unwrap();
        assert_eq!(bytes.to_vec(), out);
    }

    #[test]
    fn test_header_empty_to_bytes() {
        let header = MessageHeader::new();
        let mut out = vec![];
        header.to_writer(&mut out).unwrap();
        assert_eq!(b"".to_vec(), out);
    }
}
