use core::str;

use anyhow::Result;
use nom::{bytes::complete::is_not, IResult};

pub(super) fn vec_u8_to_u32(i: &[u8]) -> Result<u32> {
    let v = str::from_utf8(i)?.parse()?;
    Ok(v)
}

pub(super) fn until_space1(i: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not(" \t\r\n")(i)
}
