use itertools::Itertools;

use crate::spec::message::{FieldContent, FieldName, FieldValue, MessageBody, MessageHeader};

use super::ToBytes;

impl ToBytes for FieldName {
    fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl ToBytes for FieldContent {
    fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl ToBytes for FieldValue {
    #[allow(unstable_name_collisions)]
    fn into_bytes(self) -> Vec<u8> {
        self.0
            .into_iter()
            .map(ToBytes::into_bytes)
            .intersperse_with(|| b" ".into())
            .concat()
    }
}

impl ToBytes for MessageHeader {
    fn into_bytes(self) -> Vec<u8> {
        let (colon, field_value) = match self.field_value {
            Some(field_value) => (b": ".into(), field_value.into_bytes()),
            None => (b":".into(), "".into()),
        };
        [self.field_name.into_bytes(), colon, field_value].concat()
    }
}

impl ToBytes for MessageBody {
    fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn field_name() {
        assert_eq!(FieldName(b"name".to_vec()).into_bytes(), b"name")
    }

    #[test]
    fn field_content() {
        assert_eq!(FieldContent(b"content".to_vec()).into_bytes(), b"content")
    }

    #[test]
    fn field_value() {
        assert_eq!(
            FieldValue(vec![
                FieldContent(b"content1".to_vec()),
                FieldContent(b"content2".to_vec()),
            ])
            .into_bytes(),
            b"content1 content2"
        )
    }

    #[test]
    fn message_header() {
        assert_eq!(
            MessageHeader {
                field_name: FieldName(b"name".to_vec()),
                field_value: Some(FieldValue(vec![
                    FieldContent(b"content1".to_vec()),
                    FieldContent(b"content2".to_vec()),
                ]))
            }
            .into_bytes(),
            b"name: content1 content2"
        )
    }

    #[test]
    fn message_header_empty_value() {
        assert_eq!(
            MessageHeader {
                field_name: FieldName(b"name".to_vec()),
                field_value: None,
            }
            .into_bytes(),
            b"name:"
        )
    }

    #[test]
    fn message_body() {
        assert_eq!(MessageBody(b"body".to_vec()).into_bytes(), b"body")
    }
}
