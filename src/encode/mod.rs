
mod test;
pub mod patch;

// RISC-V Encoder

use ::arch::inst::*;

pub fn encode(inst: &Inst, args: &[u32]) -> u32 {
    let mut code = 0x00000003 as u32; // base

//    println!("encode");
    // opcodes
    for o in inst.opcodes.iter() {
        code = patch::write(code, o.patch.offset, o.patch.length, o.value);
    }
//    println!("encode: {:08X}", code);

    // args
    assert_eq!(inst.args.len(), args.len());

    for (a, &v) in inst.args.iter().zip(args.iter()) {
        for m in a.mappings.0.iter() {
            let read = patch::read(v, m.source_offset, m.length);
            code = patch::write(code, m.dest_offset, m.length, read);
        }
    }

//    println!("encode result: {:08X}", code);
    code
}
