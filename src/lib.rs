pub mod bytes;
pub mod parser;
mod routes;
pub mod spec;

use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Result;
use parser::Parse;
use routes::Route;
use spec::{
    message::{FieldContent, MessageBody, MessageHeader},
    request::{Method, Request},
    response::{Response, Status, StatusLine},
};

#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    #[arg(long)]
    directory: Option<PathBuf>,
}

pub fn handle_request(cli: Cli, request: Request) -> Result<Response> {
    let route = Route::from(&request.request_line.request_uri);

    let (status, headers, body) = match request.request_line.method {
        Method::Get => match route {
            Route::Root => (Status::OK, vec![], None),
            Route::Echo { command } if !command.is_empty() => {
                let mut headers = vec![
                    MessageHeader::convert("Content-Type: text/plain")?,
                    MessageHeader::convert(&format!("Content-Length: {}", command.len()))?,
                ];
                if let Some(FieldContent(accept_encoding)) =
                    request.first_value_content(b"Accept-Encoding")
                {
                    if accept_encoding == b"gzip" {
                        headers.push(MessageHeader::convert("Content-Encoding: gzip")?)
                    }
                }
                (Status::OK, headers, Some(MessageBody(command)))
            }
            Route::UserAgent => {
                let user_agent = request.first_value_content(b"User-Agent");
                match user_agent {
                    Some(user_agent) => {
                        let headers = vec![
                            MessageHeader::convert("Content-Type: text/plain")?,
                            MessageHeader::convert(&format!(
                                "Content-Length: {}",
                                user_agent.0.len()
                            ))?,
                        ];
                        (Status::OK, headers, Some(MessageBody(user_agent.0)))
                    }
                    None => (Status::NotFound, vec![], None),
                }
            }
            Route::Files { filename } => {
                let directory = cli.directory.expect("directory must be passed");

                if let Ok(Ok(mut file)) = String::from_utf8(filename)
                    .map(|filename| directory.join(filename))
                    .map(File::open)
                {
                    let mut content = vec![];
                    file.read_to_end(&mut content)?;

                    let headers = vec![
                        MessageHeader::convert("Content-Type: application/octet-stream")?,
                        MessageHeader::convert(&format!("Content-Length: {}", content.len()))?,
                    ];
                    (Status::OK, headers, Some(MessageBody(content)))
                } else {
                    (Status::NotFound, vec![], None)
                }
            }
            _ => (Status::NotFound, vec![], None),
        },
        Method::Post => match route {
            Route::Files { filename } => {
                let directory = cli.directory.expect("directory must be passed");
                if let Ok(filename) = String::from_utf8(filename) {
                    let file = directory.join(filename);
                    let mut file = File::options()
                        .create_new(true)
                        .truncate(true)
                        .write(true)
                        .open(file)
                        .expect("cannot open / create file");
                    match &request.body {
                        Some(body) => file.write_all(&body.0).expect("cannot write file"),
                        None => file.write_all(b"").expect("cannot write file"),
                    }
                    (Status::Created, vec![], None)
                } else {
                    (Status::NotFound, vec![], None)
                }
            }
            _ => (Status::NotFound, vec![], None),
        },
        Method::Extension(_) => (Status::NotFound, vec![], None),
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
