use core::str;

use anyhow::Result;
use nom::{
    branch::alt,
    bytes::{complete::is_not, tag},
    character::complete::space1,
    combinator::{eof, opt},
    sequence::preceded,
    IResult, Parser,
};

pub(super) fn vec_u8_to_u32(i: &[u8]) -> Result<u32> {
    let v = str::from_utf8(i)?.parse()?;
    Ok(v)
}

type ParserResult<'a, T> = IResult<&'a [u8], &'a T>;

pub(super) fn until_space1(i: &[u8]) -> ParserResult<[u8]> {
    is_not(" \t\r\n")(i)
}

pub(super) fn lws(i: &[u8]) -> ParserResult<[u8]> {
    // LWS = [CRLF] 1*( SP | HT )
    alt((eof, preceded(opt(tag("\r\n")), space1))).parse(i)
}

pub(super) fn many_lws(i: &[u8]) -> ParserResult<[u8; 0]> {
    let mut i = i;
    while !i.is_empty() {
        match lws.parse(i) {
            Ok(out) => i = out.0,
            Err(_) => break,
        }
    }
    Ok((i, b""))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lws_no_crlf() -> Result<()> {
        let (input, _) = lws(b" \t abc")?;
        assert_eq!(input, b"abc");
        Ok(())
    }

    #[test]
    fn test_lws_crlf() -> Result<()> {
        let (input, _) = lws(b"\r\n\t abc")?;
        assert_eq!(input, b"abc");
        Ok(())
    }

    #[test]
    fn test_lws_empty() -> Result<()> {
        let (input, _) = lws(b"")?;
        assert_eq!(input, b"");
        Ok(())
    }

    #[test]
    fn test_lws_only_space() -> Result<()> {
        let (input, _) = lws(b"   ")?;
        assert_eq!(input, b"");
        Ok(())
    }

    #[test]
    fn test_many_lws() -> Result<()> {
        let (input, _) = many_lws(b" \t abc")?;
        assert_eq!(input, b"abc");
        Ok(())
    }

    #[test]
    fn test_many_lws_empty() -> Result<()> {
        let (input, _) = many_lws(b"")?;
        assert_eq!(input, b"");
        Ok(())
    }

    #[test]
    fn test_many_lws_only_space() -> Result<()> {
        let (input, _) = many_lws(b"  ")?;
        assert_eq!(input, b"");
        Ok(())
    }
}
