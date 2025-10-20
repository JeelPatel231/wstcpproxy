mod http;
use crate::http::{HttpParser, handshake::derive_response_key, parser::HttpStreamParser};
use anyhow;
use http::handshake::perform_handshake;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, duplex},
    net::TcpStream,
};
use wstcpproxy::debug_print;

static HTTP_REQUEST: &'static [u8] =
    b"PUT /foo/bar;param/seg%20ment/@host?name=value&x=%21%40%23 HTTP/1.1\r\n\
Host: example.com\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
User-Agent: MyCustomClient/1.0\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: en-US,en;q=0.5\r\n\
Accept-Encoding: gzip, deflate\r\n\
Connection: keep-alive\r\n\
X-Custom-Header: value with spaces\t and tabs\t\r\n\
X-Empty-Header: \r\n\
Duplicate-Header: first\r\n\
Duplicate-Header: second\r\n\
\r\n\
sadfasdfdasf";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("STARTED MAIN");

    let (mut client, mut server) = duplex(1024);
    server.write_all(HTTP_REQUEST).await.unwrap();
    server.flush().await?;
    server.shutdown().await?;

    println!(">>> Send");
    println!("{}", unsafe { std::str::from_utf8_unchecked(HTTP_REQUEST) });

    ////////
    // grab socket, parse the http request and get the header value in &str or &[u8]
    let input = HTTP_REQUEST;
    let key = debug_print!(HttpStreamParser::parse_websocket_key(input))?;

    // pass that string to a function which derives the response key in base64 encoded format.
    let response_key = debug_print!(derive_response_key(key));

    // pass that key to handshake function which returns HTTP Response text with the key to upgrade the connection.
    perform_handshake(&mut client, &response_key).await?;

    client.shutdown().await?;

    println!("<<< Recv");

    let mut a = String::new();
    server.read_to_string(&mut a).await?;

    println!("{a}");

    println!("ENDED MAIN");
    Ok(())
}
