use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
};

use anyhow::Result;
use clap::Parser;
use codecrafters_http_server::{
    bytes::ToBytes, handle_request, parser::StreamParser, spec::request::Request, Cli,
};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let cli = Cli::parse();

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let cli = cli.clone();
        thread::spawn(move || {
            handle_stream(cli, stream).unwrap();
        });
    }

    Ok(())
}

fn handle_stream(cli: Cli, mut stream: TcpStream) -> Result<()> {
    let mut parser = StreamParser::new(&stream);
    match parser.parse::<Request>() {
        Ok(request) => {
            let response = handle_request(cli, request)?;
            let bytes = response.into_bytes();
            eprintln!("{}", String::from_utf8(bytes.clone()).unwrap());
            stream.write_all(&bytes)?;
        }
        Err(e) => println!("{}", e),
    }
    Ok(())
}
