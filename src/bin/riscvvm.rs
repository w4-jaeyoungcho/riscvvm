/// RISC-V VM synchronous to terminal input

extern crate getopts;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate riscvvm;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::env;
use getopts::Options;

use std::str::FromStr;

use ::riscvvm::asm;

use riscvvm::machine::*;
use riscvvm::machine::memory::Memory;
use riscvvm::machine::peri::*;

use std::rc::Rc;
use std::cell::RefCell;

const NUM_TICKS_DEFAULT: u32 = 100;

// Usage: riscvvm <options> <bin file>
// Options
// -t, --continuous-tick-limit=<n> : default 100

fn main() {
    // To distinguish between logs and vm output, need to setup logging
    env_logger::init().unwrap();

    println!("RISCVVM");

    let input: String;
    let continuous_tick_limit: u32;
    let dies_on_exception: bool;
    let memory_probe: Option<u32>;
    let vasm_file: bool;
    // Handle arguments
    {
        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();

        let mut opts = Options::new();
        opts.optopt("t", "continuous_tick_limit", "default 100", "0 means no limit");
        opts.optflag("d", "die_on_exception", "Machine stops when exception is encountered");
        opts.optopt("p", "memory_probe", "address", "set to print memory write to the address");
        opts.optflag("a", "vasm", "Input file is vasm file so assemble first");
        opts.optflag("h", "help", "print this help message");

        let mut matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => panic!(f.to_string()),
        };

        vasm_file = matches.opt_present("vasm");

        if matches.opt_present("h") {
            print_usage(&program, opts);
        }

        // get opts
        continuous_tick_limit = match matches.opt_str("continuous_tick_limit") {
            Some(s) => match u32::from_str(&s) {
                Ok(v) => v,
                Err(e) => panic!(e),
            },
            None => NUM_TICKS_DEFAULT,
        };

        dies_on_exception = matches.opt_present("die_on_exception");

        memory_probe = match matches.opt_str("memory_probe") {
            Some(s) => match u32::from_str(&s) {
                Ok(v) => Some(v),
                Err(e) => panic!(e),
            },
            None => None,
        };

        input = match matches.free.get(0) {
            Some(input) => input.clone(),
            None => print_usage(&program, opts),
        };
    }

    let mut data = Vec::<u8>::new();
    if vasm_file {
        println!("VASM file: {}", &input);

        asm::assemble(&mut data, &input, get_reader).expect("Failed to assemble");
    } else {
        println!("Bin file: {}", &input);

        let mut input_file = File::open(&input).expect("Failed to open Input file");
        input_file.read_to_end(&mut data).expect("Failed to read input file");
    }

    // Prepare machine. Load file into memory. Start ticking

    // Preferred bus configuration
    const MEMORY_START: u32 = 0;
    const MEMORY_WIDTH: u8 = 16;
    const OUTPUT_START: u32 = 0x100000;
    const INPUT_START: u32 = 0x100004;

    // Memory
    let mut memory = Memory::new(memory_probe);
    memory.load(&data);

    let output_device = OutputDevice::new(std::io::stdout());

    let input_device = InputDevice::new(std::io::stdin());

    // Machine and peripherals
    let mut m = Machine::new();

    m.attach("memory", memory, MEMORY_START, MEMORY_WIDTH);
    m.attach("output", output_device, OUTPUT_START, 0);
    m.attach("input", input_device, INPUT_START, 0);

    println!("Simulation starting");

    let mut line = String::new();

    match m.run(continuous_tick_limit, true) {
        Ok(cycles) => (),
        Err(e) => {
            println!("RunError: {:?}", e);
        }
    }

    println!("Machine is now stopped.");

}

fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
    ::std::process::exit(1);
}

// Rust is a great language. I had doubts, because it wasn't made by Rob Pike.
fn file_read_test() {

    // File read test
    let mut f = File::open("banana.txt").unwrap();

    let mut buffer = [0 as u8; 32];

    let read = f.read(&mut buffer).unwrap();

    println!("Read {} bytes. The bytes: {:?}", read, &buffer);

    let s;
    {
        let slice = &buffer[..read];

        let v = Vec::from(slice);

        s = String::from_utf8(v).unwrap();
    }

    println!("This is the string, finally: {}", &s);
}

fn get_reader(file_path: &str) -> Result<Box<Read>, std::io::Error> {
    println!("get_reader: getting file {}", file_path);
    File::open(file_path).map(|f| Box::new(f) as Box<Read>)
}