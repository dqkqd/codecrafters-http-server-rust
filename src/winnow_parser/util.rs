use winnow::{
    ascii::space0,
    combinator::opt,
    stream::{AsBytes, AsChar},
    Parser,
};

use super::base::Parse;

pub(super) fn is_space<T: AsChar>(c: T) -> bool {
    " \t\r\n".contains(c.as_char())
}

#[derive(Debug, PartialEq, Eq)]
struct Lws(Vec<u8>);
impl Parse for Lws {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: AsChar,
    {
        // LWS = [CRLF] 1*( SP | HT )
        let mut lws = vec![];

        if let Some(out) = opt("\r\n").parse_next(input)? {
            lws.extend_from_slice(out);
        }
        let out = space0.parse_next(input)?;
        lws.extend_from_slice(out);

        Ok(Lws(lws))
    }
}

#[cfg(test)]
mod test {
    use crate::winnow_parser::base::StreamParser;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_lws_no_crlf() -> Result<()> {
        let mut p = StreamParser::new(&b" \t abc"[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b" \t ".to_vec()));
        assert_eq!(p.complete_buffer(), b"abc");
        Ok(())
    }

    #[test]
    fn test_lws_crlf() -> Result<()> {
        let mut p = StreamParser::new(&b"\r\n\t abc"[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b"\r\n\t ".to_vec()));
        assert_eq!(p.complete_buffer(), b"abc");
        Ok(())
    }

    #[test]
    fn test_lws_empty() -> Result<()> {
        let mut p = StreamParser::new(&b""[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b"".to_vec()));
        assert_eq!(p.complete_buffer(), b"");
        Ok(())
    }

    #[test]
    fn test_lws_only_space() -> Result<()> {
        let mut p = StreamParser::new(&b"   "[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b"   ".to_vec()));
        assert_eq!(p.complete_buffer(), b"");
        Ok(())
    }

    #[test]
    fn test_lws_only_crlf() -> Result<()> {
        let mut p = StreamParser::new(&b"\r\n"[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b"\r\n".to_vec()));
        assert_eq!(p.complete_buffer(), b"");
        Ok(())
    }

    #[test]
    fn test_laws_only_cr() -> Result<()> {
        let mut p = StreamParser::new(&b"\r"[..]);
        let lws: Lws = p.parse()?;
        assert_eq!(lws, Lws(b"".to_vec()));
        assert_eq!(p.complete_buffer(), b"\r");
        Ok(())
    }
}
