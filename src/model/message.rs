use anyhow::Result;

use crate::model::read_until_crlf;

use super::{header::MessageHeader, request::RequestLine};
use super::{Converter, CRLF};

#[derive(Debug)]
struct Message {
    start_line: StartLine,
    header: MessageHeader,
    body: Option<MessageBody>,
}

#[derive(Debug)]
struct MessageBody {
    data: String,
}

#[derive(Debug)]
enum StartLine {
    RequestLine(RequestLine),
    StatusLine(StatusLine),
}

#[derive(Debug)]
struct StatusLine {}

// impl Display for Message {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let body = self
//             .body
//             .as_ref()
//             .map(|b| b.data.as_str())
//             .unwrap_or_default();
//         write!(f, "{}{}{CRLF}{}", self.start_line, self.header, body)
//     }
// }
//
// impl Display for StartLine {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             StartLine::RequestLine(request_line) => write!(f, "{}", request_line),
//             StartLine::ResponseLine(response_line) => unimplemented!(),
//         }
//     }
// }
impl Converter for StartLine {
    fn from_reader<R: std::io::BufRead>(reader: &mut R) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let start_line = read_until_crlf(reader)?.unwrap_or_default();
        todo!()
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_from_string() {}
}
