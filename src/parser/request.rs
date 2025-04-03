use nom::{
    branch::alt,
    bytes::complete::{is_not, tag_no_case, take_until},
    character::{
        complete::{alpha1, crlf, multispace0},
        streaming::multispace1,
    },
    combinator::map,
    Parser,
};

use super::{protocol::HttpVersion, Parse};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Method {
    Get,
    ExtensionMethod(Vec<u8>),
}

// TODO: handle uri
// TODO: add test
#[derive(Debug, PartialEq, Eq)]
pub(super) struct RequestURI(Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RequestLine {
    method: Method,
    request_uri: RequestURI,
    http_version: HttpVersion,
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
        let (input, (_, method, _, request_uri, _, http_version, _, _)) = (
            multispace0,
            Method::parse,
            multispace1,
            RequestURI::parse,
            multispace1,
            HttpVersion::parse,
            take_until("\r\n"),
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
    fn test_parse_request_line() -> Result<()> {
        let (_, request_line) = RequestLine::parse(
            b"
GET
/user-agent
HTTP/1.1\r\n",
        )?;
        assert_eq!(
            request_line,
            RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 1, minor: 1 }
            }
        );

        let (_, request_line) = RequestLine::parse(
            b"
GET
/user-agent
HTTP/2.0
\r\n",
        )?;
        assert_eq!(
            request_line,
            RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 }
            }
        );
        Ok(())
    }
}
