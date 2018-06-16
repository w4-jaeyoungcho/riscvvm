
extern crate riscvvm;
extern crate getopts;

use getopts::Options;
use std::env;
use std::path::{PathBuf};

use std::io::prelude::*;
use std::fs::File;

/*
USAGE
vasm
*/

fn main() {
    println!("VASM: A RISC-V Assembler");

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "out", "output bin file", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let mut matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.free.is_empty() {
        print_usage(&program, opts);
        ::std::process::exit(1);
    }

    let input =  matches.free.remove(0);

    let output = matches.opt_str("o").unwrap_or({
        let mut path = PathBuf::from(&input);
        if !path.set_extension("bin") {
            panic!("Some confusion");
        }
        String::from(path.to_str().unwrap())
    });

    println!("input: {}, output: {}", &input, &output);

    // Ready reader and writer

    let writer = File::create(&output).unwrap();

    println!("Start assembling");

    riscvvm::asm::assemble(Box::new(writer), &input, get_reader).expect("Assembler returned error");

    println!("Done! I guess");
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
}

fn get_reader(file_path: &str) -> Result<Box<Read>, std::io::Error> {
    File::open(file_path).map(|f| Box::new(f) as Box<Read>)
}