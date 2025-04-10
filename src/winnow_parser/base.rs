use std::io::{BufReader, Read};

use anyhow::{bail, Result};
use winnow::{
    error::ErrMode,
    stream::{AsChar, Compare, FindSlice, Offset, Stream, StreamIsPartial},
    ModalResult, Partial,
};

pub(crate) trait Convertible<'i>:
    Stream<Slice = &'i [u8]> + Compare<&'static str> + Compare<char> + StreamIsPartial + FindSlice<char>
where
    Self: std::marker::Sized,
{
}

impl<'i, I> Convertible<'i> for I
where
    I: Stream<Slice = &'i [u8]>,
    I: Compare<&'static str>,
    I: Compare<char>,
    I: StreamIsPartial,
    I: FindSlice<char>,
    I::Token: AsChar,
{
}

pub(crate) trait Parse {
    fn parse<'i, I>(input: &mut I) -> ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: Convertible<'i>,
        I::Token: AsChar;
}

#[derive(Debug)]
pub(crate) struct StreamParser<R: Read> {
    reader: BufReader<R>,
    pub buffer: Vec<u8>,
}

impl<R: Read> StreamParser<R> {
    pub fn new(stream: R) -> StreamParser<R> {
        StreamParser {
            reader: BufReader::new(stream),
            buffer: vec![],
        }
    }

    pub fn parse<T>(&mut self) -> Result<T>
    where
        T: Parse + std::fmt::Debug,
    {
        let mut buffer = [0; 4096];
        loop {
            let mut partial = Partial::new(self.buffer.as_slice());
            let start = partial.checkpoint();
            match T::parse(&mut partial) {
                Ok(out) => {
                    let consumed = partial.offset_from(&start);
                    self.buffer = self.buffer.split_off(consumed);
                    break Ok(out);
                }

                Err(ErrMode::Incomplete(_)) => {
                    let n = self.reader.read(&mut buffer)?;
                    if n == 0 {
                        break self.parse_complete();
                    }
                    self.buffer.extend_from_slice(&buffer[..n]);
                }
                Err(e) => todo!("{}", e),
            }
        }
    }

    pub fn parse_complete<T>(&mut self) -> Result<T>
    where
        T: Parse + std::fmt::Debug,
    {
        let mut buffer = self.buffer.as_slice();
        let start = buffer.checkpoint();
        match T::parse(&mut buffer) {
            Ok(out) => {
                let consumed = buffer.offset_from(&start);
                self.buffer = self.buffer.split_off(consumed);
                Ok(out)
            }
            Err(e) => bail!("unexpected eof, `{}`", e),
        }
    }
}

mod test {
    use super::*;

    impl<R: Read> StreamParser<R> {
        pub fn complete_buffer(&mut self) -> &[u8] {
            let mut buffer = vec![];
            self.reader.read_to_end(&mut buffer).unwrap();
            self.buffer.extend_from_slice(&buffer);
            &self.buffer
        }
    }
}
