
use std;
use std::io::prelude::*;

// Shouldn't something like this be in stdlib?
pub struct MemReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> MemReader<'a> {
    pub fn new(bytes: &'a [u8]) -> MemReader {
        MemReader{
            bytes: bytes,
            pos: 0,
        }
    }

    pub fn from(s: &'a str) -> MemReader {
        MemReader{
            bytes: s.as_bytes(),
            pos: 0,
        }
    }
}

impl<'a> Read for MemReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_len = self.bytes.len();

        assert!(self.pos <= bytes_len);

        // check if eof
        if self.pos == bytes_len {
            return Ok(0);
        }

        let next = &self.bytes[self.pos..std::cmp::min(bytes_len, self.pos + buf.len())];

        for (i, v) in next.iter().enumerate() {
            buf[i] = *v;
        }

        let read = next.len();
        self.pos += read;

        Ok(read)
    }
}


#[test]
fn test_mem_reader() {
    let test_string = "bananas";

    let mut reader = MemReader::new(test_string.as_bytes());

    let mut buf = [0 as u8; 4];

    match reader.read(&mut buf) {
        Ok(read) => {
            if read != 4 {
                panic!("read {}?!", read);
            }

            if &buf != &[0x62, 0x61, 0x6E, 0x61] {
                panic!("read this?!: {:?}", &buf);
            }
        }
        Err(e) => {
            panic!("read returned error: {}", e);
        }
    }

    match reader.read(&mut buf) {
        Ok(read) => {
            if read != 3 {
                panic!("read {}?!", read);
            }

            if &buf[0..3] != &[0x6E, 0x61, 0x73] {
                panic!("read this?!: {:?}", &buf);
            }
        }
        Err(e) => {
            panic!("read returned error: {}", e);
        }
    }

    match reader.read(&mut buf) {
        Ok(read) => {
            if read != 0 {
                panic!("read {}?!", read);
            }
        }
        Err(e) => {
            panic!("read returned error: {}", e);
        }
    }
}