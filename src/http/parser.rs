use crate::http::{HttpMethod, HttpParser};
use nom::{
    AsChar, IResult, Parser,
    bytes::streaming::{tag, take_while, take_while1},
    character::streaming::{digit1, space0, space1},
    combinator::map,
    error::{Error, ErrorKind},
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
};

/* nom start */
fn word(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(|c: u8| !c.is_space())(input)
}

fn carriage_return(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("\r\n")(input)
}

fn parse_method(input: &[u8]) -> IResult<&[u8], HttpMethod> {
    word(input).map(|(x, y)| (x, HttpMethod::from(y)))
}

fn parse_http_version(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    let version_parser = separated_pair(digit1, tag("."), digit1);

    preceded(
        tag("HTTP/"),
        map(version_parser, |(x, y): (&[u8], &[u8])| -> (u8, u8) {
            // digit1 parser already confirmed its a valid ascii digit and a number.

            let x = std::str::from_utf8(x).unwrap();
            let y = std::str::from_utf8(y).unwrap();

            let x: u8 = x.parse().unwrap();
            let y: u8 = y.parse().unwrap();

            return (x, y);
        }),
    )
    .parse(input)
}

fn parse_start_line(input: &[u8]) -> IResult<&[u8], ()> {
    map(
        (
            terminated(parse_method, space1),
            terminated(word, space1),
            terminated(parse_http_version, carriage_return),
        ),
        |_| (),
    )
    .parse(input)
}

/* Start line done */
fn header_key_char(c: u8) -> bool {
    // RFC 7230: token = 1*tchar
    matches!(c,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' |
        b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'
    )
}

fn header_value_char(c: u8) -> bool {
    // accept any visible ASCII or space except CR/LF
    c != b'\r' && c != b'\n'
}

fn parse_header_line(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    separated_pair(
        terminated(take_while1(|x: u8| header_key_char(x)), space0),
        tag(":"),
        terminated(
            preceded(space0, take_while(header_value_char)),
            carriage_return,
        ),
    )
    .parse(input)
}

/* nom end */

pub struct HttpStreamParser;
impl HttpParser for HttpStreamParser {
    fn parse_websocket_key<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
        let (remaining, headers) = map(
            (
                parse_start_line,
                terminated(many0(parse_header_line), carriage_return),
            ),
            |(_, y)| y,
        )
        .parse(input)?;

        let sec_key_header = headers.into_iter().find(|(key, _)| {
            let key = unsafe { std::str::from_utf8_unchecked(key) };
            str::eq_ignore_ascii_case(key, "Sec-WebSocket-Key")
        });

        match sec_key_header {
            Some((_, value)) => Ok((remaining, value)),
            None => Err(nom::Err::Error(Error::new(input, ErrorKind::Fail))),
        }
    }
}
