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
                let path = start_line.split_whitespace().collect::<Vec<&str>>();

                match (path[0], path[1]) {
                    ("GET", "/") => {
                        stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                    }
                    ("GET", echo_path) if echo_path.starts_with("/echo/") => {
                        let echo_str = echo_path
                            .strip_prefix("/echo/")
                            .expect("should have /echo/ prefix");

                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo_str.len(), echo_str);
                        stream.write(&response.into_bytes())?;
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
