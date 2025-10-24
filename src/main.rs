use anyhow::{self, bail};
use fastwebsockets::{FragmentCollector, Frame, OpCode};
use fastwebsockets::{Payload, upgrade};
use http_body_util::Empty;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use zerocopy::IntoBytes;

const NETWORK_BUFFER: usize = 8192;

async fn handle_as_websocket(fut: upgrade::UpgradeFut) -> anyhow::Result<()> {
    let mut ws = fut.await?;
    ws.set_writev(true);
    ws.set_auto_close(true);
    ws.set_auto_pong(true);
    let mut ws = FragmentCollector::new(ws);

    let tcp_conn = TcpStream::connect("0.0.0.0:9000").await?;

    eprintln!("Connected!");

    let (reader, mut writer) = tcp_conn.into_split();

    let mut con_buf = [0u8; NETWORK_BUFFER];

    loop {
        tokio::select! {
            ws_frame_data = ws.read_frame() => {
                let frame = ws_frame_data?;
                match frame.opcode {
                    OpCode::Close => bail!("Client closed connection."),
                    OpCode::Binary => {
                        let payload = frame.payload.as_bytes();
                        writer.write(payload).await?;
                        writer.flush().await?;
                    }
                    _ => {}
                }
            },
            _ = reader.readable() => {
                match reader.try_read(&mut con_buf) {
                    Ok(0) => bail!("Stream closed amidst reading"),
                    Ok(n) => {
                        let new = &con_buf[..n];
                        let ws_frame = Frame::new(true, OpCode::Binary, None, Payload::Borrowed(new));
                        ws.write_frame(ws_frame).await?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // skip iteration and allow event loop to continue
                        continue;
                    }
                    Err(e) => bail!(e),
                };

            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("Listening on 8000");

    let mut http = hyper::server::conn::http1::Builder::new();

    loop {
        http.keep_alive(true);

        loop {
            let (stream, _) = listener.accept().await?;

            let connection = http
                .serve_connection(
                    TokioIo::new(stream),
                    hyper::service::service_fn(server_upgrade),
                )
                .with_upgrades();

            tokio::spawn(async move {
                if let Err(err) = connection.await {
                    println!("Error serving HTTP connection: {err:?}");
                }
            });
        }
    }
}

async fn server_upgrade(mut req: Request<Incoming>) -> anyhow::Result<Response<Empty<Bytes>>> {
    let (response, fut) = upgrade::upgrade(&mut req)?;

    tokio::spawn(async move {
        if let Err(e) = handle_as_websocket(fut).await {
            eprintln!("Error in websocket connection: {}", e);
        }
    });

    Ok(response)
}
