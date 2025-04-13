use std::str;

use winnow::{
    ascii::{alpha1, crlf, space0, Caseless},
    combinator::{alt, empty, repeat, seq, terminated},
    token::{take, take_till},
    Parser,
};

use crate::spec::{
    message::{MessageBody, MessageHeader},
    protocol::HttpVersion,
    request::{Method, Request, RequestLine, RequestURI},
};

use super::{base::Parse, util::is_space};

impl Parse for Method {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let method: Method = alt((
            Caseless("get").map(|_| Method::Get),
            Caseless("post").map(|_| Method::Post),
            alpha1.map(|m: &[u8]| Method::Extension(m.to_vec())),
        ))
        .parse_next(input)?;
        Ok(method)
    }
}

impl Parse for RequestURI {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let uri = take_till(1.., is_space)
            .map(|uri: &[u8]| RequestURI(uri.to_vec()))
            .parse_next(input)?;
        Ok(uri)
    }
}

impl Parse for RequestLine {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let request_line = seq! {RequestLine {
            _: space0,
            method: Method::parse,
            _: space0,
            request_uri: RequestURI::parse,
            _: space0,
            http_version: HttpVersion::parse,
            _: space0,
            _: "\r\n",

        }}
        .parse_next(input)?;
        Ok(request_line)
    }
}

impl Parse for Request {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let mut request: Request = seq! {
            Request {
                request_line: RequestLine::parse,
                headers: repeat(0.., terminated(MessageHeader::parse, crlf)),
                _: crlf,
                body: empty.map(|_| None),
            }
        }
        .parse_next(input)?;

        // whether we should read body
        request.body = match request.find_value(b"Content-Length") {
            Some(content_length) => {
                if let Ok(Ok(value)) = str::from_utf8(&content_length).map(|s| s.parse::<u32>()) {
                    let body = take(value).parse_next(input)?;
                    Some(MessageBody(body.to_vec()))
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(request)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        spec::message::{FieldContent, FieldName, FieldValue},
        test_parse_ok,
    };

    use super::*;

    test_parse_ok!(get, b"get", Method::Get, b"");
    test_parse_ok!(get_case, b"Get", Method::Get, b"");
    test_parse_ok!(get_trailing, b"get ", Method::Get, b" ");
    test_parse_ok!(post, b"post", Method::Post, b"");
    test_parse_ok!(post_case, b"Post", Method::Post, b"");
    test_parse_ok!(post_trailing, b"post ", Method::Post, b" ");
    test_parse_ok!(
        get_extension,
        b"Something",
        Method::Extension(b"Something".to_vec()),
        b""
    );
    test_parse_ok!(
        get_extension_trailing,
        b"Something ",
        Method::Extension(b"Something".to_vec()),
        b" "
    );

    test_parse_ok!(
        request_uri,
        b"http://localhost",
        RequestURI(b"http://localhost".to_vec()),
        b""
    );
    test_parse_ok!(
        request_uri_trailing,
        b"http://localhost ",
        RequestURI(b"http://localhost".to_vec()),
        b" "
    );
    test_parse_ok!(
        request_uri_crlf,
        b"http://localhost\r\n",
        RequestURI(b"http://localhost".to_vec()),
        b"\r\n"
    );

    test_parse_ok!(
        request_line,
        b"GET /user-agent HTTP/1.1\r\n",
        RequestLine {
            method: Method::Get,
            request_uri: RequestURI(b"/user-agent".to_vec()),
            http_version: HttpVersion { major: 1, minor: 1 }
        },
        b""
    );
    test_parse_ok!(
        request_line_leading,
        b"  \tGET /user-agent HTTP/1.1\r\n",
        RequestLine {
            method: Method::Get,
            request_uri: RequestURI(b"/user-agent".to_vec()),
            http_version: HttpVersion { major: 1, minor: 1 }
        },
        b""
    );
    test_parse_ok!(
        request_line_trailing,
        b"GET /user-agent HTTP/1.1  \t\r\n",
        RequestLine {
            method: Method::Get,
            request_uri: RequestURI(b"/user-agent".to_vec()),
            http_version: HttpVersion { major: 1, minor: 1 }
        },
        b""
    );

    test_parse_ok!(
        request_no_body,
        b"GET /user-agent HTTP/2.0\r
Host: localhost:4221\r
User-Agent: foobar/1.2.3\r
Accept: */*\r
\r
",
        Request {
            request_line: RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 },
            },
            headers: vec![
                MessageHeader {
                    field_name: FieldName(b"Host".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"localhost:4221".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"User-Agent".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"foobar/1.2.3".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"Accept".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"*/*".to_vec())])),
                },
            ],
            body: None,
        },
        b""
    );
    test_parse_ok!(
        request_no_header,
        b"GET /user-agent HTTP/2.0\r
\r
",
        Request {
            request_line: RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 },
            },
            headers: vec![],
            body: None,
        },
        b""
    );
    test_parse_ok!(
        request_with_body,
        b"GET /user-agent HTTP/2.0\r
Host: localhost:4221\r
User-Agent: foobar/1.2.3\r
Accept: */*\r
Content-Length: 10\r
\r
0123456789",
        Request {
            request_line: RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 },
            },
            headers: vec![
                MessageHeader {
                    field_name: FieldName(b"Host".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"localhost:4221".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"User-Agent".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"foobar/1.2.3".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"Accept".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"*/*".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"Content-Length".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"10".to_vec())])),
                },
            ],
            body: Some(MessageBody(b"0123456789".into())),
        },
        b""
    );
    test_parse_ok!(
        request_no_header_and_body,
        b"GET /user-agent HTTP/2.0\r
\r
",
        Request {
            request_line: RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 },
            },
            headers: vec![],
            body: None,
        },
        b""
    );
    test_parse_ok!(
        request_with_body_no_content_length,
        b"GET /user-agent HTTP/2.0\r
Host: localhost:4221\r
User-Agent: foobar/1.2.3\r
Accept: */*\r
\r
0123456789",
        Request {
            request_line: RequestLine {
                method: Method::Get,
                request_uri: RequestURI(b"/user-agent".to_vec()),
                http_version: HttpVersion { major: 2, minor: 0 },
            },
            headers: vec![
                MessageHeader {
                    field_name: FieldName(b"Host".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"localhost:4221".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"User-Agent".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"foobar/1.2.3".to_vec())])),
                },
                MessageHeader {
                    field_name: FieldName(b"Accept".to_vec()),
                    field_value: Some(FieldValue(vec![FieldContent(b"*/*".to_vec())])),
                },
            ],
            body: None,
        },
        b"0123456789"
    );
}
