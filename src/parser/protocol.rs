use std::str;

use winnow::{ascii::digit1, combinator::seq, Parser};

use super::base::Parse;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct HttpVersion {
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
        let http_version = seq! {HttpVersion {
            _: "HTTP/",
            major: digit1
                .try_map(|s| str::from_utf8(s))
                .try_map(|s| s.parse::<u32>()),
            _: '.',
            minor: digit1
                .try_map(|s| str::from_utf8(s))
                .try_map(|s| s.parse::<u32>()),

        }}
        .parse_next(input)?;

        Ok(http_version)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_parse_ok;

    test_parse_ok!(
        http_version,
        b"HTTP/1.0 ",
        HttpVersion { major: 1, minor: 0 },
        b" "
    );
}
