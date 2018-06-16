
mod test;

use std;
use std::io::Read;
use std::io::Bytes;

#[derive(Debug)]
pub enum ReadUnreaderError {
    EOF,
    Lower(std::io::Error),
}

pub struct ReadUnreader<R: Read> {
    lower: Bytes<R>,
    chars_read: Vec<u8>,
    chars_unread: Vec<u8>,

    pub pos: usize,
}

impl<R: Read> ReadUnreader<R> {
    pub fn new(read: R) -> ReadUnreader<R> {
        ReadUnreader {
            lower: read.bytes(),
            chars_read: Vec::new(),
            chars_unread: Vec::new(),
            pos: 0,
        }
    }

    fn read_lower(&mut self) -> Result<u8, ReadUnreaderError> {
        if let Some(c) = self.chars_unread.pop() {
            Ok(c)
        } else {
            match self.lower.next() {
                Some(Ok(c)) => {
                    Ok(c)
                }
                Some(Err(e)) => Err(ReadUnreaderError::Lower(e)),
                None => Err(ReadUnreaderError::EOF),
            }
        }
    }

    pub fn read(&mut self) -> Result<char, ReadUnreaderError> {
        self.pos += 1;
        self.read_lower().map(|c| {
            self.chars_read.push(c);
            std::char::from_u32(c as u32).unwrap()
        })
    }

    pub fn unread(&mut self) -> char {
        let c = self.chars_read.pop().unwrap_or_else(|| {
            panic!("unread: tried to unread when there is none to")
        });

        self.chars_unread.push(c);

        self.pos -= 1;

        std::char::from_u32(c as u32).unwrap()
    }

    pub fn peek(&mut self) -> Result<char, ReadUnreaderError> {
        let c = self.read()?;
        self.unread();
        Ok(c)
    }
}