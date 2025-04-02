use anyhow::Result;
use std::io::Write;

pub fn response_404<W: Write>(w: &mut W) -> Result<()> {
    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
    w.write_all(response.as_bytes())?;
    Ok(())
}

pub fn response_200<W: Write>(w: &mut W) -> Result<()> {
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    w.write_all(response.as_bytes())?;
    Ok(())
}
