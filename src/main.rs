use std::io::{prelude::*, BufReader};
use std::net::TcpListener;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("sockaddr is correct");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&mut stream);
                let mut start_line = String::new();
                reader.read_line(&mut start_line)?;
                let path = start_line
                    .split_whitespace()
                    .nth(1)
                    .expect("start line have path in it");
                match path {
                    "/" => {
                        stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                    }
                    _ => {
                        stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
