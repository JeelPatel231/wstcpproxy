pub mod handshake;
pub mod parser;
use anyhow;

#[derive(Debug)]
enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl From<&[u8]> for HttpMethod {
    fn from(value: &[u8]) -> Self {
        match value {
            b"GET" => HttpMethod::Get,
            b"HEAD" => HttpMethod::Head,
            b"POST" => HttpMethod::Post,
            b"PUT" => HttpMethod::Put,
            b"DELETE" => HttpMethod::Delete,
            b"CONNECT" => HttpMethod::Connect,
            b"OPTIONS" => HttpMethod::Options,
            b"TRACE" => HttpMethod::Trace,
            b"PATCH" => HttpMethod::Patch,
            _ => unreachable!(),
        }
    }
}

pub trait HttpParser {
    fn parse_websocket_key<'a>(input: &'a [u8]) -> anyhow::Result<&'a [u8]>;
}
