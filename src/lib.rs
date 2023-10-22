use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while1},
    character::complete::crlf,
    combinator::{map, value},
    multi::many0,
    sequence::{preceded, separated_pair, terminated, Tuple},
    IResult,
};

#[derive(Debug, Clone)]
pub struct StartLine {
    pub method: HttpMethod,
    pub target: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
}

fn parse_method(input: &str) -> IResult<&str, HttpMethod> {
    alt((
        value(HttpMethod::GET, tag("GET")),
        value(HttpMethod::POST, tag("POST")),
    ))(input)
}

pub fn parse_start_line(line: &str) -> IResult<&str, StartLine> {
    let space = take_while1(|c| c == ' ');
    let url = take_while1(|c| c != ' ');
    let is_version = |c| c >= '0' && c <= '9' || c == '.';
    let http = tag("HTTP/");
    let version = take_while1(is_version);
    let http_version = preceded(http, version);
    let (input, (method, _, url, _, version, _)) =
        (parse_method, &space, url, &space, http_version, crlf).parse(line)?;
    Ok((
        input,
        StartLine {
            method,
            target: url.to_string(),
            version: version.to_string(),
        },
    ))
}

fn parse_header(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(is_not(":"), tag(": "), take_until("\r\n")),
        tag("\r\n"),
    )(input)
}

fn parse_headers(input: &str) -> IResult<&str, HashMap<String, String>> {
    terminated(
        map(many0(parse_header), |l| {
            l.into_iter()
                .map(|(v1, v2)| (v1.to_string(), v2.to_string()))
                .collect()
        }),
        tag("\r\n"),
    )(input)
}

pub fn parse_request<'a>(
    request: &'a str,
) -> IResult<&'a str, (StartLine, HashMap<String, String>, String)> {
    let (rest, start_line) = parse_start_line(&request)?;
    let (rest, headers) = parse_headers(&rest)?;
    Ok(("", (start_line, headers, rest.to_string())))
}
