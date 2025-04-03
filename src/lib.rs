// mod model;
mod parser;
pub mod request;
pub mod response;
pub mod routes;

use anyhow::Result;
use std::io::BufRead;

fn read_until_crlf<R: BufRead>(reader: &mut R, buf: &mut Vec<u8>) -> Result<()> {
    loop {
        // read \r
        let _ = reader.read_until(b'\r', buf)?;
        // read \n
        let n = reader.read_until(b'\n', buf)?;
        if n == 1 || n == 0 {
            break;
        }
    }

    Ok(())
}
