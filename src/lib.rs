pub mod bytes;
pub mod parser;
mod request;
mod spec;

use std::path::PathBuf;

use request::Handler;
pub use spec::{request::Request, response::Response};

#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    #[arg(long)]
    directory: Option<PathBuf>,
}

pub fn handle_request(cli: Cli, request: Request) -> Response {
    let handler = Handler::new(request, cli.directory);
    handler.process()
}
