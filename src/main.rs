mod http;
use crate::http::handshake::handle_http_upgrade;
use anyhow::{self, bail};
use bytes::BytesMut;
use fastwebsockets::{FragmentCollector, Frame, OpCode, WebSocket};
use nom::AsBytes;
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use wstcpproxy::debug_print;

const NETWORK_BUFFER: usize = 8192;

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
    let mut ws = FragmentCollector::new(ws);

    let tcp_conn = TcpStream::connect("httpbin.org:81").await?;

    println!("Connected!");

    let (reader, mut writer) = tcp_conn.into_split();

    let mut con_buf = [0u8; NETWORK_BUFFER];

    loop {
        tokio::select! {
            ws_frame_data = ws.read_frame() => {
                debug_print!("WS WAS SELCTED");
                let frame = ws_frame_data?;
                match frame.opcode {
                    OpCode::Close => {
                        debug_print!("Close was recvd");
                        break
                    },
                    OpCode::Binary => {
                        let payload = frame.payload.as_bytes();
                        debug_print!(&payload);
                        writer.write(payload).await?;
                        writer.flush().await?;
                    }
                    _ => {}
                }
            },
            _ = reader.readable() => {
                debug_print!("Ready for Reading from socket!");
                match reader.try_read(&mut con_buf) {
                    Ok(0) => bail!("Stream closed amidst reading"),
                    Ok(n) => {
                        let new = &con_buf[..n];
                        let ws_frame = Frame::new(true, OpCode::Binary, None, fastwebsockets::Payload::Borrowed(new));
                        ws.write_frame(ws_frame).await?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => bail!(e),
                };

            }
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
