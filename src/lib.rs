pub mod bytes;
pub mod parser;
mod routes;
pub mod spec;

use anyhow::Result;
use parser::Parse;
use routes::Route;
use spec::{
    message::{MessageBody, MessageHeader},
    request::Request,
    response::{Response, Status, StatusLine},
};

pub fn handle_request(request: Request) -> Result<Response> {
    let route = Route::from(&request.request_line.request_uri);

    let (status, body) = match route {
        Route::Root => (Status::OK, None),
        Route::Echo { command } if !command.is_empty() => (Status::OK, Some(MessageBody(command))),
        Route::UserAgent => {
            let user_agent = request
                .find_header(b"User-Agent")
                .and_then(|header| header.field_value.as_ref())
                .and_then(|field_value| field_value.0.first())
                .map(|field_content| field_content.0.to_vec());
            match user_agent {
                Some(user_agent) => (Status::OK, Some(MessageBody(user_agent))),
                None => (Status::NotFound, None),
            }
        }
        _ => (Status::NotFound, None),
    };

    let headers = match &body {
        Some(b) => vec![
            MessageHeader::convert("Content-Type: text/plain")?,
            MessageHeader::convert(&format!("Content-Length: {}", b.0.len()))?,
        ],
        None => vec![],
    };

    Ok(Response {
        status_line: StatusLine {
            http_version: request.request_line.http_version,
            status,
        },
        headers,
        body,
    })
}
