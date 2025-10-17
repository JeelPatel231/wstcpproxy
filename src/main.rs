// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use nom::{
    AsChar, IResult, Input, Parser,
    bytes::{complete::tag, take_while},
    character::{
        char,
        complete::{not_line_ending, space0},
    },
    error::ParseError,
    multi::many_till,
    sequence::{self, terminated},
};

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

// TODO: implement full parser for this.
#[derive(Debug)]
struct HttpRequestMetadata<'a> {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: Vec<(HeaderKey<'a>, HeaderValue<'a>)>,
}

#[derive(Debug)]
struct HttpHeaders<'a> {
    pub map: Vec<(HeaderKey<'a>, HeaderValue<'a>)>,
}
#[derive(Debug)]
struct HeaderKey<'a>(&'a str);
#[derive(Debug)]
struct HeaderValue<'a>(&'a str);

struct CarriageReturn;

fn parse_header_key<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, HeaderKey<'a>, E> {
    // is alphabet or dash
    take_while(|x: char| x.is_alpha() || x == '-')
        .parse(input)
        .map(|(rem, parsed)| (rem, HeaderKey(parsed)))
}

fn whitespace_around<I: Input, O, E: ParseError<I>, P: Parser<I, Output = O, Error = E>>(
    parser: P,
) -> impl Parser<I, Output = O, Error = E>
where
    <I as Input>::Item: AsChar,
{
    sequence::delimited(space0, parser, space0)
}

fn parse_header_value<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, HeaderValue<'a>, E> {
    // THIS NEEDS TO BE FIXED AS PER HTTP RFC, but parse everything for now.
    not_line_ending
        .parse(input)
        .map(|(rem, parsed)| (rem, HeaderValue(parsed)))
}

fn parse_carriage_return<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, CarriageReturn, E> {
    tag("\r\n")
        .parse(input)
        .map(|(rem, _)| (rem, CarriageReturn))
}

fn single_header<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (HeaderKey<'a>, HeaderValue<'a>), E> {
    let key_delimited = whitespace_around(parse_header_key);
    let value_delimited = whitespace_around(parse_header_value);
    let header = sequence::separated_pair(key_delimited, char(':'), value_delimited);

    terminated(header, parse_carriage_return).parse(input)
}

fn multiple_headers<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, HttpHeaders<'a>, E> {
    many_till(single_header, parse_carriage_return)
        .parse(input)
        .map(|(rem, (map, _))| (rem, HttpHeaders { map }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: parse the start line of http as well.

    // HTTP MESSAGE, we don't really care about the body so we will just drop after parsing metadata.
    // start-line -> optional headers -> carriage return -> optional body -> EOF
    let string = "Hello-World:Hello&World\r\nSec-WebSocket-Key:87tgb9786f67tv7\r\n\r\n";
    let (_remaining, parsed) = multiple_headers::<nom::error::Error<&str>>(string)?;

    let (key, value) = parsed
        .map
        .iter()
        .find(|(key, _)| key.0.to_lowercase() == "sec-websocket-key")
        .ok_or("Api Key Not Found!")?;

    dbg!(key, value);

    Ok(())
}
