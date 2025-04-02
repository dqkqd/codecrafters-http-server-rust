use anyhow::{bail, Result};
use std::io::BufRead;
use std::str;

use crate::read_until_crlf;

#[derive(Debug)]
pub enum RequestMethod {
    Get,
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: RequestMethod,
    pub target: String,
    version: String,
}

impl RequestLine {
    pub fn parse<R: BufRead>(reader: &mut R) -> Result<RequestLine> {
        let mut buf = Vec::new();
        read_until_crlf(reader, &mut buf)?;

        let whiespace = b" \t\r\n";
        let mut chunks = buf
            .split(|c| whiespace.contains(c))
            .map(|chunk| chunk.trim_ascii())
            .filter(|chunk| !chunk.is_empty());

        let method = chunks.next().unwrap_or_default();
        let method = str::from_utf8(method)?;
        let method = match method.to_lowercase().as_str() {
            "get" => RequestMethod::Get,
            _ => bail!("invalid method"),
        };

        let target = chunks.next().unwrap_or_default();
        let target = String::from_utf8(target.to_vec())?;

        let version = chunks.next().unwrap_or_default();
        let version = String::from_utf8(version.to_vec())?;

        Ok(RequestLine {
            method,
            target,
            version,
        })
    }
}
