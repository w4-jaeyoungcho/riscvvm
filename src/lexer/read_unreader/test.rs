#![cfg(test)]

use super::*;
use ::lexer::mem_reader::*;

#[test]
fn test_read_unreader() {
    let mut r = ReadUnreader::new(MemReader::new("ban".as_bytes()));

    {
        let c = r.read().unwrap();
        assert_eq!(c, 'b');

        let c = r.read().unwrap();
        assert_eq!(c, 'a');

        r.unread();
        let c= r.read().unwrap();
        assert_eq!(c, 'a');

        r.read().unwrap();

        match r.read() {
            Err(ReadUnreaderError::EOF) => (), // ok
            other => panic!("eh: {:?}", other),
        }

    }
}