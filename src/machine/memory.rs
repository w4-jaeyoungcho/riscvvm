use super::*;

pub const MEMORY_SIZE: usize = 0x10000;

// 64kB memory
pub struct Memory {
    pub data: Box<[u32; MEMORY_SIZE]>,
    write_probe: Option<u32>,
}

impl Memory {
    pub fn new(write_probe: Option<u32>) -> Memory {
        Memory {
            data: Box::new([0u32; MEMORY_SIZE]),
            write_probe: write_probe,
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert_eq!(data.len() % 4, 0);

        for i in 0..data.len() / 4 {
            let there = i*4;
            let v = slice_to_u32(&data[there..there + 4]);
//            println!("slice: {:?} v: {:08X}", &data[i..i+4], v);
            self.data[i] = v;
        }

    }
}

fn slice_to_u32(buf: &[u8]) -> u32 {
    let mut x = 0u32;
    x |= buf[3] as u32;
    x <<= 8;
    x |= buf[2] as u32;
    x <<= 8;
    x |= buf[1] as u32;
    x <<= 8;
    x |= buf[0] as u32;

    x
}

impl BusEnd for Memory {
    fn read_word(&mut self, addr: u32) -> u32 {
        self.data[(addr / 4) as usize]
    }

    fn write_word(&mut self, addr: u32, value: u32) {
        if let Some(probe_addr) = self.write_probe {
            if addr == probe_addr {
                println!("memory_probe: [{:X}] ({}) <- {} ", addr, self.data[(addr / 4) as usize], value);
            }
        }

        self.data[(addr / 4) as usize] = value;
    }

    fn is_interrupting(&self) -> bool {
        false
    }
}

impl Peri for Memory {

}