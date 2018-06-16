
use std;
use std::io::prelude::*;
use ::decode::*;

mod test;

use ::arch::inst;

#[derive(Debug)]
pub enum DisasmError {
    Unknown,
    Read(std::io::Error),
    Write(std::io::Error),
}

/*
listing

<counter> : <hex> <binary> <mnemonic>



*/

// I hope reader and writers are buffered, because I don't
pub fn disassemble<W: Write, R: Read>(mut writer: W, mut reader: R) -> Result<(), DisasmError> {
    // Let's decode and detranslate!

    let mut counter: u32 = 0;

    let mut buf = [0u8; 4];

    // Read until EOF
    while reader.read(&mut buf).map_err(|e| DisasmError::Read(e))? == 4 {
        let x = array_to_u32(&buf);

        write!(&mut writer, "{:08X} : {:08X} - {:08b} {:08b} {:08b} {:08b}    ; ", counter, x, buf[3], buf[2], buf[1], buf[0]);

        match decode(x) {
            Some((name, inst, args)) => {
                // format and write
                write!(&mut writer, "({}", name);
                for i in 0..inst.args.len() {
                    let arg = inst.args[i];
                    let a = args[i];

                    write!(&mut writer, " {}", inst::format_arg(arg, a));
                }
                writeln!(&mut writer, ")");
            }
            None => {
                // Just print its value in hexadecimal
                writeln!(&mut writer, "0x{:08X}", x);
            }
        };

        counter += 4;
    }

    Ok(())
}

fn array_to_u32(buf: &[u8; 4]) -> u32 {
    let mut x = 0u32;
    x |= buf[3] as u32;
    x <<= 8;
    x |= buf[2] as u32;
    x <<= 8;
    x |= buf[1] as u32;
    x <<= 8;
    x |= buf[0] as u32;

    x
}