use crate::bytes::ToBytes;

use super::{
    message::{FieldName, MessageBody, MessageHeader},
    protocol::HttpVersion,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Method {
    Get,
    Post,
    Extension(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RequestURI(pub Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RequestLine {
    pub method: Method,
    pub request_uri: RequestURI,
    pub http_version: HttpVersion,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    pub(crate) request_line: RequestLine,
    pub(crate) headers: Vec<MessageHeader>,
    pub(crate) body: Option<MessageBody>,
}

impl Request {
    pub(crate) fn find_header(&self, header: &[u8]) -> Option<&MessageHeader> {
        self.headers
            .iter()
            .rev()
            .find(|h| h.field_name == FieldName(header.into()))
    }

    pub(crate) fn find_value(&self, header: &[u8]) -> Option<Vec<u8>> {
        self.find_header(header)
            .and_then(|header| header.field_value.as_ref())
            .map(|field_value| field_value.clone().into_bytes())
    }
}
