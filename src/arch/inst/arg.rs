
use ::encode::patch::*;

#[derive(Debug)]
pub struct Mapping {
    pub source_offset: u8,
    pub dest_offset: u8,
    pub length: u8,
}

impl Mapping {
    pub fn forward_write(&self, base: u32, src: u32) -> u32 {
        let v = read(src, self.source_offset, self.length);
        write(base, self.dest_offset, self.length, v)
    }

    pub fn backward_write(&self, base: u32, src: u32) -> u32 {
        let v = read(src, self.dest_offset, self.length);
        write(base, self.source_offset, self.length, v)
    }
}

#[derive(Debug)]
pub struct Mappings(pub &'static [Mapping]);

impl Mappings {
    pub fn forward_write(&self, mut base: u32, src: u32) -> u32 {
        self.0.iter().fold(base, |b, m| m.forward_write(b, src))
    }

    pub fn backward_write(&self, mut base: u32, src: u32) -> u32 {
        self.0.iter().fold(base, |b, m| m.backward_write(b, src))
    }
}

macro_rules! of_register {
    ($e:expr) => (
        Arg {
            mappings: Mappings(&[Mapping {
                source_offset: 0,
                dest_offset: $e,
                length: 5,
            }]),
            arg_type: ArgType::Register,
            sign_bit: None,
        }
    )
}

macro_rules! of_patch {
    ($source_offset:expr, $dest_offset:expr, $length:expr) => (
        Mappings(&[Mapping{
            source_offset: $source_offset,
            dest_offset: $dest_offset,
            length: $length,
        }])
    )
}

pub const RD: Arg = of_register!(7);
pub const RS1: Arg = of_register!(15);
pub const RS2: Arg = of_register!(20);
pub const SHAMT: Arg = Arg{
    mappings: of_patch!(0, 20, 5),
    arg_type: ArgType::General,
    sign_bit: None,
};
pub const IMM12: Arg = Arg{
    mappings: of_patch!(0, 20, 12),
    arg_type: ArgType::General,
    sign_bit: Some(11),
};
pub const SIMM12: Arg = Arg{
    mappings: Mappings(&[
        Mapping{ source_offset: 0, dest_offset: 7, length: 5},
        Mapping{ source_offset: 5, dest_offset: 25, length: 7},
    ]),
    arg_type: ArgType::General,
    sign_bit: Some(11),
};
pub const BIMM12: Arg = Arg{
    mappings: Mappings(&[
        Mapping{ source_offset: 1, dest_offset: 8, length: 4 },
        Mapping{ source_offset: 5, dest_offset: 25, length: 6 },
        Mapping{ source_offset: 11, dest_offset: 7, length: 1 },
        Mapping{ source_offset: 12, dest_offset: 31, length: 1 },
    ]),
    arg_type: ArgType::Address,
    sign_bit: Some(12),
};
pub const IMM20: Arg = Arg{
    mappings: of_patch!(12, 12, 20),
    arg_type: ArgType::General,
    sign_bit: None,
};
pub const JIMM20: Arg = Arg{
    mappings: Mappings(&[
        Mapping{ source_offset: 1, dest_offset: 21, length: 10 },
        Mapping{ source_offset: 11, dest_offset: 20, length: 1 },
        Mapping{ source_offset: 12, dest_offset: 12, length: 8 },
        Mapping{ source_offset: 20, dest_offset: 31, length: 1 },
    ]),
    arg_type: ArgType::Address,
    sign_bit: Some(20),
};
pub const CSR: Arg = Arg{
    mappings: of_patch!(0, 20, 12),
    arg_type: ArgType::Csr,
    sign_bit: None,
};
pub const ZIMM: Arg = Arg{
    mappings: of_patch!(0, 15, 5),
    arg_type: ArgType::General,
    sign_bit: Some(4),
};

#[derive(Debug)]
pub enum ArgType {
    Register,
    Address, // all relative
    General,
    Csr,
}

#[derive(Debug)]
pub struct Arg {
    pub mappings: Mappings,
    pub arg_type: ArgType,
    pub sign_bit: Option<u8>,
}

impl Arg {
    pub fn length(&self) -> u32 {
        self.mappings.0.iter().fold(0u32, |s, m| m.length as u32 + s)
    }

    pub fn read(&self, src: u32) -> u32 {
        // map and sign extend if signed
        let v = self.mappings.backward_write(0, src);
        if let Some(sign_bit) = self.sign_bit {
            sign_extend(v, sign_bit)
        } else {
            v
        }
    }
}