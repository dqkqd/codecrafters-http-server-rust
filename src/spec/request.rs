use super::{
    message::{MessageBody, MessageHeader},
    protocol::HttpVersion,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Method {
    Get,
    ExtensionMethod(Vec<u8>),
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
        self.headers.iter().find(|h| h.field_name.0 == header)
    }
}
