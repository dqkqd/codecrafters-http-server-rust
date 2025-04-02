use anyhow::Result;
use std::io::Write;

const CRLF: &str = "\r\n";

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    NotFound,
}

#[derive(Debug)]
pub struct HttpHeader {
    content_type: String,
    content_length: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Response {
    header: Option<HttpHeader>,
    body: Option<String>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    response: Response,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> HttpResponse {
        HttpResponse {
            status,
            response: Response::default(),
        }
    }

    pub fn with_text_response(mut self, text: &str) -> HttpResponse {
        self.response = Response::text_response(text);
        self
    }

    pub fn output<W: Write>(self, w: &mut W) -> Result<()> {
        let response = self.to_string();
        dbg!(&response);
        w.write_all(response.as_bytes())?;
        Ok(())
    }
}

impl Response {
    pub fn text_response(s: &str) -> Response {
        let header = Some(HttpHeader {
            content_type: "text/plain".to_string(),
            content_length: s.len(),
        });
        let body = Some(s.to_string());
        Response { header, body }
    }
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "HTTP/1.1 200 OK{CRLF}"),
            HttpStatus::NotFound => write!(f, "HTTP/1.1 404 Not Found{CRLF}"),
        }
    }
}

impl std::fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
Content-Type: {}{CRLF}\
Content-Length: {}{CRLF}",
            self.content_type, self.content_length
        )
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{CRLF}{}",
            self.header
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.body.as_deref().unwrap_or_default()
        )
    }
}

impl std::fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.status, self.response)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_http_status_str_200() {
        assert_eq!(HttpStatus::Ok.to_string(), "HTTP/1.1 200 OK\r\n");
    }

    #[test]
    fn test_http_status_str_400() {
        assert_eq!(
            HttpStatus::NotFound.to_string(),
            "HTTP/1.1 404 Not Found\r\n"
        );
    }

    #[test]
    fn test_http_header() {
        assert_eq!(
            HttpHeader {
                content_type: "text/plain".to_string(),
                content_length: 3
            }
            .to_string(),
            "Content-Type: text/plain\r\nContent-Length: 3\r\n"
        )
    }

    #[test]
    fn test_response_non_empty() {
        let resp = Response::text_response("abc");
        assert_eq!(
            resp.to_string(),
            "\
Content-Type: text/plain\r\n\
Content-Length: 3\r\n\
\r\n\
abc"
        )
    }

    #[test]
    fn test_response_empty() {
        let resp = Response::default();
        assert_eq!(
            resp.to_string(),
            "\
\r\n"
        )
    }

    #[test]
    fn test_http_response_non_empty() {
        let resp = HttpResponse {
            status: HttpStatus::Ok,
            response: Response::text_response("abcd"),
        };

        assert_eq!(
            resp.to_string(),
            "\
HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: 4\r\n\
\r\n\
abcd"
        )
    }

    #[test]
    fn test_http_response_empty() {
        let resp = HttpResponse {
            status: HttpStatus::Ok,
            response: Response::default(),
        };

        assert_eq!(
            resp.to_string(),
            "\
HTTP/1.1 200 OK\r\n\r\n"
        )
    }
}
