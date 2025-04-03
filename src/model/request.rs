use anyhow::{bail, Result};
use bytes::Bytes;

use super::{read_until_crlf, Converter, CRLF};

#[derive(Debug, PartialEq, Eq)]
enum Method {
    Get,
    Extension(Bytes),
}

#[derive(Debug)]
pub(super) struct RequestLine {
    method: Method,
    request_uri: Bytes,
    http_version: Bytes,
}

impl Converter for Method {
    fn from_reader<R: std::io::BufRead>(reader: &mut R) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;
        let method = match buf.to_ascii_lowercase().as_slice() {
            b"get" => Method::Get,
            _ => Method::Extension(Bytes::from(buf)),
        };

        Ok(method)
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Method::Get => write!(writer, "GET")?,
            Method::Extension(bytes) => writer.write_all(bytes)?,
        };
        Ok(())
    }
}

impl Converter for RequestLine {
    fn from_reader<R: std::io::BufRead>(reader: &mut R) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        match read_until_crlf(reader)? {
            Some(request_line) => {
                let mut split = request_line
                    .split(|v| v.is_ascii_whitespace())
                    .filter(|v| !v.is_empty());

                let mut method = split.next().unwrap_or_default();
                let method = Method::from_reader(&mut method)?;

                let request_uri = split.next().unwrap_or_default();
                let request_uri = Bytes::copy_from_slice(request_uri);

                let http_version = split.next().unwrap_or_default();
                let http_version = Bytes::copy_from_slice(http_version);

                Ok(RequestLine {
                    method,
                    request_uri,
                    http_version,
                })
            }
            None => bail!("cannot read request line"),
        }
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        self.method.to_writer(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(&self.request_uri)?;
        writer.write_all(b" ")?;
        writer.write_all(&self.http_version)?;
        writer.write_all(CRLF.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_method_from_bytes() {
        assert_eq!(Method::from_u8(b"get").unwrap(), Method::Get);
        assert_eq!(Method::from_u8(b"Get").unwrap(), Method::Get);
        assert_eq!(Method::from_u8(b"GET").unwrap(), Method::Get);
        assert_eq!(
            Method::from_u8(b"Extension").unwrap(),
            Method::Extension(Bytes::copy_from_slice(b"Extension"))
        );
    }

    #[test]
    fn test_method_to_bytes() {
        assert_eq!(Method::Get.to_bytes().unwrap(), b"GET");
        assert_eq!(
            Method::Extension(Bytes::copy_from_slice(b"SomeMethod"))
                .to_bytes()
                .unwrap(),
            b"SomeMethod"
        );
    }

    #[test]
    fn test_request_line_from_bytes() {
        let r = RequestLine::from_u8(
            b"
    GET
    /user-agent
    HTTP/1.1
    \r\n
    ",
        )
        .unwrap();
        assert_eq!(r.method, Method::Get);
        assert_eq!(r.request_uri, "/user-agent");
        assert_eq!(r.http_version, "HTTP/1.1");
    }

    #[test]
    fn test_request_line_to_bytes() {
        let r = RequestLine::from_u8(
            b"
GET
/user-agent
HTTP/1.1
\r\n
",
        )
        .unwrap();
        assert_eq!(r.to_bytes().unwrap(), b"GET /user-agent HTTP/1.1\r\n");
    }
}
