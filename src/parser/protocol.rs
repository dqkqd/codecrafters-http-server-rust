use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map_res,
    Parser,
};

use super::{util::vec_u8_to_u32, Parse};

#[derive(Debug, PartialEq, Eq)]
pub struct HttpVersion {
    pub major: u32,
    pub minor: u32,
}

impl Parse for HttpVersion {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let (input, (_, major, _, minor)) = (
            tag("HTTP/"),
            map_res(digit1, vec_u8_to_u32),
            char('.'),
            map_res(digit1, vec_u8_to_u32),
        )
            .parse(i)?;
        Ok((input, HttpVersion { major, minor }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_http_version() -> Result<()> {
        let (_, http_version) = HttpVersion::parse(b"HTTP/1.0")?;
        assert_eq!(http_version, HttpVersion { major: 1, minor: 0 });
        Ok(())
    }
}
