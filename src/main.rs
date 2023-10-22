use anyhow::Context;
use itertools::Itertools;
use std::env;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use http_server_starter_rust::{parse_request, HttpMethod};

async fn handle_stream(mut stream: TcpStream, dir: Option<String>) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    let read_bytes = stream.read(&mut buf).await.context("read request")?;
    let request = String::from_utf8_lossy(&buf[..read_bytes]).to_string();
    let (_, (start_line, headers, body)) =
        parse_request(request.as_str()).expect("request to be parsable");

    match (start_line.method, start_line.target.as_str()) {
        (HttpMethod::GET, "/") => {
            stream.write(b"HTTP/1.1 200 OK\r\n\r\n").await?;
        }
        (HttpMethod::GET, "/user-agent") => {
            let user_agent = headers
                .get("User-Agent")
                .expect("user agent head to be present");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                user_agent
            );
            stream.write(&response.into_bytes()).await?;
        }
        (HttpMethod::GET, echo_path) if echo_path.starts_with("/echo/") => {
            let echo_str = echo_path
                .strip_prefix("/echo/")
                .expect("should have /echo/ prefix");

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                echo_str.len(),
                echo_str
            );
            stream.write(&response.into_bytes()).await?;
        }
        (HttpMethod::GET, file_name) if file_name.starts_with("/files/") && dir.is_some() => {
            let file_name = file_name
                .strip_prefix("/files/")
                .expect("should have /files/ prefix");
            let dir = dir.expect("shouldn't go into this branch");
            let file_path = PathBuf::from_iter([dir.as_str(), file_name].iter());
            match File::open(file_path).await {
                Ok(mut file) => {
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).await?;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        buf.len(),
                        buf
                    );
                    stream.write(&response.into_bytes()).await?;
                }
                Err(_) => {
                    stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
                }
            }
        }
        (HttpMethod::POST, file_name) if file_name.starts_with("/files/") && dir.is_some() => {
            let file_name = file_name
                .strip_prefix("/files/")
                .expect("should have /files/ prefix");
            let dir = dir.expect("shouldn't go into this branch");
            let file_path = PathBuf::from_iter([dir.as_str(), file_name].iter());
            let mut file = File::create(file_path).await?;
            file.write_all(body.as_bytes()).await?;
            stream.write(b"HTTP/1.1 201 Created\r\n\r\n").await?;
        }
        _ => {
            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = env::args().collect_vec();
    let dir = if args.len() > 2 {
        assert_eq!(args[1].as_str(), "--directory");
        Some(args[2].clone())
    } else {
        None
    };
    let listener = TcpListener::bind("127.0.0.1:4221")
        .await
        .expect("sockaddr is correct");

    loop {
        let (stream, _) = listener.accept().await?;
        let dir_loop = dir.clone();
        tokio::spawn(async move {
            println!(
                "accepted a connection from {}",
                stream
                    .peer_addr()
                    .context("couldn't get peer address of stream")?
            );
            handle_stream(stream, dir_loop).await
        });
    }
}
