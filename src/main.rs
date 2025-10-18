mod http;

use http::parser::parse_http_message;
use nom::Parser;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

static HTTP_REQUEST: &'static str = "PUT /foo/bar;param/seg%20ment/@host?name=value&x=%21%40%23 HTTP/1.1\r\n\
Host: example.com\r\n\
User-Agent: MyCustomClient/1.0\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: en-US,en;q=0.5\r\n\
Accept-Encoding: gzip, deflate\r\n\
Connection: keep-alive\r\n\
X-Custom-Header: value with spaces and tabs\t\r\n\
X-Empty-Header: \r\n\
Duplicate-Header: first\r\n\
Duplicate-Header: second\r\n\
\r\n\
sadfasdfdasf";

#[tokio::main]
async fn main() -> Result<()> {
    let http_message = HTTP_REQUEST;

    let (_, parsed) = parse_http_message::<nom::error::Error<&str>>.parse(http_message)?;

    let connection_value: Vec<&str> = parsed
        .headers
        .into_iter()
        .filter_map(|(x, y)| {
            if x.eq_ignore_ascii_case("duplicate-header") {
                Some(y)
            } else {
                None
            }
        })
        .collect();

    dbg!(connection_value);

    Ok(())
}
