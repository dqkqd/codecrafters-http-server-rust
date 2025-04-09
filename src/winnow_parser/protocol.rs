use std::str;

use winnow::{ascii::digit1, Parser};

use super::base::Parse;

#[derive(Debug, PartialEq, Eq)]
struct HttpVersion {
    pub major: u32,
    pub minor: u32,
}

impl Parse for HttpVersion {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let (_, major, _, minor) = (
            "HTTP/",
            digit1
                .try_map(|s| str::from_utf8(s))
                .try_map(|s| s.parse::<u32>()),
            '.',
            digit1
                .try_map(|s| str::from_utf8(s))
                .try_map(|s| s.parse::<u32>()),
        )
            .parse_next(input)?;

        Ok(HttpVersion { major, minor })
    }
}

#[cfg(test)]
mod test {
    use crate::winnow_parser::base::StreamParser;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_http_version() -> Result<()> {
        let mut p = StreamParser::new(&b"HTTP/1.0 "[..]);
        let http_version: HttpVersion = p.parse()?;
        assert_eq!(http_version, HttpVersion { major: 1, minor: 0 });
        assert_eq!(p.buffer, b" ");
        Ok(())
    }
}
