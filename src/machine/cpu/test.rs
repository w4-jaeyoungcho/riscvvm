
use super::*;

use ::parser;
use ::translate;
use ::translate::symtab;
use ::lexer::mem_reader;
use ::arch::inst::register;

// Test cpu state (registers and stuff) after execution through binary from translate
#[test]
fn test_cpu() {

    let mut m = MockMachine {
        cpu: Cpu::new(),
        bus: MockBus { addr: 0, value: 0 },
        symtab: symtab::Symtab::prepopulated(),
    };

    // reset state...
    for i in 0..32 {
        assert_eq!(m.cpu.regs[i], 0);
    }

    m.exec("(nop)");

    // reset state...
    for i in 0..32 {
        assert_eq!(m.cpu.regs[i], 0);
    }

    // Test I computation
    m.exec("(addi t0 t0 1)");
    assert_eq!(m.reg_from_name("t0"), 1, "addi 1");

    m.exec("(addi t0 t0 0xF0)");
    assert_eq!(m.reg_from_name("t0"), 0xF1, "addi 0xF0");

    m.exec("(slti t1 t0 1)");
    assert_eq!(m.reg_from_name("t1"), 0, "slti false");

    m.exec("(slti t1 t0 0xFF)");
    assert_eq!(m.reg_from_name("t1"), 1, "slti true");

    m.exec("(ori t2 t0 0x765)");
    assert_eq!(m.reg_from_name("t2"), 0x7F5, "ori");

    m.exec("(slli t2 t2 4)");
    assert_eq!(m.reg_from_name("t2"), 0x7F50, "slli");

    // Test U instructions
    m.exec("(lui t2 0x12345678)");
    assert_eq!(m.reg_from_name("t2"), 0x12345000, "lui");

//    assert_eq!(cpu.pc, 0x1C, "pc check");
    m.cpu.pc = 0x1C;
    m.exec("(auipc t2 0x23456789)");
    assert_eq!(m.reg_from_name("t2"), 0x2345601C, "auipc");

    // Need to check wrapping cases/corner cases...

    // Test R computation

    m.exec("(li t0 5)");
    assert_eq!(m.reg_from_name("t0"), 5, "li");
    m.exec("(mv t1 t0)");
    assert_eq!(m.reg_from_name("t1"), 5, "mv");
    m.exec("(add t0 t0 t1)");
    assert_eq!(m.reg_from_name("t0"), 10, "add");

    m.exec("(li t3 3)");
    m.exec("(sub t0 t0 t3)");
    assert_eq!(m.reg_from_name("t0"), 7, "sub");

    m.exec("(sll t2 t0 t1)");
    assert_eq!(m.reg_from_name("t2"), 224, "sll");

    // Test JAL
    m.cpu.pc = 0x100;
    m.exec("(jal t0 0x1234)");
    assert_eq!(m.reg_from_name("t0"), 0x104, "jal rd");
    assert_eq!(m.cpu.pc, 0x1334, "jal pc");

    // Test conditional branches
    m.exec("(li t1 0)");
    m.cpu.pc = 0;
    m.exec("(beq t1 zero 0x120)");
    assert_eq!(m.cpu.pc, 0x120, "beq");

    m.exec("(li t1 5)");
    m.cpu.pc = 0;
    m.exec("(blt t1 zero 0x120)");
    assert_eq!(m.cpu.pc, 0x4, "blt false");

    // Test Load and Store
    let there: u32 = 0x450;
    m.symtab.register_label("there", there);
    // TODO: Note that parameter order for store is not good
    m.exec("(li t1 0x123)");
    m.exec("(sw zero t1 there)");

    assert_eq!(m.bus.addr, there, "sw addr");
    assert_eq!(m.bus.value, 0x123, "sw value");

    m.exec("(lw t0 zero there)");
    assert_eq!(m.reg_from_name("t0"), 0x123, "lw");
}

struct MockBus {
    addr: u32,
    value: u32,
}

impl MasterBusEnd for MockBus {
    fn read_word(&mut self, addr: u32) -> Result<u32, ()> {
        assert_eq!(addr & 0x3, 0);
        assert_eq!(addr, self.addr);
        Ok(self.value)
    }
    fn write_word(&mut self, addr: u32, value: u32) -> Result<(), ()> {
        assert_eq!(addr & 0x3, 0);
        self.addr = addr;
        self.value = value;
        Ok(())
    }
    fn is_interrupting(&self) -> bool {
        false
    }
}


// Convenience things

struct MockMachine {
    cpu: Cpu,
    bus: MockBus,
    symtab: symtab::Symtab,
}

impl MockMachine {
    // no exception
    fn exec(&mut self, s: &str) {
        let word = inst_to_word(s, &self.symtab, self.cpu.pc);
        self.cpu.cycle(word, &mut self.bus);
        assert_eq!(self.cpu.cause, 0, "cpu.cause");
        assert_eq!(self.cpu.epc, 0, "cpu.epc");
    }

    fn reg_from_name(&self, name: &str) -> i32 {
        self.cpu.regs[register::index(name).unwrap() as usize]
    }
}

fn inst_to_word(s: &str, symtab: &symtab::Symtab, pc: u32) -> u32 {
    let expr = parser::parse(s);
    let word = translate::translate(&expr, symtab, pc).expect("translate");
    word
}
