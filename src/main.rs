use anyhow::Context;
use std::io::prelude::*;
use std::net::TcpListener;

use http_server_starter_rust::{parse_request, HttpMethod};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("sockaddr is correct");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!(
                    "accepted a connection from {}",
                    stream
                        .peer_addr()
                        .context("couldn't get peer address of stream")?
                );

                let mut buf = [0; 1024];
                let read_bytes = stream.read(&mut buf).context("read request")?;
                let request = String::from_utf8_lossy(&buf[..read_bytes]).to_string();
                let (_, (start_line, headers)) =
                    parse_request(request.as_str()).expect("request to be parsable");

                match (start_line.method, start_line.target.as_str()) {
                    (HttpMethod::GET, "/") => {
                        stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                    }
                    (HttpMethod::GET, "/user-agent") => {
                        let user_agent = headers
                            .get("User-Agent")
                            .expect("user agent head to be present");
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent);
                        stream.write(&response.into_bytes())?;
                    }
                    (HttpMethod::GET, echo_path) if echo_path.starts_with("/echo/") => {
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
