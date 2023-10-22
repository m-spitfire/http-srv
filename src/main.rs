use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use http_server_starter_rust::{parse_request, HttpMethod};

async fn handle_stream(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    let read_bytes = stream.read(&mut buf).await.context("read request")?;
    let request = String::from_utf8_lossy(&buf[..read_bytes]).to_string();
    let (_, (start_line, headers)) =
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
        _ => {
            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")
        .await
        .expect("sockaddr is correct");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            println!(
                "accepted a connection from {}",
                stream
                    .peer_addr()
                    .context("couldn't get peer address of stream")?
            );
            handle_stream(stream).await
        });
    }
}
