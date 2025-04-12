use std::{
    io::{BufReader, ErrorKind, Read},
    thread,
    time::Duration,
};

use anyhow::{bail, Result};
use winnow::{
    ascii::Caseless,
    error::ErrMode,
    stream::{AsChar, Compare, FindSlice, Offset, Stream, StreamIsPartial},
    ModalResult, Partial,
};

pub trait Convertible<'i>:
    Stream<Slice = &'i [u8]>
    + Compare<&'static str>
    + Compare<Caseless<&'static str>>
    + Compare<char>
    + StreamIsPartial
    + FindSlice<char>
where
    Self: std::marker::Sized,
{
}

impl<'i, I> Convertible<'i> for I
where
    I: Stream<Slice = &'i [u8]>,
    I: Compare<&'static str>,
    I: Compare<Caseless<&'static str>>,
    I: Compare<char>,
    I: StreamIsPartial,
    I: FindSlice<char>,
    I::Token: AsChar,
{
}

pub trait Parse {
    fn parse<'i, I>(input: &mut I) -> ModalResult<Self>
    where
        Self: std::marker::Sized,
        I: Convertible<'i>,
        I::Token: AsChar;

    fn convert(b: &str) -> Result<Self>
    where
        Self: std::marker::Sized,
        Self: std::fmt::Debug,
    {
        StreamParser::new(b.as_bytes()).parse()
    }
}

#[derive(Debug)]
pub struct StreamParser<R: Read> {
    reader: BufReader<R>,
    pub buffer: Vec<u8>,
}

impl<R: Read> StreamParser<R> {
    pub fn new(reader: R) -> StreamParser<R> {
        StreamParser {
            reader: BufReader::new(reader),
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
                    let n = match self.reader.read(&mut buffer) {
                        Ok(n) => n,
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(100));
                            continue;
                        }
                        Err(e) => {
                            dbg!(e);
                            0
                        }
                    };
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

    #[cfg(test)]
    pub fn complete_buffer(&mut self) -> &[u8] {
        let mut buffer = vec![];
        self.reader.read_to_end(&mut buffer).unwrap();
        self.buffer.extend_from_slice(&buffer);
        &self.buffer
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use anyhow::Result;

    pub fn case_ok<P>(input: &[u8], expected_result: P, remaining: &[u8]) -> Result<()>
    where
        P: std::fmt::Debug,
        P: Parse,
        P: PartialEq,
    {
        let mut p = StreamParser::new(input);
        let result: P = p.parse()?;
        assert_eq!(result, expected_result);
        assert_eq!(p.complete_buffer(), remaining);
        Ok(())
    }

    pub fn case_error<P>(input: &[u8], remaining: &[u8]) -> Result<()>
    where
        P: std::fmt::Debug,
        P: Parse,
        P: PartialEq,
    {
        let mut p = StreamParser::new(input);
        assert!(p.parse::<P>().is_err());
        assert_eq!(p.complete_buffer(), remaining);
        Ok(())
    }

    #[macro_export]
    macro_rules! test_parse_ok {
        ($func_name:ident, $input:literal, $expected_result:expr, $remaining:literal) => {
            #[test]
            fn $func_name() -> anyhow::Result<()> {
                $crate::parser::base::test::case_ok($input, $expected_result, $remaining)
            }
        };
    }

    #[macro_export]
    macro_rules! test_parse_error {
        ($func_name:ident, $ty:ty, $input:literal, $remaining:literal) => {
            #[test]
            fn $func_name() -> anyhow::Result<()> {
                $crate::parser::base::test::case_error::<$ty>($input, $remaining)
            }
        };
    }
}
