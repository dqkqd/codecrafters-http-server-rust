use winnow::{
    ascii::{space0, space1},
    combinator::{alt, preceded, separated, seq},
    stream::AsChar,
    token::{rest, take_till, take_until, take_while},
    Parser,
};

use crate::{
    parser::util::is_space,
    spec::message::{FieldContent, FieldName, FieldValue, MessageBody, MessageHeader},
};

use super::base::Parse;

impl Parse for FieldName {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let field_name = preceded(
            space0,
            take_until(.., ':')
                .map(|field_name: &[u8]| FieldName(field_name.trim_ascii_end().to_vec())),
        )
        .parse_next(input)?;

        Ok(field_name)
    }
}

impl Parse for FieldContent {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: winnow::stream::AsChar,
    {
        let field_content = alt((take_till(0.., is_space), rest))
            .map(|field_content: &[u8]| FieldContent(field_content.to_vec()))
            .parse_next(input)?;
        Ok(field_content)
    }
}

impl Parse for FieldValue {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: AsChar,
    {
        let field_contents = separated(0.., FieldContent::parse, space1);
        let field_value = preceded(space0, field_contents)
            .map(|contents: Vec<FieldContent>| {
                contents.into_iter().filter(|c| !c.0.is_empty()).collect()
            })
            .map(FieldValue)
            .parse_next(input)?;

        Ok(field_value)
    }
}

impl Parse for MessageHeader {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: AsChar,
    {
        let message_header = seq! {
            MessageHeader {
                field_name: FieldName::parse,
                _: ':',
                field_value: FieldValue::parse.map(|field_value| {
                    match !field_value.0.is_empty() {
                        true => Some(field_value),
                        false => None,
                    }
                })

            }
        }
        .parse_next(input)?;

        Ok(message_header)
    }
}

impl Parse for MessageBody {
    fn parse<'i, I>(input: &mut I) -> winnow::ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: super::base::Convertible<'i>,
        I::Token: AsChar,
    {
        let body = take_while(0.., |_| true)
            .map(|body: &[u8]| MessageBody(body.to_vec()))
            .parse_next(input)?;
        Ok(body)
    }
}

#[cfg(test)]
mod test {
    use crate::{test_parse_error, test_parse_ok};

    use super::*;

    test_parse_ok!(
        field_name,
        b"Content-Type: 3\r\n",
        FieldName(b"Content-Type".to_vec()),
        b": 3\r\n"
    );
    test_parse_ok!(
        field_name_leading,
        b" Content-Type: 3\r\n",
        FieldName(b"Content-Type".to_vec()),
        b": 3\r\n"
    );
    test_parse_ok!(
        field_name_trailing,
        b"Content-Type \t: 3\r\n",
        FieldName(b"Content-Type".to_vec()),
        b": 3\r\n"
    );

    test_parse_ok!(field_content, b"ab", FieldContent(b"ab".to_vec()), b"");
    test_parse_ok!(
        field_content_trailing,
        b"ab ",
        FieldContent(b"ab".to_vec()),
        b" "
    );

    test_parse_ok!(
        field_value,
        b"ab  \tcd",
        FieldValue(vec![
            FieldContent(b"ab".to_vec()),
            FieldContent(b"cd".to_vec())
        ]),
        b""
    );
    test_parse_ok!(
        field_value_crlf,
        b"ab  \tcd\r\n",
        FieldValue(vec![
            FieldContent(b"ab".to_vec()),
            FieldContent(b"cd".to_vec())
        ]),
        b"\r\n"
    );
    test_parse_ok!(
        field_value_leading,
        b" \tab  \tcd",
        FieldValue(vec![
            FieldContent(b"ab".to_vec()),
            FieldContent(b"cd".to_vec())
        ]),
        b""
    );
    test_parse_ok!(
        field_value_trailing,
        b" \tab  \tcd   \r\n",
        FieldValue(vec![
            FieldContent(b"ab".to_vec()),
            FieldContent(b"cd".to_vec())
        ]),
        b"\r\n"
    );
    test_parse_ok!(field_value_empty, b"", FieldValue(vec![]), b"");
    test_parse_ok!(field_value_only_space, b"   \t", FieldValue(vec![]), b"");
    test_parse_ok!(field_value_only_crlf, b" \r\n", FieldValue(vec![]), b"\r\n");

    test_parse_ok!(
        message_header,
        b"Content-Length: 3 4 5\r\n",
        MessageHeader {
            field_name: FieldName(b"Content-Length".to_vec()),
            field_value: Some(FieldValue(vec![
                FieldContent(b"3".to_vec()),
                FieldContent(b"4".to_vec()),
                FieldContent(b"5".to_vec()),
            ]))
        },
        b"\r\n"
    );
    test_parse_ok!(
        message_header_leading,
        b"   Content-Length: 3 4 5\r\n",
        MessageHeader {
            field_name: FieldName(b"Content-Length".to_vec()),
            field_value: Some(FieldValue(vec![
                FieldContent(b"3".to_vec()),
                FieldContent(b"4".to_vec()),
                FieldContent(b"5".to_vec()),
            ]))
        },
        b"\r\n"
    );
    test_parse_ok!(
        message_header_empty_value,
        b"Content-Length: \r\n",
        MessageHeader {
            field_name: FieldName(b"Content-Length".to_vec()),
            field_value: None,
        },
        b"\r\n"
    );
    test_parse_error!(message_header_empty_header, MessageHeader, b"\r\n", b"\r\n");

    test_parse_ok!(
        body,
        b"one two three",
        MessageBody(b"one two three".to_vec()),
        b""
    );
    test_parse_ok!(body_empty, b"", MessageBody(b"".to_vec()), b"");
}
