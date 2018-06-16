
// Dubious use of Rust enum
#![allow(non_camel_case_types)]

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum OPCODE {
    LOAD = 0,
    LOAD_FP,
    CUSTOM_0, // _
    MISC_MEM,
    OP_IMM,
    AUIPC,
    OP_IMM_32,
    _48B, // _
    STORE,
    STORE_FP,
    CUSTOM_1,
    AMO,
    OP,
    LUI,
    OP_32,
    _64B, // _
    MADD,
    MSUB,
    NMSUB,
    NMADD,
    OP_FP,
    _RESERVED0,
    _CUSTOM_2,
    _48B_1,
    BRANCH,
    JALR,
    _RESERVED1,
    JAL,
    SYSTEM,
    _RESERVED2,
    _CUSTOM_3,
    _80b,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_LOAD {
    LB = 0,
    LH,
    LW,
    LBU,
    LHU,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_MISC_MEM {
    FENCE = 0,
    FENCE_I,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_OP_IMM {
    ADDI = 0,

    SLLI,

    SLTI,
    SLTIU,
    XORI,

    SRLI,

    ORI,
    ANDI,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT7_OP_IMM {
    BASE = 0,
    ALT = 0x20,
}
}


enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_OP_IMM_ALT {
    SRAI = 5,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_STORE {
    SB = 0,
    SH,
    SW,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_OP {
    ADD = 0,
    SLL,
    SLT,
    SLTU,
    XOR,
    SRL,
    OR,
    AND,
}
}


enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT7_OP {
    BASE = 0,
    ALT = 0x20,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_OP_ALT {
    SUB = 0,
    SRA = 5,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_BRANCH {
    BEQ = 0,
    BNE,
    BLT = 4,
    BGE,
    BLTU,
    BGEU,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_JALR {
    JALR = 0,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT3_SYSTEM {
    PRIV = 0,
    CSRRW,
    CSRRS,
    CSRRC,
    CSRRWI = 5,
    CSRRSI,
    CSRRCI,
}
}

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum FUNCT12_PRIV {
    ECALL = 0,
    EBREAK,

    URET = 0x002,
    SRET = 0x102,
//    HRET = 0x202,
    MRET = 0x302,
    WFI = 0x105,

    SFENCEVM = 0x104,
}
}
