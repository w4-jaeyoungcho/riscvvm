
//TODO check that the cycle is pipeline correct in regard to exceptions (write back must not succeed if exception)

#[cfg(test)]
mod test;

mod system;

use self::system::*;

// Cpu

use super::*;

use ::arch::inst::*;
use ::arch::inst::patch;
use ::arch::inst::opcode::*;
use ::arch::inst::arg;
use ::arch::system::*;

use enum_primitive::FromPrimitive;

const RESET_VECTOR: u32 = 0;

const NUM_REGS: usize = 32;

/*
match opcode::FUNCT3_OP_IMM::from_u32(patch::FUNCT3.read(word)) {
                    Some(funct3) => funct3,
                    None => {
                        self.exception(ILLEGAL_INSTRUCTION);
                        return;
                    }
                };
*/
macro_rules! read_opcode {
($self:expr, $word:expr, $opcode_enum:ident, $patch:ident) => {
    match opcode::$opcode_enum::from_u32(patch::$patch.read($word)) {
        Some(v) => v,
        None => {
            $self.exception(ILLEGAL_INSTRUCTION);
            return;
        }
    }
}
}

#[derive(Default)]
pub struct Cpu {
    pub regs: [i32; NUM_REGS],
    pub pc: u32,

    // SYSTEM state
    // CSRs
    pub level: u8,

    pub status: u32,

    pub scratch: u32,
    pub epc: u32,
    pub cause: u32,
    pub mtval: u32,

    // Interrupts
    pub ip: u32,
    pub ie: u32,

    pub wfi: bool,

    // csrs
    pub cycle: u64,

    pub mtvec: u32,

    // Debug...
    pub num_cycles: i32,

    pub die_on_exception: bool,

    // DEBUG
    pub last_word: u32,

    pub pc_trail: Option<Vec<u32>>,
}

impl Cpu {
    pub fn new() -> Cpu {
        // Reset condition...
        let mut cpu: Cpu = Default::default();
        cpu.debug_reset();
        cpu
    }

    fn debug_reset(&mut self) {
        self.reset();
    }

    fn reset(&mut self) {
        self.level = MACHINE;
        self.status = 0;
        self.cause = 0;

        self.ip = 0;
        self.ie = 0;

        self.mtvec = MTVEC_VALUE;
    }

    fn reg(&self, r: u8) -> i32 {
        info!("reg: {} = 0x{:08X}", ::arch::inst::register::abi_name(r), self.regs[r as usize]);
        self.regs[r as usize]
    }

    fn set_reg(&mut self, r: u8, v: i32) {
        info!("set_reg: {} <- 0x{:08X}", ::arch::inst::register::abi_name(r), v);
        if r != 0 {
            self.regs[r as usize] = v;
        }
    }

    pub(super) fn tick(&mut self, bus: &mut MasterBusEnd) {
        let pc = self.pc; // Borrow checker thing
        self.pc_trail.as_mut().map(|t| t.push(pc));

        self.wfi = false;
        self.tick_inner(bus);
        self.num_cycles += 1;
    }

    fn tick_inner(&mut self, bus: &mut MasterBusEnd) {
        // Check for NMI
        // TODO

        // Check for interrupt conditions
        /*
        Interrupt priority rule
        * higher priv interrupt is higher priority
        * among the same priv:
          * external
          * software
          * timer
          * so both interrupts and traps are called interrupts?

        */

        // Check from high priv to low
        for &level in LEVELS.iter().rev() {
            if !self.is_interrupt_possible(level) {
                break;
            }

            // Check each of interrupt sources
            // TODO
        }

        // Generate address misaligned exception on branch/jump target
        if self.pc & 0x3 != 0 {
            self.mtval = self.pc;
            self.cause = INSTRUCTION_ADDRESS_MISALIGNED;
            return;
        }

        // Instruction fetch
        let word = match bus.read_word(self.pc){
            Ok(v) => v,
            Err(e) => {
                self.mtval = self.pc;
                self.exception(INSTRUCTION_ACCESS_FAULT);
                return;
            },
        };

        self.last_word = word;

        self.cycle(word, bus);
    }

    fn cycle(&mut self, word: u32, bus: &mut MasterBusEnd) {
//        println!("cpu cycle on {:X}, word: {:X}", self.pc, word);
        // NOP may be optimized here

        // check
        if word & 0x3 != 0x3 {
            self.exception(ILLEGAL_INSTRUCTION);
            return;
        }

        // Read opcode
        let opcode = read_opcode!(self, word, OPCODE, OPCODE);
        let inst_type = inst_type(opcode);

        match inst_type {

            INST_TYPE::R => {
                let rd = arg::RD.read(word) as u8;
                let rs1 = arg::RS1.read(word) as u8;
                let rs2 = arg::RS2.read(word) as u8;

                // OPCODE OP
                assert_eq!(opcode, OPCODE::OP);

                let funct7 = read_opcode!(self, word, FUNCT7_OP, FUNCT7);
                let res = match funct7 {
                    FUNCT7_OP::BASE => {
                        let funct3 = read_opcode!(self, word, FUNCT3_OP, FUNCT3);
                        match funct3 {
                            FUNCT3_OP::ADD => self.reg(rs1).wrapping_add(self.reg(rs2)),
                            FUNCT3_OP::SLL => self.reg(rs1) << (self.reg(rs2) & 0x1F),
                            FUNCT3_OP::SLT => if self.reg(rs1) < self.reg(rs2) { 1 } else { 0 },
                            FUNCT3_OP::SLTU => if (self.reg(rs1) as u32) < (self.reg(rs2) as u32) { 1 } else { 0 },
                            FUNCT3_OP::XOR => self.reg(rs1) ^ self.reg(rs2),
                            FUNCT3_OP::SRL => self.reg(rs1) >> self.reg(rs2) & 0x1F,
                            FUNCT3_OP::OR => self.reg(rs1) | self.reg(rs2),
                            FUNCT3_OP::AND => self.reg(rs1) & self.reg(rs2),
                        }
                    }
                    FUNCT7_OP::ALT => {
                        let funct3 = read_opcode!(self, word, FUNCT3_OP_ALT, FUNCT3);
                        match funct3 {
                            FUNCT3_OP_ALT::SUB => self.reg(rs1).wrapping_sub(self.reg(rs2)),
                            FUNCT3_OP_ALT::SRA => (self.reg(rs1) as u32 >> self.reg(rs2) & 0x1F) as i32,
                        }
                    }
                };

                self.set_reg(rd, res);
            }

            INST_TYPE::I => {
                let rd = arg::RD.read(word) as u8;
                let rs1 = arg::RS1.read(word) as u8;
                let imm = arg::IMM12.read(word) as i32;

                match opcode {
                    OPCODE::LOAD => {
                        // Completly faulting at misaligned load for now

                        let funct3 = read_opcode!(self, word, FUNCT3_LOAD, FUNCT3);

                        let addr = self.reg(rs1).wrapping_add(imm) as u32;
                        let byte_offset = addr & 0x3;

                        // Catch misaligned access early
                        if funct3 == FUNCT3_LOAD::LH || funct3 == FUNCT3_LOAD::LHU {
                            if !(byte_offset == 0 || byte_offset == 2) {
                                self.exception(LOAD_ADDRESS_MISALIGNED);
                                return;
                            }
                        } else if funct3 == FUNCT3_LOAD::LW {
                            if byte_offset != 0 {
                                self.exception(LOAD_ADDRESS_MISALIGNED);
                                return;
                            }
                        }

                        let addr_word_aligned = addr & !0x3;

                        let read = match bus.read_word(addr_word_aligned) {
                            Ok(v) => v,
                            Err(()) => {
                                self.exception(LOAD_ACCESS_FAULT);
                                return;
                            }
                        };

                        let value = match funct3 {
                            FUNCT3_LOAD::LB => (read >> byte_offset*8) as i8 as i32,
                            FUNCT3_LOAD::LH => (read >> byte_offset*8) as i16 as i32,
                            FUNCT3_LOAD::LW => read as i32,
                            FUNCT3_LOAD::LBU => (read >> byte_offset*8) as u8 as i32,
                            FUNCT3_LOAD::LHU => (read >> byte_offset*8) as u16 as i32,
                        };

                        self.set_reg(rd, value);
                    }

                    OPCODE::OP_IMM => {
                        let funct3 = read_opcode!(self, word, FUNCT3_OP_IMM, FUNCT3);

                        let funct7 = patch::FUNCT7.read(word);

                        let value = match funct3 {
                            FUNCT3_OP_IMM::ADDI => self.reg(rs1).wrapping_add(imm),
                            FUNCT3_OP_IMM::SLLI => self.reg(rs1) << (imm & 0x1F),
                            FUNCT3_OP_IMM::SLTI => if self.reg(rs1) < imm { 1 } else { 0 },
                            FUNCT3_OP_IMM::SLTIU => if (self.reg(rs1) as u32) < (imm as u32) { 1 } else { 0 },
                            FUNCT3_OP_IMM::XORI => self.reg(rs1) ^ imm,
                            FUNCT3_OP_IMM::SRLI => match funct7 {
                                // SRLI
                                0 => (self.reg(rs1) as u32 >> (imm & 0x1F)) as i32,
                                // SRAI
                                0x20 => self.reg(rs1) >> (imm & 0x1F),
                                _ => {
                                    self.exception(ILLEGAL_INSTRUCTION);
                                    return;
                                }
                            },
                            FUNCT3_OP_IMM::ORI => self.reg(rs1) | imm,
                            FUNCT3_OP_IMM::ANDI => self.reg(rs1) & imm,
                        };

                        self.set_reg(rd, value);
                    }

                    OPCODE::JALR => {
                        let link = self.pc.wrapping_add(4);
                        self.set_reg(rd, link as i32);
                        self.pc = (self.reg(rs1).wrapping_add(imm) as u32) & !0x1;
                        return;
                    }
                    OPCODE::SYSTEM => {
                        let funct3 = read_opcode!(self, word, FUNCT3_SYSTEM, FUNCT3);

                        match funct3 {
                            FUNCT3_SYSTEM::PRIV => {
                                let funct12 = read_opcode!(self, word, FUNCT12_PRIV, FUNCT12);

                                match funct12 {
                                    FUNCT12_PRIV::ECALL => {
                                        if rs1 != 0 || rd != 0 {
                                            self.exception(ILLEGAL_INSTRUCTION);
                                            return;
                                        }

                                        panic!("Implement ECALL")
                                    }
                                    FUNCT12_PRIV::EBREAK => {
                                        if rs1 != 0 || rd != 0 {
                                            self.exception(ILLEGAL_INSTRUCTION);
                                            return;
                                        }

                                        panic!("Implement EBREAK, whatever it is")
                                    }

                                    // RET
                                    FUNCT12_PRIV::URET => {
                                        self.ret(USER);
                                        return;
                                    }
                                    FUNCT12_PRIV::SRET => {
                                        if self.level < SUPERVISOR {
                                            self.exception(ILLEGAL_INSTRUCTION);
                                            return;
                                        }
                                        self.ret(SUPERVISOR);
                                        return;
                                    }
                                    FUNCT12_PRIV::MRET => {
                                        if self.level < MACHINE {
                                            self.exception(ILLEGAL_INSTRUCTION);
                                            return;
                                        }
                                        self.ret(MACHINE);
                                        return;
                                    }
                                    FUNCT12_PRIV::WFI => self.wfi = true,
                                    FUNCT12_PRIV::SFENCEVM => panic!("implement please"),
                                }
                            }
                            // CSR instructions
                            funct3 => {
                                let csr = (imm & 0xFFF) as u32;
                                let zimm = arg::ZIMM.read(word);

                                // Check privilege level
                                if csr_level(csr) > self.level {
                                    self.exception(ILLEGAL_INSTRUCTION);
                                    return;
                                }

                                //NOTE: Write accessibility is check is deferred. I am not sure about the right behavior

                                // x0 special:
                                // If rs1=x0, Rs and Rc will not write to the CSR at all.
                                // If rd=x0, Rw will not read the CSR.

                                // Read from CSR
                                // Not writing to rd if this instruction raises exception
                                let r = if !(rd == 0 && (funct3 == FUNCT3_SYSTEM::CSRRW || funct3 == FUNCT3_SYSTEM::CSRRWI)) {
                                    if let Ok(value) = self.read_csr(csr) {
                                        value
                                    } else {
                                        self.exception(ILLEGAL_INSTRUCTION);
                                        return;
                                    }
                                } else {
                                    0 // don't matter
                                };

                                let v_rs1 = self.reg(rs1) as u32;
                                let uimm = rs1 as u32; // Zero-extending

                                // write_csr, .. helper functions affect system
                                let result = match funct3 {
                                    FUNCT3_SYSTEM::CSRRW => self.write_csr(csr, v_rs1),
                                    FUNCT3_SYSTEM::CSRRS => if rs1 != 0 { self.write_csr(csr, r | v_rs1) } else { Ok(()) },
                                    FUNCT3_SYSTEM::CSRRC => if rs1 != 0 { self.write_csr(csr, r & !v_rs1) } else { Ok(()) },
                                    FUNCT3_SYSTEM::CSRRWI => self.write_csr(csr, uimm),
                                    FUNCT3_SYSTEM::CSRRSI => if uimm != 0 { self.write_csr(csr, r | uimm) } else { Ok(()) },
                                    FUNCT3_SYSTEM::CSRRCI => if uimm != 0 { self.write_csr(csr, r & !uimm) } else { Ok(()) },
                                    _ =>  panic!("Should be statically impossible... I wish compiler were smarter about enums"),
                                };
                                match result {
                                    Err(()) => {
                                        self.exception(ILLEGAL_INSTRUCTION);
                                        return;
                                    }
                                    _ => (),
                                }

                                self.set_reg(rd, r as i32);
                            }
                        }
                    }
                    _ => panic!("statically impossible"),
                }
            }

            INST_TYPE::S => {
                assert_eq!(opcode, OPCODE::STORE);

                let funct3 = read_opcode!(self, word, FUNCT3_STORE, FUNCT3);

                // no misaligned store for now
                let rs1 = arg::RS1.read(word) as u8;
                let rs2 = arg::RS2.read(word) as u8;
                let imm = arg::SIMM12.read(word);

                let addr = (self.reg(rs1) as u32).wrapping_add(imm);
                let addr_word_aligned = addr & !0x3;
                let byte_offset = (addr & 0x3) as u8;

                // fault on misaligned store
                if funct3 == FUNCT3_STORE::SH {
                    if byte_offset != 0 && byte_offset != 2 {
                        self.exception(STORE_ADDRESS_MISALIGNED);
                        return;
                    }
                } else if funct3 == FUNCT3_STORE::SW {
                    if byte_offset != 0 {
                        self.exception(STORE_ADDRESS_MISALIGNED);
                        return;
                    }
                }

                let value = self.reg(rs2) as u32;

                // CPU driven read and write back on smaller than byte write
                match funct3 {
                    FUNCT3_STORE::SB => {
                        let read = match bus.read_word(addr_word_aligned) {
                            Ok(v) => v,
                            Err(()) => {
                                self.exception(LOAD_ACCESS_FAULT); // ????????????????????? should it load or store access fault?
                                return;
                            }
                        };

                        let mask: u32 = 0xFF;
                        let updated = (read & !(mask << byte_offset*8)) | ((value & mask) << byte_offset*8);
                        match bus.write_word(addr_word_aligned, updated) {
                            Ok(()) => (),
                            Err(()) => {
                                self.exception(STORE_ACCESS_FAULT);
                                return;
                            }
                        }
                    }
                    FUNCT3_STORE::SH => {
                        let read = match bus.read_word(addr_word_aligned) {
                            Ok(v) => v,
                            Err(()) => {
                                self.exception(LOAD_ACCESS_FAULT); // ????????????????????? should it load or store access fault?
                                return;
                            }
                        };

                        let mask: u32 = 0xFFFF;
                        let updated = (read & !(mask << byte_offset*8)) | ((value & mask) << byte_offset*8);
                        match bus.write_word(addr_word_aligned, updated) {
                            Ok(()) => (),
                            Err(()) => {
                                self.exception(STORE_ACCESS_FAULT);
                                return;
                            }
                        }
                    }
                    FUNCT3_STORE::SW => {
                        match bus.write_word(addr_word_aligned, value) {
                            Ok(()) => (),
                            Err(()) => {
                                println!("STORE error with addr={:08X} addr_word_aligned={:08X}", addr, addr_word_aligned);
                                self.exception(STORE_ACCESS_FAULT);
                                return;
                            }
                        }
                    }
                }


            }

            INST_TYPE::SB => {
                assert_eq!(opcode, OPCODE::BRANCH);

                let rs1 = arg::RS1.read(word) as u8;
                let rs2 = arg::RS2.read(word) as u8;
                let imm = arg::BIMM12.read(word);

                let v_rs1 = self.reg(rs1);
                let v_rs2 = self.reg(rs2);

                let funct3 = read_opcode!(self, word, FUNCT3_BRANCH, FUNCT3);
                let branch = match funct3 {
                    FUNCT3_BRANCH::BEQ => v_rs1 == v_rs2,
                    FUNCT3_BRANCH::BNE => v_rs1 != v_rs2,
                    FUNCT3_BRANCH::BLT => v_rs1 < v_rs2,
                    FUNCT3_BRANCH::BGE => v_rs1 >= v_rs2,
                    FUNCT3_BRANCH::BLTU => (v_rs1 as u32) < (v_rs2 as u32),
                    FUNCT3_BRANCH::BGEU => (v_rs1 as u32) >= (v_rs2 as u32),
                };

                if branch {
                    self.pc = self.pc.wrapping_add(imm);
                    return;
                }
            }

            INST_TYPE::U => {
                let rd = arg::RD.read(word) as u8;
                let imm = arg::IMM20.read(word);

                let value = match opcode {
                    OPCODE::LUI => imm,
                    OPCODE::AUIPC => imm.wrapping_add(self.pc),
                    _ => panic!("Statically impossible"),
                } as i32;

                self.set_reg(rd, value);
            }

            INST_TYPE::UJ => {
                assert_eq!(opcode, OPCODE::JAL);
                let rd = arg::RD.read(word) as u8;
                let imm = arg::JIMM20.read(word);

                let link =self.pc.wrapping_add(4);
                self.set_reg(rd, link as i32);
                self.pc = self.pc.wrapping_add(imm);
                return;
            }
        }

        // No jump/branch path come here

        // Advance pc
        self.pc = self.pc.wrapping_add(4);
    }


    // Call to SYSTEM
    // This has to look like this because I don't know how to impl a struct across multiple files

    //TODO mtval should be cleared for exceptions that don't set it
    fn exception(&mut self, cause: u32) {
        // Convenience debug measure
        if self.die_on_exception {
            // Print pc trail
            if self.pc_trail.is_some() {
                println!("PC trail:");
                for v in self.pc_trail.as_ref().unwrap() {
                    println!("{:08X}", v);
                }
                println!();
            }

            panic!("dying on exception with pc={:08X}, cause={}, word={:08X}, num_cycles={}", self.pc, cause, self.last_word, self.num_cycles);
        }

        exception(self, cause);
    }

    fn is_interrupt_possible(&self, level: u8) -> bool {
        return is_interrupt_possible(self, level);
    }

    fn ret(&mut self, from: u8) {
        ret(self, from);
    }

    fn trap(&mut self, to: u8) {
        trap(self, to);
    }

    // CSR instructions helpers
    // They handle exceptional themselves
    fn read_csr(&mut self, csr: u32) -> Result<u32, ()> {
        read_csr(self, csr)
    }

    // I wish ? operator were more general...
    fn write_csr(&mut self, csr: u32, v: u32) -> Result<(), ()> {
        if is_csr_readonly(csr) {
            return Err(());
        }
        write_csr(self, csr, v)
    }
}

fn shamtify(v: i32) -> u8 {
    (v & 0x1F) as u8
}