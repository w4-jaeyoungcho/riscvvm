// Assembled machine

#[cfg(test)]
mod test;

use std::io::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use ::machine::*;
use ::machine::peri::*;
use ::machine::memory::*;


pub const SYSTEM_HEADER: &'static str = "\
; peripherals
(equ output 0x00100000)
(equ input 0x00100004)
(equ end_pc_target 0x10000000)
(equ mtvec 0x00000010)
";

pub fn vpc<W: Write + 'static>(bin: &[u8], writer: W) -> (Machine, Rc<RefCell<OutputDevice<W>>>, Rc<RefCell<InputDevice>>) {

    // Machine
    const MEMORY_START: u32 = 0;
    const MEMORY_WIDTH: u8 =16;
    const MEMORY_PROBE: u32 = 0x1000;

    const OUTPUT_START: u32 = 0x100000;
    const INPUT_START: u32 = 0x100004;

    let mut memory = Memory::new(Some(MEMORY_PROBE));
    let output_device = Rc::new(RefCell::new(OutputDevice::new(writer)));
    let input_device = Rc::new(RefCell::new(InputDevice::new()));

    let mut m = Machine::new();
    memory.load(&bin[..]);
    m.attach("memory", memory, MEMORY_START, MEMORY_WIDTH);
    m.attach("output", output_device.clone(), OUTPUT_START, 0);
    m.attach("input", input_device.clone(), INPUT_START, 0);

    (m, output_device, input_device)
}
