mod http;
use crate::http::handshake::handle_http_upgrade;
use anyhow;
use bytes::BytesMut;
use fastwebsockets::{OpCode, WebSocket};
use nom::AsBytes;
use tokio::net::{TcpListener, TcpStream};

const NETWORK_BUFFER: usize = 2048;

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buffer = BytesMut::with_capacity(NETWORK_BUFFER);
    handle_http_upgrade(&mut stream, &mut buffer).await?;
    handle_as_websocket(stream).await?;
    Ok(())
}

async fn handle_as_websocket(stream: TcpStream) -> anyhow::Result<()> {
    let mut ws = WebSocket::after_handshake(stream, fastwebsockets::Role::Server);
    ws.set_writev(true);
    ws.set_auto_close(true);
    ws.set_auto_pong(true);

    loop {
        let frame = ws.read_frame().await?;
        match frame.opcode {
            OpCode::Close => break,
            OpCode::Binary => {
                print!("{}", std::str::from_utf8(frame.payload.as_bytes()).unwrap())
            }
            _ => {}
        }
    }

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
