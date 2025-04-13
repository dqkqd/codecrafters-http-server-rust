use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
};

use anyhow::Result;
use clap::Parser;
use codecrafters_http_server::{
    handle_request, parser::StreamParser, Cli, Request, ServerResponse,
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
    loop {
        let mut parser = StreamParser::new(&stream);
        match parser.parse::<Request>() {
            Ok(request) => {
                let resp = handle_request(cli.clone(), request);
                stream.write_all(resp.data())?;
                if let ServerResponse::Close(_) = resp {
                    break Ok(());
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}
