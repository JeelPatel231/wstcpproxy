mod http;
use crate::http::handshake::handle_http_upgrade;
use anyhow;
use bytes::BytesMut;
use tokio::net::{TcpListener, TcpStream};

const NETWORK_BUFFER: usize = 2048;

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buffer = BytesMut::with_capacity(NETWORK_BUFFER);
    handle_http_upgrade(&mut stream, &mut buffer).await?;
    // TODO: hand off to some websocket frame handling library after handshake is performed
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    println!("Listening on 8000");

    loop {
        let (stream, _) = listener.accept().await?;
        let _ = tokio::spawn(handle_connection(stream));
    }
}
