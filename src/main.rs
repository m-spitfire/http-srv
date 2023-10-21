use std::io::prelude::*;
use std::net::TcpListener;

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").expect("sockaddr is correct");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("connection received");
                stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
