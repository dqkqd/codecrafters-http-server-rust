use nom::{
    branch::alt,
    bytes::complete::{is_not, tag_no_case},
    character::complete::{alpha1, crlf, space0},
    combinator::map,
    multi::many0,
    sequence::terminated,
    Parser,
};

use super::{
    message::{MessageBody, MessageHeader},
    protocol::HttpVersion,
    Parse,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Method {
    Get,
    ExtensionMethod(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RequestURI(Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RequestLine {
    method: Method,
    request_uri: RequestURI,
    http_version: HttpVersion,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Request {
    request_line: RequestLine,
    headers: Vec<MessageHeader>,
    body: Option<MessageBody>,
}

impl Parse for Method {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        alt((
            map(tag_no_case("get"), |_: &[u8]| Method::Get),
            map(alpha1, |m: &[u8]| Method::ExtensionMethod(m.to_vec())),
        ))
        .parse(i)
    }
}

impl Parse for RequestURI {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        map(is_not(" \t\r\n"), |request_uri: &[u8]| {
            RequestURI(request_uri.to_vec())
        })
        .parse(i)
    }
}

impl Parse for RequestLine {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let (input, (_, method, _, request_uri, _, http_version, _)) = (
            space0,
            Method::parse,
            space0,
            RequestURI::parse,
            space0,
            HttpVersion::parse,
            crlf,
        )
            .parse(i)?;

        Ok((
            input,
            RequestLine {
                method,
                request_uri,
                http_version,
            },
        ))
    }
}

impl Parse for Request {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let (input, (request_line, headers, _, body)) = (
            RequestLine::parse,
            many0(terminated(MessageHeader::parse, crlf)),
            crlf,
            MessageBody::parse,
        )
            .parse(i)?;

        let body = match !body.0.is_empty() {
            true => Some(body),
            false => None,
        };

        Ok((
            input,
            Request {
                request_line,
                headers,
                body,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_method() -> Result<()> {
        let (_, method) = Method::parse(b"get")?;
        assert_eq!(method, Method::Get);

        let (_, method) = Method::parse(b"Get ")?;
        assert_eq!(method, Method::Get);

        let (_, method) = Method::parse(b"Something ")?;
        assert_eq!(method, Method::ExtensionMethod(b"Something".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_request_uri() -> Result<()> {
        let (_, request_uri) = RequestURI::parse(b"http://localhost")?;
        assert_eq!(request_uri, RequestURI(b"http://localhost".to_vec()));

        let (_, request_uri) = RequestURI::parse(b"http://localhost  ")?;
        assert_eq!(request_uri, RequestURI(b"http://localhost".to_vec()));

        Ok(())
    }

    #[test]
    fn test_parse_request_line() -> Result<()> {
        let (_, request_line) = RequestLine::parse(b"GET /user-agent HTTP/1.1\r\n")?;

        assert_eq!(
            request_line,
            RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 1, minor: 1 }
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_request_with_body() -> Result<()> {
        let (_, request) = Request::parse(
            b"GET /user-agent HTTP/2.0\r
Host: localhost:4221\r
User-Agent: foobar/1.2.3\r
Accept: */*\r
\r
message body1
message body2",
        )?;

        assert_eq!(
            request,
            Request {
                request_line: RequestLine::parse(b"GET /user-agent HTTP/2.0\r\n")?.1,
                headers: vec![
                    MessageHeader::parse(b"Host: localhost:4221")?.1,
                    MessageHeader::parse(b"User-Agent: foobar/1.2.3")?.1,
                    MessageHeader::parse(b"Accept: */*")?.1,
                ],
                body: Some(MessageBody::parse(b"message body1\nmessage body2")?.1),
            }
        );
        Ok(())
    }
}
