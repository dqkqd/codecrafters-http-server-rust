use itertools::Itertools;

use crate::spec::response::{Response, Status, StatusLine};

use super::ToBytes;

impl ToBytes for Status {
    fn into_bytes(self) -> Vec<u8> {
        [
            self.code().to_string().into_bytes(),
            b" ".into(),
            self.reason_phrase().to_string().into_bytes(),
        ]
        .concat()
    }
}

impl ToBytes for StatusLine {
    fn into_bytes(self) -> Vec<u8> {
        [
            self.http_version.into_bytes(),
            b" ".into(),
            self.status.into_bytes(),
            b"\r\n".into(),
        ]
        .concat()
    }
}

impl ToBytes for Response {
    fn into_bytes(self) -> Vec<u8> {
        [
            self.status_line.into_bytes(),
            self.headers
                .into_iter()
                .map(|header| [header.into_bytes(), b"\r\n".into()].concat())
                .concat(),
            b"\r\n".into(),
            self.body.map(ToBytes::into_bytes).unwrap_or_default(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {
    use crate::spec::{
        message::{FieldContent, FieldName, FieldValue, MessageBody, MessageHeader},
        protocol::HttpVersion,
    };

    use super::*;

    #[test]
    fn status_200() {
        assert_eq!(Status::OK.into_bytes(), b"200 OK");
    }

    #[test]
    fn status_201() {
        assert_eq!(Status::Created.into_bytes(), b"201 Created");
    }

    #[test]
    fn status_404() {
        assert_eq!(Status::NotFound.into_bytes(), b"404 Not Found");
    }

    #[test]
    fn status_line() {
        assert_eq!(
            StatusLine {
                http_version: HttpVersion { major: 2, minor: 0 },
                status: Status::OK,
            }
            .into_bytes(),
            b"HTTP/2.0 200 OK\r\n"
        );
    }

    #[test]
    fn request() {
        assert_eq!(
            Response {
                status_line: StatusLine {
                    http_version: HttpVersion { major: 2, minor: 0 },
                    status: Status::OK,
                },
                headers: vec![
                    MessageHeader {
                        field_name: FieldName(b"header1".to_vec()),
                        field_value: Some(FieldValue(vec![
                            FieldContent(b"a".to_vec()),
                            FieldContent(b"b".to_vec()),
                        ]))
                    },
                    MessageHeader {
                        field_name: FieldName(b"header2".to_vec()),
                        field_value: Some(FieldValue(vec![
                            FieldContent(b"c".to_vec()),
                            FieldContent(b"d".to_vec()),
                        ]))
                    }
                ],
                body: Some(MessageBody(b"message body".to_vec())),
            }
            .into_bytes(),
            b"\
HTTP/2.0 200 OK\r
header1: a b\r
header2: c d\r
\r
message body"
        );
    }
}
