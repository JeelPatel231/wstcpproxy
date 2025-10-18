use nom::{
    AsChar, IResult, Parser,
    bytes::{complete::tag, take_while, take_while1},
    character::{
        char,
        complete::{self, not_line_ending, space0},
    },
    error::ParseError,
    multi::many0,
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

type HTTPInput<'a> = &'a str;

type MajorVersion = u8;
type MinorVersion = u8;

type HeaderKey<'a> = &'a str;
type HeaderValue<'a> = &'a str;

// TODO: implement full parser for this.
#[derive(Debug)]
pub struct HttpRequestMetadata<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
    pub version: (MajorVersion, MinorVersion),
    pub headers: Vec<(HeaderKey<'a>, HeaderValue<'a>)>,
}

fn is_ctl_or_separator(c: char) -> bool {
    // Control chars: ASCII 0–31 or 127
    if c.is_ascii_control() {
        return true;
    }

    // Separators from RFC 2616, (HTTP 1.1)
    matches!(
        c,
        '(' | ')'
            | '<'
            | '>'
            | '@'
            | ','
            | ';'
            | ':'
            | '\\'
            | '"'
            | '/'
            | '['
            | ']'
            | '?'
            | '='
            | '{'
            | '}'
            | ' '
            | '\t'
    )
}

fn is_token_char(c: char) -> bool {
    c.is_ascii() && !is_ctl_or_separator(c)
}

fn parse_carriage_return<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    tag("\r\n").parse(input)
}

fn token<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while1(is_token_char).parse(input)
}

fn parse_method<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, HttpMethod, E> {
    take_while(|x: char| x.is_alpha())
        .parse(input)
        .map(|(rem, parsed)| {
            // method names are case sensitive
            let method = match parsed {
                "GET" => HttpMethod::Get,
                "HEAD" => HttpMethod::Head,
                "POST" => HttpMethod::Post,
                "PUT" => HttpMethod::Put,
                "DELETE" => HttpMethod::Delete,
                "CONNECT" => HttpMethod::Connect,
                "OPTIONS" => HttpMethod::Options,
                "TRACE" => HttpMethod::Trace,
                "PATCH" => HttpMethod::Patch,
                _ => unreachable!(),
            };

            return (rem, method);
        })
}

fn parse_http_version<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (u8, u8), E> {
    let (input, _) = tag("HTTP/").parse(input)?;

    let (input, major_version) = complete::digit1(input)
        .map(|(rem, parsed)| (rem, u8::from_str_radix(parsed, 10).unwrap()))?;

    let (input, _) = char('.').parse(input)?;

    let (input, minor_version) = complete::digit1(input)
        .map(|(rem, parsed)| (rem, u8::from_str_radix(parsed, 10).unwrap()))?;

    return Ok((input, (major_version, minor_version)));
}

fn parse_header_key<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, HeaderKey<'a>, E> {
    token::<E>(input).map(|(rem, parsed)| (rem, parsed))
}

fn single_header<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (HeaderKey<'a>, HeaderValue<'a>), E> {
    let (input, header_key) = parse_header_key(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':').parse(input)?;
    let (input, _) = space0(input)?;
    let (input, header_value) = parse_header_value(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = parse_carriage_return(input)?;

    return Ok((input, (header_key, header_value)));
}

fn parse_until_whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    // just take everything until a space is seen
    take_while(|x: char| !x.is_ascii_whitespace())
        .parse(input)
        .map(|(rem, parsed)| (rem, parsed))
}

// TODO: not RFC compliant
fn parse_header_value<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, HeaderValue<'a>, E> {
    // THIS NEEDS TO BE FIXED AS PER HTTP RFC, but parse everything for now.
    not_line_ending
        .parse(input)
        .map(|(rem, parsed)| (rem, parsed))
}

fn optional_multiple_headers<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Vec<(HeaderKey<'a>, HeaderValue<'a>)>, E> {
    many0(single_header).parse(input)
}

pub fn parse_http_message<'a, E: ParseError<&'a str>>(
    input: HTTPInput<'a>,
) -> IResult<HTTPInput<'a>, HttpRequestMetadata<'a>, E> {
    let (input, method) = parse_method(input)?;
    let (input, _) = char(' ').parse(input)?;
    let (input, path) = parse_until_whitespace(input)?;
    let (input, _) = char(' ').parse(input)?;
    let (input, version) = parse_http_version(input)?;
    let (input, _) = parse_carriage_return(input)?;
    let (input, headers) = optional_multiple_headers(input)?;
    let (input, _) = parse_carriage_return(input)?;

    return Ok((
        input,
        HttpRequestMetadata {
            method,
            path,
            version,
            headers,
        },
    ));
}
