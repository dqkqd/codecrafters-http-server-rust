
use super::{
    message::{MessageBody, MessageHeader},
    protocol::HttpVersion,
};

#[derive(Debug)]
pub(crate) enum Status {
    OK,
    NotFound,
}

#[derive(Debug)]
pub(crate) struct StatusLine {
    pub http_version: HttpVersion,
    pub status: Status,
}

#[derive(Debug)]
pub struct Response {
    pub(crate) status_line: StatusLine,
    pub(crate) headers: Vec<MessageHeader>,
    pub(crate) body: Option<MessageBody>,
}

impl Status {
    pub fn code(&self) -> u16 {
        match self {
            Status::OK => 200,
            Status::NotFound => 404,
        }
    }
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            Status::OK => "OK",
            Status::NotFound => "Not Found",
        }
    }
}
