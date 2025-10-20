use crate::NETWORK_BUFFER;
use crate::http::{HttpParser, parser::HttpStreamParser};
use anyhow::{self, bail};
use base64::{engine::general_purpose::STANDARD, write::EncoderWriter};
use bytes::{Buf, BufMut, BytesMut};
use std::io::{IoSlice, Result, Write};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use wstcpproxy::debug_print;

const HANDSHAKE_RESPONSE_PARTIAL: &[u8] = b"\
HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ";

const MAGIC_STRING: &'static [u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const SHA1_DIGEST_LEN: usize = 20;
const BASE64_ENCODED_DIGEST_LEN: usize = 4 * ((SHA1_DIGEST_LEN + 2) / 3); // = 28

pub async fn handle_http_upgrade(
    mut stream: &mut TcpStream,
    buffer: &mut BytesMut,
) -> anyhow::Result<()> {
    loop {
        stream.readable().await?;
        let mut _b = [0; NETWORK_BUFFER];
        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        let (remaining, key) = match stream.try_read(&mut _b) {
            Ok(0) => bail!("Stream closed amidst reading"),
            Ok(n) => {
                buffer.put(&_b[..n]);

                if buffer.len() > NETWORK_BUFFER {
                    bail!("HOW COME YOUR REQUEST STILL DOESN'T HAVE THE KEY HEADER?")
                }

                match HttpStreamParser::parse_websocket_key(&buffer) {
                    Ok((rem, key)) => (rem.len(), key),
                    Err(nom::Err::Incomplete(_)) => continue,
                    Err(_) => bail!("Failed to parse HTTP for websocket key"),
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => bail!(e),
        };
        let key = derive_response_key(key);
        perform_handshake(&mut stream, &key).await?;
        buffer.advance(buffer.len() - remaining);
        break;
    }
    Ok(())
}

pub async fn perform_handshake<T: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut T,
    key: &[u8],
) -> Result<()> {
    let handshake_slice = IoSlice::new(HANDSHAKE_RESPONSE_PARTIAL);
    let key_slice = IoSlice::new(key);
    let returns = IoSlice::new(b"\r\n\r\n");

    stream
        .write_vectored(&[handshake_slice, key_slice, returns])
        .await?;
    stream.flush().await?;
    Ok(())
}

pub fn derive_response_key(key: &[u8]) -> [u8; BASE64_ENCODED_DIGEST_LEN] {
    let sha1_hash = {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(&key);
        hasher.update(&MAGIC_STRING);
        hasher.digest().bytes()
    };

    let mut return_key = [0; BASE64_ENCODED_DIGEST_LEN];

    {
        let mut encoder = EncoderWriter::new(&mut return_key[..], &STANDARD);
        encoder.write_all(&sha1_hash).unwrap();
        encoder.flush().unwrap();
    }

    return return_key;
}
