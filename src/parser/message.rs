use nom::{
    bytes::streaming::take_until1,
    character::{complete::char, streaming::space0},
    combinator::{map, opt, rest},
    multi::many0,
    sequence::{delimited, preceded, terminated},
    Parser,
};

use super::{
    util::{many_lws, until_space1},
    Parse,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct MessageHeader {
    pub field_name: FieldName,
    pub field_value: Option<FieldValue>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct FieldName(pub Vec<u8>);
#[derive(Debug, PartialEq, Eq)]
pub(super) struct FieldValue(pub Vec<FieldContent>);
#[derive(Debug, PartialEq, Eq)]
pub(super) struct FieldContent(pub Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct MessageBody(pub Vec<u8>);

impl Parse for FieldName {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let field_name = map(take_until1(":"), |field_name: &[u8]| {
            FieldName(field_name.trim_ascii_end().to_vec())
        });
        preceded(many_lws, field_name).parse(i)
    }
}

impl Parse for FieldContent {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        // The field-content does not include any leading or trailing LWS
        map(until_space1, |field_content| {
            FieldContent(field_content.to_vec())
        })
        .parse(i)
    }
}

impl Parse for FieldValue {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let field_contents = many0(terminated(FieldContent::parse, space0));
        let field_contents = map(opt(field_contents), |c| c.unwrap_or_default());
        map(delimited(space0, field_contents, space0), FieldValue).parse(i)
    }
}

impl Parse for MessageHeader {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let (input, (field_name, _, _, field_value)) =
            (FieldName::parse, many_lws, char(':'), FieldValue::parse).parse(i)?;

        let field_value = match !field_value.0.is_empty() {
            true => Some(field_value),
            false => None,
        };

        Ok((
            input,
            MessageHeader {
                field_name,
                field_value,
            },
        ))
    }
}

impl Parse for MessageBody {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        map(rest, |body: &[u8]| MessageBody(body.to_vec())).parse_complete(i)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test::TestParserStream;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_field_name() -> Result<()> {
        let mut p = TestParserStream::init(b"Content-Type: 3\r\n");
        let field_name: FieldName = p.parse()?;
        assert_eq!(field_name, FieldName(b"Content-Type".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_name_leading() -> Result<()> {
        let mut p = TestParserStream::init(b" Content-Type: 3\r\n");
        let field_name: FieldName = p.parse()?;
        assert_eq!(field_name, FieldName(b"Content-Type".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_name_trailing() -> Result<()> {
        let mut p = TestParserStream::init(b"Content-Type \t: 3\r\n");
        let field_name: FieldName = p.parse()?;
        assert_eq!(field_name, FieldName(b"Content-Type".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_content() -> Result<()> {
        let mut p = TestParserStream::init(b"ab\r\n");
        let field_content: FieldContent = p.parse()?;
        assert_eq!(field_content, FieldContent(b"ab".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_content_trailing() -> Result<()> {
        let mut p = TestParserStream::init(b"ab ");
        let field_content: FieldContent = p.parse()?;
        assert_eq!(field_content, FieldContent(b"ab".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_value() -> Result<()> {
        let mut p = TestParserStream::init(b"ab  \tcd\r\n");
        let field_value: FieldValue = p.parse()?;
        assert_eq!(
            field_value,
            FieldValue(vec![
                FieldContent(b"ab".to_vec()),
                FieldContent(b"cd".to_vec())
            ])
        );
        Ok(())
    }

    #[test]
    fn test_parse_field_value_leading() -> Result<()> {
        let mut p = TestParserStream::init(b"  ab  \tcd\r\n");
        let field_value: FieldValue = p.parse()?;
        assert_eq!(
            field_value,
            FieldValue(vec![
                FieldContent(b"ab".to_vec()),
                FieldContent(b"cd".to_vec())
            ])
        );
        Ok(())
    }

    #[test]
    fn test_parse_field_value_trailing() -> Result<()> {
        let mut p = TestParserStream::init(b"  ab  \tcd   \r\n");
        let field_value: FieldValue = p.parse()?;
        assert_eq!(
            field_value,
            FieldValue(vec![
                FieldContent(b"ab".to_vec()),
                FieldContent(b"cd".to_vec())
            ])
        );
        Ok(())
    }

    #[test]
    fn test_parse_field_value_empty() -> Result<()> {
        let mut p = TestParserStream::init(b"\r\n");
        let field_value: FieldValue = p.parse()?;
        assert_eq!(field_value, FieldValue(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_field_value_only_space() -> Result<()> {
        let mut p = TestParserStream::init(b"  \r\n");
        let field_value: FieldValue = p.parse()?;
        assert_eq!(field_value, FieldValue(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_field_value_only_crlf() -> Result<()> {
        let (_, field_value) = FieldValue::parse(b"  \r\n")?;
        assert_eq!(field_value, FieldValue(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_message_header() -> Result<()> {
        let mut p = TestParserStream::init(b"Content-Length: 3 4 5\r\n");
        let message_header: MessageHeader = p.parse()?;
        assert_eq!(
            message_header,
            MessageHeader {
                field_name: FieldName(b"Content-Length".to_vec()),
                field_value: Some(FieldValue(vec![
                    FieldContent(b"3".to_vec()),
                    FieldContent(b"4".to_vec()),
                    FieldContent(b"5".to_vec()),
                ]))
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_message_header_leading() -> Result<()> {
        let mut p = TestParserStream::init(b"  Content-Length: 3 4 5\r\n");
        let message_header: MessageHeader = p.parse()?;
        assert_eq!(
            message_header,
            MessageHeader {
                field_name: FieldName(b"Content-Length".to_vec()),
                field_value: Some(FieldValue(vec![
                    FieldContent(b"3".to_vec()),
                    FieldContent(b"4".to_vec()),
                    FieldContent(b"5".to_vec()),
                ]))
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_message_header_empty_value() -> Result<()> {
        let mut p = TestParserStream::init(b"Content-Length: \r\n");
        let message_header: MessageHeader = p.parse()?;
        assert_eq!(
            message_header,
            MessageHeader {
                field_name: FieldName(b"Content-Length".to_vec()),
                field_value: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_message_header_empty() -> Result<()> {
        let mut p = TestParserStream::init(b"\r\n");
        assert!(p.parse::<MessageHeader>().is_err());
        Ok(())
    }

    #[test]
    fn test_parse_body() -> Result<()> {
        let mut p = TestParserStream::init(b"one two three");
        let body: MessageBody = p.parse()?;
        assert_eq!(body, MessageBody(b"one two three".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_body_empty() -> Result<()> {
        let mut p = TestParserStream::init(b"");
        let body: MessageBody = p.parse()?;
        assert_eq!(body, MessageBody(b"".to_vec()));
        Ok(())
    }
}
