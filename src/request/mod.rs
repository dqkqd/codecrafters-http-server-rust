use std::path::PathBuf;

use crate::spec::{
    message::{FieldContent, FieldName, FieldValue, MessageBody, MessageHeader},
    request::{Method, Request as RawRequest},
    response::{Response, Status, StatusLine},
};

mod routes;

pub(crate) use routes::Route;

pub(super) type AdditionalHeader = Vec<(String, String)>;
pub(super) type AdditionalBody = Vec<u8>;

pub(super) struct Request {
    inner: RawRequest,
    cli_directory: Option<PathBuf>,
}

pub(super) trait HandleRequest {
    fn handle(&self, request: &Request) -> (Option<Status>, AdditionalHeader, AdditionalBody);
}

pub(crate) struct Handler {
    request: Request,
    response: Response,
}

impl Request {
    pub fn method(&self) -> &Method {
        &self.inner.request_line.method
    }
}

impl Handler {
    pub fn new(request: RawRequest, cli_directory: Option<PathBuf>) -> Handler {
        Handler {
            response: Response {
                status_line: StatusLine {
                    http_version: request.request_line.http_version,
                    status: Status::NotFound,
                },
                headers: vec![],
                body: None,
            },
            request: Request {
                inner: request,
                cli_directory,
            },
        }
    }

    pub fn process(mut self) -> Response {
        let route = Route::from(&self.request.inner.request_line.request_uri);

        if let Some(FieldContent(accept_encoding)) =
            self.request.inner.first_value_content(b"Accept-Encoding")
        {
            if accept_encoding == b"gzip" {
                self.add_response_header("Content-Encoding", "gzip");
            }
        }

        let (status, headers, body) = route.handle(&self.request);
        if let Some(status) = status {
            self.response.status_line.status = status;
        }
        for (header, content) in headers {
            self.add_response_header(&header, &content);
        }
        if !body.is_empty() {
            match self.response.body.as_mut() {
                Some(response_body) => response_body.0.extend_from_slice(&body),
                None => self.response.body = Some(MessageBody(body)),
            }
        }

        self.response
    }

    fn add_response_header(&mut self, header: &str, content: &str) {
        let header = MessageHeader {
            field_name: FieldName(header.as_bytes().into()),
            field_value: Some(FieldValue(vec![FieldContent(content.as_bytes().into())])),
        };
        self.response.headers.push(header);
    }
}
