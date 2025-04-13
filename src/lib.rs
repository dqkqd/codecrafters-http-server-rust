pub mod bytes;
pub mod parser;
mod request;
mod spec;

use std::path::PathBuf;

use request::Handler;
pub use spec::request::Request;

#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    #[arg(long)]
    directory: Option<PathBuf>,
}

pub enum ServerResponse {
    Continue(Vec<u8>),
    Close(Vec<u8>),
}

impl ServerResponse {
    pub fn data(&self) -> &[u8] {
        match self {
            ServerResponse::Continue(data) => data,
            ServerResponse::Close(data) => data,
        }
    }
}

pub fn handle_request(cli: Cli, request: Request) -> ServerResponse {
    let handler = Handler::new(request, cli.directory);
    handler.process()
}
