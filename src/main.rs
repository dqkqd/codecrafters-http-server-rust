use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

use anyhow::Result;
use codecrafters_http_server::{
    response::{response_200, response_404},
    RequestLine, RequestMethod,
};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_stream(stream)?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let request_line = RequestLine::parse(&mut reader)?;
    match (request_line.method, request_line.target.as_str()) {
        (RequestMethod::Get, "/") => response_200(&mut stream),
        _ => response_404(&mut stream),
    }
}
