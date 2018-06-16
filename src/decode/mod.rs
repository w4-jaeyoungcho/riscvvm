
mod test;

use ::arch::inst;

use ::encode::patch::*;

// Returns (inst name, inst, arg values)
pub fn decode(x: u32) -> Option<(&'static str, &'static inst::Inst, Vec<u32>)> {
    // go through all the insts and compare opcodes
    'inst: for &(name, ref inst) in inst::INSTS {
//        println!("decode: checking for inst {}", name);
        // Check opcodes
        for opcode in inst.opcodes {
            // Extract patch from x
            let v = read(x, opcode.patch.offset, opcode.patch.length);
//            println!("decode: v = {}", v);
            if opcode.value != v {
                continue 'inst;
            }
        }

        // Match
        // Read args
        let args: Vec<u32> = inst.args.iter().map(|&arg| arg.read(x)).collect();
        return Some((name, inst, args))
    }
    None
}

