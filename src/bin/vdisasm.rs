
extern crate riscvvm;
extern crate getopts;

use getopts::Options;
use std::env;
use std::path::{PathBuf};

use std::io::prelude::*;
use std::fs::File;

/*
USAGE
vdisasm -o outfile infile
*/

fn main() {
    println!("VDISASM");

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
        if !path.set_extension("vdisasm") {
            panic!("Some confusion");
        }
        String::from(path.to_str().unwrap())
    });

    println!("input: {}, output: {}", &input, &output);

    // Prepare reader and writer

    let writer = File::create(&output).unwrap();

    let reader = File::open(&input).unwrap();
    // quickly check that the length of the file is multiple of the size of a word
    {
        let metadata = reader.metadata().unwrap();
        let length = metadata.len();
        println!("Input file size: {}", length);
        if length % 4 != 0 {
            println!("Eh.. the file size is not multiple of 4: {}", length);
            std::process::exit(1);
        }
    }

    println!("Start disassembling");

    riscvvm::disasm::disassemble(writer, reader).expect("Disassembler returned error");

    println!("Done! I am pretty sure");
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
}
