
use super::*;

use std::io::prelude::*;

// General byte output device. ignores address. returns 0 on read
pub struct OutputDevice<W: Write> {
    pub writer: W,
}

impl<W: Write> OutputDevice<W> {
    pub fn new(writer: W) -> OutputDevice<W> {
        OutputDevice {
            writer: writer,
        }
    }
}

impl<W: Write> BusEnd for OutputDevice<W> {
    fn read_word(&mut self, addr: u32) -> u32 {
        0
    }
    fn write_word(&mut self, addr: u32, value: u32) {
        let buf = &[value as u8];
        self.writer.write(buf).unwrap();
    }
    fn is_interrupting(&self) -> bool {
        false
    }
}

impl<W: Write> Peri for OutputDevice<W> {}

pub struct InputDevice<R: Read> {
    reader: R,
}

impl<R: Read> InputDevice<R> {
    pub fn new(reader: R) -> InputDevice<R> {
        InputDevice {
            reader: reader,
        }
    }
}

impl<R: Read> BusEnd for InputDevice<R> {
    // Reading pops from the input queue
    fn read_word(&mut self, addr: u32) -> u32 {
        let mut buf = [0u8; 1];
        match self.reader.read(&mut buf[..]) {
            Ok(read) => {
                //TODO EOF should gracefully terminate
                if read == 0 {
                    panic!("InputDevice: EOF");
                }

                return buf[0] as u32;
            }
            Err(e) => panic!("InputDevice read error: {:?}", e),
        }
    }

    fn write_word(&mut self, addr: u32, value: u32) {
        // ignoring...
    }

    // interrupt is not used for this synchronuous architecture
    fn is_interrupting(&self) -> bool {
        false
    }
}

impl<R: Read> Peri for InputDevice<R> {}
