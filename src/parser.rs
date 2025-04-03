use core::str;

use anyhow::Result;
use nom::{
    bytes::complete::{tag, take_till, take_till1, take_while, take_while1},
    combinator::map_res,
    multi::many0,
    AsChar, IResult, Parser,
};

#[derive(Debug, PartialEq, Eq)]
enum Method {
    Get,
    ExtensionMethod(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
struct HttpVersion {
    major: u32,
    minor: u32,
}

#[derive(Debug)]
enum HttpMessage {
    Request(Request),
    Response(Response),
}
#[derive(Debug)]
struct Request {}
#[derive(Debug)]
struct Response {}

#[derive(Debug)]
enum StartLine {
    RequestLine(RequestLine),
    StatusLine(StatusLine),
}
#[derive(Debug, PartialEq, Eq)]
struct RequestLine {
    method: Method,
    request_uri: Vec<u8>,
    http_version: HttpVersion,
}
#[derive(Debug)]
struct StatusLine {}

#[derive(Debug, PartialEq, Eq)]
struct MessageHeader {
    field_name: FieldName,
    field_value: Option<FieldValue>,
}
#[derive(Debug, PartialEq, Eq)]
struct FieldName(Vec<u8>);
#[derive(Debug, PartialEq, Eq)]
struct FieldValue(Vec<FieldContent>);
#[derive(Debug, PartialEq, Eq)]
struct FieldContent(Vec<u8>);

fn vec_u8_to_u32(i: &[u8]) -> Result<u32> {
    let v = str::from_utf8(i)?.parse()?;
    Ok(v)
}

fn is_space_or_newline(c: u8) -> bool {
    AsChar::is_space(c) || AsChar::is_newline(c)
}

fn skip_space_or_newline(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_space_or_newline)(i)
}

fn expect_space(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_space_or_newline)(i)
}

fn expect_crlf(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("\r\n")(i)
}

fn parse_method(i: &[u8]) -> IResult<&[u8], Method> {
    let mut method = take_while1(AsChar::is_alpha);
    let (input, method) = (method).parse(i)?;
    let method = match method.to_ascii_lowercase().as_slice() {
        b"get" => Method::Get,
        _ => Method::ExtensionMethod(method.to_vec()),
    };
    Ok((input, method))
}

fn parse_http_version(i: &[u8]) -> IResult<&[u8], HttpVersion> {
    let http = tag("HTTP/");
    let major = map_res(take_while1(AsChar::is_dec_digit), vec_u8_to_u32);
    let dot = tag(".");
    let minor = map_res(take_while1(AsChar::is_dec_digit), vec_u8_to_u32);

    let (input, (_, major, _, minor)) = (http, major, dot, minor).parse(i)?;
    Ok((input, HttpVersion { major, minor }))
}

fn parse_request_uri(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_till(is_space_or_newline)(i)
}

fn parse_request_line(i: &[u8]) -> IResult<&[u8], RequestLine> {
    let (input, (_, method, _, request_uri, _, http_version, _, _)) = (
        skip_space_or_newline,
        parse_method,
        expect_space,
        parse_request_uri,
        expect_space,
        parse_http_version,
        skip_space_or_newline,
        expect_crlf,
    )
        .parse(i)?;

    Ok((
        input,
        RequestLine {
            method,
            request_uri: request_uri.to_vec(),
            http_version,
        },
    ))
}

fn parse_field_name(i: &[u8]) -> IResult<&[u8], FieldName> {
    let field_name = take_till1(|c: u8| c == b':');
    let (input, (_, field_name)) = (skip_space_or_newline, field_name).parse(i)?;
    Ok((input, FieldName(field_name.trim_ascii_end().to_vec())))
}

fn parse_field_content(i: &[u8]) -> IResult<&[u8], FieldContent> {
    let field_content = take_till1(|c: u8| is_space_or_newline(c) || c == b'\r');
    let (input, (_, field_content, _)) =
        (skip_space_or_newline, field_content, skip_space_or_newline).parse(i)?;
    Ok((input, FieldContent(field_content.to_vec())))
}

fn parse_field_value(i: &[u8]) -> IResult<&[u8], FieldValue> {
    let (input, field_contents) = many0(parse_field_content).parse(i)?;
    Ok((input, FieldValue(field_contents)))
}

fn parse_message_header(i: &[u8]) -> IResult<&[u8], MessageHeader> {
    let colon = tag(":");
    let (input, (field_name, _, field_value)) =
        (parse_field_name, colon, parse_field_value).parse(i)?;

    let field_value = match !field_value.0.is_empty() {
        true => Some(field_value),
        false => None,
    };

    Ok((
        input,
        MessageHeader {
            field_name,
            field_value,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_skip_empty() -> Result<()> {
        let (i, _) = skip_space_or_newline(b"  one")?;
        assert_eq!(i, b"one");

        let (i, _) = skip_space_or_newline(
            b"
one",
        )?;
        assert_eq!(i, b"one");
        Ok(())
    }

    #[test]
    fn test_parse_method() -> Result<()> {
        let (_, method) = parse_method(b"get")?;
        assert_eq!(method, Method::Get);

        let (_, method) = parse_method(b"Get ")?;
        assert_eq!(method, Method::Get);

        let (_, method) = parse_method(b"Something ")?;
        assert_eq!(method, Method::ExtensionMethod(b"Something".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_http_version() -> Result<()> {
        let (_, http_version) = parse_http_version(b"HTTP/1.0")?;
        assert_eq!(http_version, HttpVersion { major: 1, minor: 0 });
        Ok(())
    }

    #[test]
    fn test_parse_request_line() -> Result<()> {
        let (_, request_line) = parse_request_line(
            b"
GET
/user-agent
HTTP/1.1\r\n",
        )?;
        assert_eq!(
            request_line,
            RequestLine {
                method: Method::Get,
                request_uri: b"/user-agent".to_vec(),
                http_version: HttpVersion { major: 1, minor: 1 }
            }
        );

        let (_, request_line) = parse_request_line(
            b"
GET
/user-agent
HTTP/1.1
\r\n",
        )?;
        assert_eq!(
            request_line,
            RequestLine {
                method: Method::Get,
                request_uri: b"/user-agent".to_vec(),
                http_version: HttpVersion { major: 1, minor: 1 }
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_field_name() -> Result<()> {
        let (_, field_name) = parse_field_name(
            b"
Content-Type: 3\r\n",
        )?;
        assert_eq!(field_name, FieldName(b"Content-Type".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_content() -> Result<()> {
        let (input, field_content) = parse_field_content(
            b"
ab  \tcd\r\n
",
        )?;
        assert_eq!(field_content, FieldContent(b"ab".to_vec()));

        let (input, field_content) = parse_field_content(input)?;
        assert_eq!(field_content, FieldContent(b"cd".to_vec()));

        assert!(parse_field_content(input).is_err());
        Ok(())
    }

    #[test]
    fn test_parse_field_value() -> Result<()> {
        let (input, field_value) = parse_field_value(
            b"
ab  \tcd\r\n
",
        )?;
        assert_eq!(
            field_value,
            FieldValue(vec![
                FieldContent(b"ab".to_vec()),
                FieldContent(b"cd".to_vec())
            ])
        );

        let (_, field_value) = parse_field_value(input)?;
        assert_eq!(field_value, FieldValue(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_message_header() -> Result<()> {
        let (_, message_header) = parse_message_header(
            b"
Content-Length: 3 4 5
",
        )?;

        assert_eq!(
            message_header,
            MessageHeader {
                field_name: FieldName(b"Content-Length".to_vec()),
                field_value: Some(FieldValue(vec![
                    FieldContent(b"3".to_vec()),
                    FieldContent(b"4".to_vec()),
                    FieldContent(b"5".to_vec()),
                ]))
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_message_empty_value() -> Result<()> {
        let (_, message_header) = parse_message_header(
            b"
Content-Length:
",
        )?;

        assert_eq!(
            message_header,
            MessageHeader {
                field_name: FieldName(b"Content-Length".to_vec()),
                field_value: None,
            }
        );

        Ok(())
    }
}
