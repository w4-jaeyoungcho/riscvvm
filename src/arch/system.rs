
// In privileged specification v1.10, Hypervisor mode things became reserved

// csr is 12 bits
// The upper 4 bits encode accessibility
// upper 2 bits for read/write (0b11 for read-only), lower 2 bits for the lowest level that can access the CSR
// Access to non-existent CSR raises illegal instruction exception
// Inappropriate access also raises illegal instruction exception

enum_from_primitive! {
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum CSR {
    //// User level
    // User counter/timers

    CYCLE = 0xC00,
    TIME = 0xC01,
    INSTRET = 0xC02,

    CYCLEH = 0xC80,
    TIMEH = 0xC81,
    INSTRETH = 0xC82,

    // User trap setup
    USTATUS = 0x000,
    UIE = 0x004,
    UTVEC = 0x005,

    // User trap handling
    USTRATCH = 0x040,
    UEPC = 0x041,
    UCAUSE = 0x042,
    UTVAL = 0x043,
    UIP = 0x044,

    // User floating point CSRs
    FFLAGS = 0x001,
    FRM = 0x002,
    FCSR = 0x003,

    //// Supervisor level

    // Supervisor trap setup
    SSTATUS = 0x100,
    SEDELEG = 0x102,
    SIDELEG = 0x103,
    SIE = 0x104,
    STVEC = 0x105,

    // Supervisor trap handling
    SSCRATCH = 0x140,
    SEPC = 0x141,
    SCAUSE = 0x142,
    STVAL = 0x143,
    SIP = 0x144,

    // Supervisor protection and translation
    SPTBR = 0x180,


    //// Machine level

    // Machine information registers
    MVENDORID = 0xF11,
    MARCHID = 0xF12,
    MIMPID = 0xF13,
    MHARTID = 0xF14,

    // Machine trap setup
    MSTATUS = 0x300,
    MISA = 0x301,
    MEDELEG = 0x302,
    MIDELEG = 0x303,
    MIE = 0x304,
    MTVEC = 0x305,

    // Machine trap handling
    MSCRATCH = 0x340,
    MEPC = 0x341,
    MCAUSE = 0x342,
    MTVAL = 0x343,
    MIP = 0x344,

    // Machine protection and translation
    MBASE = 0x380,
    MBOUND = 0x381,
    MIBASE = 0x382,
    MIBOUND = 0x383,
    MDBASE = 0x384,
    MDBOUND = 0x385,

    // Machine counter/timers
    MCYCLE = 0xB00,
    MINSTRET = 0xB02,
    MCYCLEH = 0xB80,
    MINSTRETH = 0xB82,

    // Machine counter setup
    MUCOUNTEREN = 0x320,
    MSCOUNTEREN = 0x321,

    // Debug/trace registers (shared with debug mode)

    // Debug mode registers
}
}

const CSRS: &'static [(&'static str, u32)] = &[
    ("cycle", CSR::CYCLE as u32),
    ("time", CSR::TIME as u32),
    ("instret", CSR::INSTRET as u32),

    ("cycleh", CSR::CYCLEH as u32),
    ("timeh", CSR::TIMEH as u32),
    ("instreth", CSR::INSTRETH as u32),

    // User trap setup
    ("ustatus", CSR::USTATUS as u32),
    ("uie", CSR::UIE as u32),
    ("utvec", CSR::UTVEC as u32),

    // User trap handling
    ("ustratch", CSR::USTRATCH as u32),
    ("uepc", CSR::UEPC as u32),
    ("ucause", CSR::UCAUSE as u32),
    ("utval", CSR::UTVAL as u32),
    ("uip", CSR::UIP as u32),

    // User floating point CSRs
    ("fflags", CSR::FFLAGS as u32),
    ("frm", CSR::FRM as u32),
    ("fcsr", CSR::FCSR as u32),

    //// Supervisor level

    // Supervisor trap setup
    ("sstatus", CSR::SSTATUS as u32),
    ("sedeleg", CSR::SEDELEG as u32),
    ("sideleg", CSR::SIDELEG as u32),
    ("sie", CSR::SIE as u32),
    ("stvec", CSR::STVEC as u32),

    // Supervisor trap handling
    ("sscratch", CSR::SSCRATCH as u32),
    ("sepc", CSR::SEPC as u32),
    ("scause", CSR::SCAUSE as u32),
    ("stval", CSR::STVAL as u32),
    ("sip", CSR::SIP as u32),

    // Supervisor protection and translation
    ("sptbr", CSR::SPTBR as u32),


    //// Machine level

    // Machine information registers
    ("mvendorid", CSR::MVENDORID as u32),
    ("marchid", CSR::MARCHID as u32),
    ("mimpid", CSR::MIMPID as u32),
    ("mhartid", CSR::MHARTID as u32),

    // Machine trap setup
    ("mstatus", CSR::MSTATUS as u32),
    ("misa", CSR::MISA as u32),
    ("medeleg", CSR::MEDELEG as u32),
    ("mideleg", CSR::MIDELEG as u32),
    ("mie", CSR::MIE as u32),
    ("mtvec", CSR::MTVEC as u32),

    // Machine trap handling
    ("mscratch", CSR::MSCRATCH as u32),
    ("mepc", CSR::MEPC as u32),
    ("mcause", CSR::MCAUSE as u32),
    ("mtval", CSR::MTVAL as u32),
    ("mip", CSR::MIP as u32),
];

pub fn csr_name(value: u32) -> Option<&'static str> {
    for &(name, v) in CSRS {
        if v == value {
            return Some(name);
        }
    }

    None
}

pub fn csr_value(name: &str) -> Option<u32> {
    for &(n, v) in CSRS {
        if n == name {
            return Some(v);
        }
    }

    None
}

// General value/encoding
pub const USER: u8 = 0;
pub const SUPERVISOR: u8 = 1;
//pub const HYPERVISOR: u8 = 2;
pub const MACHINE: u8 = 3;

pub const LEVELS: [u8; 3] = [0, 1, 3];

// CSRs Offsets

// misa
pub const BASE: u8 = 30;

// mstatus
pub const IE_BASE: u8 = 0;

pub const PIE_BASE: u8 = 4;

pub const SPP: u8 = 8;
//pub const HPP: u8 = 9;
pub const MPP: u8 = 11;

pub const VM: u8 = 24;

// mip
pub const USIP: u8 = 0;
pub const SSIP: u8 = 1;

pub const MSIP: u8 = 3;

pub const UTIP: u8 = 4;
pub const STIP: u8 = 5;
pub const MTIP: u8 = 7;

pub const UEIP: u8 = 8;
pub const SEIP: u8 = 9;
pub const MEIP: u8 = 11;

// mie
pub const USIE: u8 = 0;
pub const SSIE: u8 = 1;

pub const MSIE: u8 = 3;

pub const UTIE: u8 = 4;
pub const STIE: u8 = 5;
pub const MTIE: u8 = 7;

pub const UEIE: u8 = 8;
pub const SEIE: u8 = 9;
pub const MEIE: u8 = 11;

// mcause
pub const EXCEPTION_CODE: u8 = 0;
pub const INTERRUPT: u8 = 31;


// Exception codes in mcause
pub const SOFTWARE_INTERRUPT_BASE: u32 = 0;
pub const TIMER_INTERRUPT_BASE: u32 = 4;
pub const EXTERNAL_INTERRUPT_BASE: u32 = 8;

pub const INSTRUCTION_ADDRESS_MISALIGNED: u32 = 0;
pub const INSTRUCTION_ACCESS_FAULT: u32 = 1;
pub const ILLEGAL_INSTRUCTION: u32 = 2;
pub const BREAKPOINT: u32 = 3;
pub const LOAD_ADDRESS_MISALIGNED: u32 = 4;
pub const LOAD_ACCESS_FAULT: u32 = 5;
pub const STORE_ADDRESS_MISALIGNED: u32 = 6;
pub const STORE_ACCESS_FAULT: u32 = 7;
pub const ECALL_BASE: u32 = 8;


// Virtualization management scheme
pub const MBARE: u32 = 0;
pub const MBB: u32 = 1;
pub const MBBID: u32 = 2;

pub const SV32: u32 = 8;
