mod message;
mod protocol;
mod request;
mod response;
mod util;

use nom::IResult;

use request::RequestLine;
use response::StatusLine;

trait Parse {
    fn parse(i: &[u8]) -> IResult<&[u8], Self>
    where
        Self: std::marker::Sized;
}

#[derive(Debug)]
struct Request {}
#[derive(Debug)]
struct Response {}

#[derive(Debug)]
enum StartLine {
    RequestLine(RequestLine),
    StatusLine(StatusLine),
}
