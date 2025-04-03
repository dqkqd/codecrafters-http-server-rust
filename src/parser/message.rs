use nom::{
    bytes::complete::take_until1,
    character::complete::{char, multispace0},
    combinator::map,
    multi::many0,
    sequence::preceded,
    Parser,
};

use super::{util::until_space1, Parse, Request, Response};

#[derive(Debug)]
enum HttpMessage {
    Request(Request),
    Response(Response),
}

#[derive(Debug, PartialEq, Eq)]
struct MessageHeader {
    field_name: FieldName,
    field_value: Option<FieldValue>,
}
#[derive(Debug, PartialEq, Eq)]
struct FieldName(Vec<u8>);
#[derive(Debug, PartialEq, Eq)]
struct FieldValue(Vec<FieldContent>);
#[derive(Debug, PartialEq, Eq)]
struct FieldContent(Vec<u8>);

impl Parse for FieldName {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let field_name = map(take_until1(":"), |field_name: &[u8]| {
            FieldName(field_name.trim_ascii_end().to_vec())
        });
        preceded(multispace0, field_name).parse(i)
    }
}

impl Parse for FieldContent {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let field_content = map(until_space1, |field_content| {
            FieldContent(field_content.to_vec())
        });
        preceded(multispace0, field_content).parse(i)
    }
}

impl Parse for FieldValue {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        map(
            many0(FieldContent::parse),
            |field_contents: Vec<FieldContent>| FieldValue(field_contents),
        )
        .parse(i)
    }
}

impl Parse for MessageHeader {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized,
    {
        let (input, (field_name, _, field_value)) =
            (FieldName::parse, char(':'), FieldValue::parse).parse(i)?;

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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_field_name() -> Result<()> {
        let (_, field_name) = FieldName::parse(
            b"
Content-Type: 3\r\n",
        )?;
        assert_eq!(field_name, FieldName(b"Content-Type".to_vec()));
        Ok(())
    }

    #[test]
    fn test_parse_field_content() -> Result<()> {
        let (input, field_content) = FieldContent::parse(
            b"
ab  \tcd\r\n
",
        )?;
        assert_eq!(field_content, FieldContent(b"ab".to_vec()));

        let (input, field_content) = FieldContent::parse(input)?;
        assert_eq!(field_content, FieldContent(b"cd".to_vec()));

        assert!(FieldContent::parse(input).is_err());
        Ok(())
    }

    #[test]
    fn test_parse_field_value() -> Result<()> {
        let (input, field_value) = FieldValue::parse(
            b"
ab  \tcd\r\n
",
        )?;
        assert_eq!(
            field_value,
            FieldValue(vec![
                FieldContent(b"ab".to_vec()),
                FieldContent(b"cd".to_vec())
            ])
        );

        let (_, field_value) = FieldValue::parse(input)?;
        assert_eq!(field_value, FieldValue(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_message_header() -> Result<()> {
        let (_, message_header) = MessageHeader::parse(
            b"
Content-Length: 3 4 5
",
        )?;

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
    fn test_parse_message_empty_value() -> Result<()> {
        let (_, message_header) = MessageHeader::parse(
            b"
Content-Length:
",
        )?;

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
    fn test_parse_message_has_space_in_field_name() -> Result<()> {
        let (_, message_header) = MessageHeader::parse(
            b"
Content-Length : 3 4 5
",
        )?;

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
}
