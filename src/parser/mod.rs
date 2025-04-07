use std::io::{BufReader, Read};

mod message;
mod protocol;
pub mod request;
mod response;
mod util;
use anyhow::{bail, Result};

pub trait Parse {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: std::marker::Sized;
}

struct ParserStream<R>
where
    R: Read,
{
    input: BufReader<R>,
    buffer: Vec<u8>,
}

impl<R> ParserStream<R>
where
    R: Read,
{
    fn new(input: BufReader<R>) -> ParserStream<R> {
        ParserStream {
            input,
            buffer: Vec::new(),
        }
    }

    fn parse<P>(&mut self) -> Result<P>
    where
        P: Parse,
    {
        loop {
            match P::parse(&self.buffer) {
                Ok((buffer, out)) => {
                    let n = buffer.len();
                    self.buffer = self.buffer.split_off(n);
                    break Ok(out);
                }
                Err(nom::Err::Incomplete(nom::Needed::Unknown)) => {
                    let mut buffer = [0; 4096];
                    let n = self.input.read(&mut buffer)?;
                    if n == 0 {
                        bail!("unexpected eof")
                    }
                    self.buffer.extend_from_slice(&buffer[..n]);
                }
                Err(nom::Err::Incomplete(nom::Needed::Size(n))) => {
                    let from = self.buffer.len();
                    let to = from + n.get();
                    self.buffer.resize(to, 0);
                    self.input.read_exact(&mut self.buffer[from..to])?
                }
                Err(e) => {
                    dbg!(&self.buffer);
                    dbg!(e);
                    todo!()
                }
            }
        }
    }

    fn fill_all(&mut self) -> Result<()> {
        let mut buffer = [0; 4096];
        loop {
            let n = self.input.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            self.buffer.extend_from_slice(&buffer[..n]);
        }
        Ok(())
    }

    fn parse_complete<P>(&mut self) -> Result<P>
    where
        P: Parse,
    {
        match P::parse(&self.buffer) {
            Ok((buffer, out)) => {
                let n = buffer.len();
                self.buffer = self.buffer.split_off(n);
                Ok(out)
            }
            Err(e) => {
                dbg!(&self.buffer);
                dbg!(e);
                todo!()
            }
        }
    }
}

mod test {
    use super::*;

    pub(super) type TestParserStream<'a> = ParserStream<&'a [u8]>;
    impl<'a> TestParserStream<'a> {
        #[allow(dead_code)]
        pub fn init(bytes: &'a [u8]) -> TestParserStream<'a> {
            let input = BufReader::new(bytes);
            TestParserStream::new(input)
        }
    }
}
