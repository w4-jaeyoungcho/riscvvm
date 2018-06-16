

#[derive(Debug)]
pub struct Patch {
    pub offset: u8,
    pub length: u8,
}

impl Patch {
    pub fn of_bit(offset: u8) -> Patch {
        Patch { offset: offset, length: 1 }
    }

    pub fn write(&self, base: u32, value: u32) -> u32 {
        let mask = ((1u64 << self.length) - 1) as u32;
        (base & !(mask << self.offset)) | ((value & mask) << self.offset)
    }

    pub fn write_on_ref(&self, base: &mut u32, value: u32) {
        *base = self.write(*base, value);
    }

    pub fn read(&self, base: u32) -> u32 {
        (base >> self.offset) & ((1u32 << self.length) - 1)
    }

    // read and write at the same patch
    // returns (value, updated base)
    pub fn exchange(&self, base: u32, value: u32) -> (u32, u32) {
        let v = self.read(base);
        let updated = self.write(base, value);
        (v, updated)
    }

    pub fn exchange_on_ref(&self, base: &mut u32, value: u32) -> u32 {
        let v = self.read(*base);
        self.write_on_ref(base, value);
        v
    }
}

pub const ZIMM: Patch = Patch{ offset: 15, length: 5 };
pub const OPCODE: Patch = Patch{ offset: 2, length: 5 };
pub const RD: Patch = Patch{ offset: 7, length: 5 };
pub const SHAMT: Patch = Patch{ offset: 20, length: 5 };
pub const FUNCT3: Patch = Patch{ offset: 12, length: 3 };
pub const RS1: Patch = Patch{ offset: 15, length: 5 };
pub const RS2: Patch = Patch{ offset: 20, length: 5 };
pub const FUNCT7: Patch = Patch{ offset: 25, length: 7 };
pub const FUNCT12: Patch = Patch{ offset: 20, length: 12 };
pub const IMM12: Patch = Patch{ offset: 20, length: 12 };
pub const SIMM12LO: Patch = Patch{ offset: 7, length: 5 };
pub const SIMM12HI: Patch = Patch{ offset: 25, length: 7 };
pub const IMM20: Patch = Patch{ offset: 12, length: 20 };

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_write_ref() {
        let mut base: u32 = 0x345;
        let patch = Patch { offset: 4, length: 4 };
        let expected: u32 = 0x3E5;

        patch.write_on_ref(&mut base, 0xE);
        assert_eq!(base, expected);
    }

    #[test]
    fn test_exchange_ref() {
        let mut base: u32 = 0x345;
        let patch = Patch { offset: 4, length: 4 };
        let expected: u32 = 0x315;
        let expected_v: u32 = 0x4;

        let v = patch.exchange_on_ref(&mut base, 0x1);
        assert_eq!(base, expected);
        assert_eq!(v, expected_v);
    }
}