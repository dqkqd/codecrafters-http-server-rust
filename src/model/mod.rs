use std::io::{BufRead, BufReader, Write};

// mod error;
mod header;
mod message;
mod request;
mod response;
// mod serialize;

use anyhow::Result;

const CRLF: &str = "\r\n";

trait Converter {
    fn from_reader<R: BufRead>(reader: &mut R) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn from_u8(data: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let mut reader = BufReader::new(data);
        let s = Self::from_reader(&mut reader)?;
        Ok(s)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        self.to_writer(&mut buf)?;
        Ok(buf)
    }
}

fn read_until_crlf<R: BufRead>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut buf = vec![];
    loop {
        // read \r
        let _ = reader.read_until(b'\r', &mut buf)?;
        // read \n
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 1 || n == 0 {
            break;
        }
    }

    if buf.ends_with(b"\r\n") {
        buf.truncate(buf.len() - 2);
    }

    if buf.is_empty() {
        Ok(None)
    } else {
        Ok(Some(buf))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;

    #[test]
    fn test_read_until_crlf() {
        let mut cursor = io::Cursor::new(b"zero\r\none\r\n\r\n");

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, Some(b"zero".to_vec()));

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, Some(b"one".to_vec()));

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, None);

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, None);
    }

    #[test]
    fn test_read_until_crlf_only_cr() {
        let mut cursor = io::Cursor::new(b"zero\r\r\r");

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, Some(b"zero\r\r\r".to_vec()));

        let buf = read_until_crlf(&mut cursor).unwrap();
        assert_eq!(buf, None);
    }
}
