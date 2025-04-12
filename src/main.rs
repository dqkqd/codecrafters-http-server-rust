use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    time::Duration,
};

use anyhow::Result;
use codecrafters_http_server::{
    bytes::ToBytes,
    handle_request,
    parser::StreamParser,
    spec::request::Request,
};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_millis(100)))?;
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
    let mut parser = StreamParser::new(&stream);
    match parser.parse::<Request>() {
        Ok(request) => {
            let response = handle_request(request)?;
            let bytes = response.into_bytes();
            eprintln!("{}", String::from_utf8(bytes.clone()).unwrap());
            stream.write_all(&bytes)?;
        }
        Err(_) => todo!(),
    }
    Ok(())
}
