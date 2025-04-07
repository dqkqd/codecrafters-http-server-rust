mod message;
mod protocol;
pub mod request;
mod response;
mod util;

pub trait Parse {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized;
}
