use winnow::{ascii::space0, combinator::opt, stream::AsChar, Parser};

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
    use super::*;
    use crate::test_parse_ok;

    test_parse_ok!(lws_no_crlf, b" \t abc", Lws(b" \t ".to_vec()), b"abc");
    test_parse_ok!(lws_crlf, b"\r\n\t abc", Lws(b"\r\n\t ".to_vec()), b"abc");
    test_parse_ok!(lws_empty, b"", Lws(b"".to_vec()), b"");
    test_parse_ok!(lws_only_space, b"    ", Lws(b"    ".to_vec()), b"");
    test_parse_ok!(lws_only_crlf, b"\r\n", Lws(b"\r\n".to_vec()), b"");
    test_parse_ok!(lws_only_cr, b"\r", Lws(b"".to_vec()), b"\r");
}
