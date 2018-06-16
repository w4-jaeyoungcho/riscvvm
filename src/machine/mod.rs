//! RISC-V Virtual Machine functionality

// Implements run-loop

pub mod peri;
mod cpu;
pub mod memory;
#[cfg(test)]
mod test;

use std::io::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;

use self::cpu::*;
use self::memory::*;

use ::arch::system::*;

// I don't know. Something unmistakable.
const TERMINATION_PC: u32 = 0x10000000;

// Machine is CPU and the bus
pub struct Machine {
    cpu: Cpu,
    peripherals: HashMap<String, PeriConnection>,
}

#[derive(Debug)]
pub enum RunError {
    CyclesLimitExceeded(u32), // last pc
    Exception(u32),
    Terminated,
}

impl Machine {
    /// Configure and return new machine
    pub fn new() -> Machine {
        Machine {
            cpu: Cpu::new(),
            peripherals: HashMap::new(),
        }
    }

    pub fn attach<T: Peri + 'static>(&mut self, name: &str, peri: T, addr_start: u32, addr_width: u8) {
        let peripheral = PeriConnection {
            addr_start: addr_start,
            addr_width: addr_width,
            device: Box::new(peri),
        };
        self.peripherals.insert(String::from(name), peripheral);
    }

    pub fn tick(&mut self) {
        // Tick peripherals
        let mut interrupting = false;
        for (n, c) in &mut self.peripherals {
            c.device.tick();
            interrupting = interrupting || c.device.is_interrupting();
        }

        // Set MEIP
        self.cpu.ip = set_patch(self.cpu.ip, MEIP, 1, if interrupting { 1 } else { 0 });

        // Tick Cpu
        self.cpu.tick(&mut self.peripherals);
    }

    // Run until WFI
    // Returns cycles run
    // Need to catch runaway. num_tick_limit = 0 means no limit
    pub fn run(&mut self, num_tick_limit: u32, die_on_exception: bool) -> Result<u32, RunError> {
        let mut cycles = 0;

        self.cpu.die_on_exception = die_on_exception;

        loop {
            if self.cpu.pc == TERMINATION_PC {
                return Err(RunError::Terminated);
            }

            if num_tick_limit != 0 && cycles >= num_tick_limit {
                return Err(RunError::CyclesLimitExceeded(self.cpu.pc));
            }

            if die_on_exception && self.cpu.cause != 0 {
                return Err(RunError::Exception(self.cpu.pc));
            }

            self.tick();

            cycles += 1;

            if self.cpu.wfi {
                break;
            }
        }

        Ok(cycles)
    }
}

impl<'a> MasterBusEnd for HashMap<String, PeriConnection> {
    fn read_word(&mut self, addr: u32) -> Result<u32, ()> {
        if let Some((name, peri)) = bus_select_mut(self, addr) {
            Ok(peri.device.read_word(addr))
        } else {
            Err(())
        }
    }

    fn write_word(&mut self, addr: u32, value: u32) -> Result<(), ()> {
        if let Some((name, peri)) = bus_select_mut(self, addr) {
            Ok(peri.device.write_word(addr, value))
        } else {
            Err(())
        }
    }

    fn is_interrupting(&self) -> bool {
        // if any of the peripherals is interrupting, it is interrupting
        for (_, p) in self {
            if p.device.is_interrupting() {
                return true;
            }
        }

        false
    }
}

fn bus_select<'a, 'b>(peris: &'a HashMap<String, PeriConnection>, addr: u32) -> Option<(&'a str, &'a PeriConnection)> {
    for (n, p) in peris {
        if p.addr_start == addr & !((1u32 << p.addr_width) - 1) {
            return Some((n, p));
        }
    }
    None
}

fn bus_select_mut<'a, 'b>(peris: &'a mut HashMap<String, PeriConnection>, addr: u32) -> Option<(&'a str, &'a mut PeriConnection)> {
    for (n, p) in peris {
        if p.addr_start == addr & !((1u32 << p.addr_width) - 1) {
            return Some((n, p));
        }
    }
    None
}

// Peripheral registration
struct PeriConnection {
    addr_start: u32,
    addr_width: u8,
    device: Box<Peri>,
}

pub trait BusEnd {
    // addr is word aligned

    fn read_word(&mut self, addr: u32) -> u32;
    fn write_word(&mut self, addr: u32, value: u32);
    fn is_interrupting(&self) -> bool;
}

// Exception can be raised
trait MasterBusEnd {
    // addr is assumed to be word aligned

    fn read_word(&mut self, addr: u32) -> Result<u32, ()>;
    fn write_word(&mut self, addr: u32, value: u32) -> Result<(), ()>;
    fn is_interrupting(&self) -> bool;
}

pub trait Peri: BusEnd {
    fn tick(&mut self) {
        // nop default
    }
}

fn set_patch(base: u32, offset: u8, length: u8, value: u32) -> u32 {
    let mask = ((1u32 << length) - 1) << offset;
    (base & !mask) | (value << offset)
}