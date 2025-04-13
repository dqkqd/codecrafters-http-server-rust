use std::{io::Write, path::PathBuf};

use crate::spec::{
    message::{FieldContent, FieldName, FieldValue, MessageBody, MessageHeader},
    request::{Method, Request as RawRequest},
    response::{Response, Status, StatusLine},
};

mod routes;

use flate2::{write::GzEncoder, Compression};
pub(crate) use routes::Route;

pub(super) type AdditionalHeader = Vec<(String, String)>;
pub(super) type AdditionalBody = Vec<u8>;

#[derive(Debug, PartialEq, Eq)]
enum Encoding {
    Gzip,
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Request {
    inner: RawRequest,
    cli_directory: Option<PathBuf>,
    encoding: Vec<Encoding>,
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
                encoding: vec![],
            },
        }
    }

    pub fn process(mut self) -> Response {
        let route = Route::from(&self.request.inner.request_line.request_uri);

        if let Some(accept_encoding) = self.request.inner.find_value(b"Accept-Encoding") {
            self.request.encoding = accept_encoding
                .split(|u| u == &b',')
                .map(|v| match v.trim_ascii() {
                    b"gzip" => Encoding::Gzip,
                    _ => Encoding::Invalid,
                })
                .collect();
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

        if self.request.encoding.contains(&Encoding::Gzip) {
            self.add_response_header("Content-Encoding", "gzip");
            if let Some(MessageBody(data)) = self.response.body.as_ref() {
                let mut e = GzEncoder::new(Vec::new(), Compression::default());
                e.write_all(data).expect("failed to encode using gzip");
                let body = e.finish().expect("failed to encode gzip");
                self.response.body = Some(MessageBody(body));
            }
        }

        if let Some(body) = self.response.body.as_ref() {
            self.add_response_header("Content-Length", &body.0.len().to_string());
        }

        self.response
    }

    fn add_response_header(&mut self, header: &str, content: &str) {
        let header = MessageHeader {
            field_name: FieldName(header.as_bytes().into()),
            field_value: Some(FieldValue(vec![FieldContent(content.as_bytes().into())])),
        };
        // replace the old one
        match self
            .response
            .headers
            .iter()
            .position(|h| h.field_name == header.field_name)
        {
            Some(existing_idx) => self.response.headers[existing_idx] = header,
            None => self.response.headers.push(header),
        }
    }
}
