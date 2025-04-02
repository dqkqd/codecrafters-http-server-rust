use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

use anyhow::Result;
use codecrafters_http_server::{
    request::{RequestLine, RequestMethod},
    response::{HttpResponse, HttpStatus},
    routes::Route,
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

    let response = match (request_line.method, request_line.route) {
        (RequestMethod::Get, Route::Root) => HttpResponse::new(HttpStatus::Ok),
        (RequestMethod::Get, Route::Echo { command }) => {
            HttpResponse::new(HttpStatus::Ok).with_text_response(&command)
        }
        _ => HttpResponse::new(HttpStatus::NotFound),
    };
    response.output(&mut stream)
}
