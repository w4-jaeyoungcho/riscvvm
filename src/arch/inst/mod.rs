pub mod patch;
pub mod register;
pub mod arg;
pub mod opcode;

use self::patch::Patch;
use self::arg::*;

#[derive(Debug)]
pub struct Opcode {
    pub patch: &'static Patch,
    pub value: u32,
}

#[derive(Debug)]
pub struct Inst {
    pub args: &'static [&'static Arg],
    pub opcodes: &'static [Opcode],
}

macro_rules! i_type_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &IMM12],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::$funct3 as u32 },
        ],
    }
)
}

macro_rules! load_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &IMM12],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::LOAD as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_LOAD::$funct3 as u32 },
        ],
    }
)
}

macro_rules! store_inst {
($funct3:ident) => (
    Inst{
        args: &[&RS1, &RS2, &SIMM12],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::STORE as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_STORE::$funct3 as u32 },
        ],
    }
)
}

macro_rules! i_type_shift_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &SHAMT],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::$funct3 as u32 },
            Opcode{ patch: &patch::FUNCT7, value: 0 },
        ],
    }
)
}

macro_rules! i_type_shift_inst_alt {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &SHAMT],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM_ALT::$funct3 as u32 },
            Opcode{ patch: &patch::FUNCT7, value: 0x20 },
        ],
    }
)
}


macro_rules! u_type_inst {
($opcode:ident) => (
    Inst{
        args: &[&RD, &IMM20],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::$opcode as u32 },
        ],
    }
)
}

macro_rules! r_type_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &RS2],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::OP as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_OP::$funct3 as u32 },
            Opcode{ patch: &patch::FUNCT7, value: 0 },
        ],
    }
)
}

macro_rules! r_type_inst_alt {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &RS2],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::OP as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_ALT::$funct3 as u32 },
            Opcode{ patch: &patch::FUNCT7, value: 0x20 },
        ],
    }
)
}

macro_rules! branch_inst {
($funct3:ident) => (
    Inst{
        args: &[&RS1, &RS2, &BIMM12],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::BRANCH as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_BRANCH::$funct3 as u32 },
        ],
    }
)
}

macro_rules! csr_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &RS1, &CSR],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::SYSTEM as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_SYSTEM::$funct3 as u32 },
        ],
    }
)
}

macro_rules! csr_i_inst {
($funct3:ident) => (
    Inst{
        args: &[&RD, &ZIMM, &CSR],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::SYSTEM as u32 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_SYSTEM::$funct3 as u32 },
        ],
    }
)
}

macro_rules! env_inst {
($funct12:ident) => (
    Inst{
        args: &[],
        opcodes: &[
            Opcode{ patch: &patch::OPCODE, value: opcode::OPCODE::SYSTEM as u32 },
            Opcode{ patch: &patch::RD, value: 0 },
            Opcode{ patch: &patch::FUNCT3, value: opcode::FUNCT3_SYSTEM::PRIV as u32 },
            Opcode{ patch: &patch::RS1, value: 0 },
            Opcode{ patch: &patch::FUNCT12, value: opcode::FUNCT12_PRIV::$funct12 as u32 },
        ],
    }
)
}

// In the descending order of precedence for disassemble
pub const INSTS: &[(&'static str, Inst)] = &[
    ("nop", Inst {
        args: &[],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::ADDI as u32 },
            Opcode { patch: &patch::RS1, value: 0 },
            Opcode { patch: &patch::RD, value: 0 },
            Opcode { patch: &patch::IMM12, value: 0 },
        ],
    }),
    // addi rd, rs1, 0
    ("mv", Inst {
        args: &[&arg::RD, &arg::RS1],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::ADDI as u32 },
            Opcode { patch: &patch::IMM12, value: 0 },
        ],
    }),
    // addi rd, zero, imm12
    ("li", Inst {
        args: &[&arg::RD, &arg::IMM12],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::ADDI as u32 },
            Opcode { patch: &patch::RS1, value: 0 },
        ],
    }),
    // sltiu rd, rs1, 1
    ("seqz", Inst {
        args: &[&arg::RD, &arg::RS1],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::SLTIU as u32 },
        ],
    }),
    // xori rd, rs1, -1
    ("not", Inst {
        args: &[&arg::RD, &arg::RS1],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP_IMM as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP_IMM::XORI as u32 },
            Opcode { patch: &patch::IMM12, value: 0xFFF },
        ],
    }),
    // sltu rd, x0, rs2
    ("snez", Inst {
        args: &[&arg::RD, &arg::RS2],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::OP as u32 },
            Opcode { patch: &patch::FUNCT3, value: opcode::FUNCT3_OP::SLTU as u32 },
            Opcode { patch: &patch::FUNCT7, value: 0 },
            Opcode { patch: &patch::RS1, value: 0 },
        ],
    }),
    ("j", Inst {
        args: &[&arg::JIMM20],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::JAL as u32 },
            Opcode { patch: &patch::RD, value: 0 },
        ],
    }),

    // TODO
    // CSRR, CSRW, CSRWI
    // CSRS, CSRC, CSRSI, CSRCI
    // RDCYCLE, RDCYCLEH, RDTIME, RDTIMEH, RDINSTRET, RDINSTRETH

    // Computational

    // imm12
    ("addi", i_type_inst!(ADDI)),
    ("slti", i_type_inst!(SLTI)),
    ("sltiu", i_type_inst!(SLTIU)),
    ("andi", i_type_inst!(ANDI)),
    ("ori", i_type_inst!(ORI)),
    ("xori", i_type_inst!(XORI)),

    // imm12-shifts
    ("slli", i_type_shift_inst!(SLLI)),
    ("srli", i_type_shift_inst!(SRLI)),

    ("srai", i_type_shift_inst_alt!(SRAI)),

    // imm20
    ("lui", u_type_inst!(LUI)),
    ("auipc", u_type_inst!(AUIPC)),

    // r-types
    ("add", r_type_inst!(ADD)),
    ("slt", r_type_inst!(SLT)),
    ("sltu", r_type_inst!(SLTU)),
    ("and", r_type_inst!(AND)),
    ("or", r_type_inst!(OR)),
    ("xor", r_type_inst!(XOR)),
    ("sll", r_type_inst!(SLL)),
    ("srl", r_type_inst!(SRL)),

    ("sub", r_type_inst_alt!(SUB)),
    ("sra", r_type_inst_alt!(SRA)),

    // Control transfer
    ("jal", Inst {
        args: &[&arg::RD, &arg::JIMM20],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::JAL as u32 },
        ],
    }),
    ("jalr", Inst {
        args: &[&arg::RD, &arg::RS1, &arg::IMM12],
        opcodes: &[
            Opcode { patch: &patch::OPCODE, value: opcode::OPCODE::JALR as u32 },
            Opcode { patch: &patch::FUNCT3, value: 0 },
        ],
    }),

    ("beq", branch_inst!(BEQ)),
    ("bne", branch_inst!(BNE)),
    ("blt", branch_inst!(BLT)),
    ("bltu", branch_inst!(BLTU)),
    ("bge", branch_inst!(BGE)),
    ("bgeu", branch_inst!(BGEU)),

    // Load/Store
    ("lb", load_inst!(LB)),
    ("lh", load_inst!(LH)),
    ("lw", load_inst!(LW)),
    ("lbu", load_inst!(LBU)),
    ("lhu", load_inst!(LHU)),

    ("sb", store_inst!(SB)),
    ("sh", store_inst!(SH)),
    ("sw", store_inst!(SW)),

    // Memory model
    // fence..

    // SYSTEM
    ("csrrw", csr_inst!(CSRRW)),
    ("csrrs", csr_inst!(CSRRS)),
    ("csrrc", csr_inst!(CSRRC)),
    ("csrrwi", csr_i_inst!(CSRRWI)),
    ("csrrsi", csr_i_inst!(CSRRSI)),
    ("csrrci", csr_i_inst!(CSRRCI)),

    // Timers and counters..

    // Env calls
    ("ecall", env_inst!(ECALL)),
    ("ebreak", env_inst!(EBREAK)),

    ("uret", env_inst!(URET)),
    ("sret", env_inst!(SRET)),
//    ("hret", env_inst!(HRET)),
    ("mret", env_inst!(MRET)),

    ("wfi", env_inst!(WFI)),

    // Low priority pseudoinstructions
];

pub fn inst(name: &str) -> Option<&'static Inst> {
    for &(n, ref i) in INSTS {
        if n == name {
            return Some(i);
        }
    }

    None
}

pub fn is_inst(name: &str) -> bool {
    INSTS.iter().any(|x| x.0 == name)
}

pub fn format_arg(arg: &Arg, v: u32) -> String {
    match &arg.arg_type {
        &ArgType::Register => format!("{}", ::arch::inst::register::abi_name(v as u8)),
        &ArgType::Address => format!("0x{:08X}", v),
        &ArgType::General => format!("{}", v as i32),
        &ArgType::Csr => {
            if let Some(s) = ::arch::system::csr_name(v) {
                format!("{}", s)
            } else {
                format!("(csr {})", v)
            }
        },
    }
}

pub enum INST_TYPE {
    R,
    I,
    S,
    SB,
    U,
    UJ,
}

use self::opcode::*;

pub fn inst_type(opcode: OPCODE) -> INST_TYPE {
    match opcode {
        OPCODE::OP => INST_TYPE::R,

        OPCODE::LOAD => INST_TYPE::I,
        OPCODE::OP_IMM => INST_TYPE::I,
        OPCODE::JALR => INST_TYPE::I,
        OPCODE::SYSTEM => INST_TYPE::I,

        OPCODE::STORE => INST_TYPE::S,

        OPCODE::BRANCH => INST_TYPE::SB,

        OPCODE::AUIPC => INST_TYPE::U,
        OPCODE::LUI => INST_TYPE::U,

        OPCODE::JAL => INST_TYPE::UJ,

        _ => panic!("Not handled yet"),
    }
}