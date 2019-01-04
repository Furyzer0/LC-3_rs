use std::ops::{Index, IndexMut};
use std::fs::File;
use std::io::{self, Read};

pub struct Memory([u16; Memory::MAX_MEMORY]);

extern "C" {
    fn check_key() -> libc::uint16_t;
}

impl Memory {
    const MAX_MEMORY: usize = std::u16::MAX as usize;
    const MR_KBSR: usize = 0xFE00;
    const MR_KBDR: usize = 0xFE02;
    
    pub fn new() -> Self {
        Memory([0u16; Self::MAX_MEMORY])
    }

    pub fn read(&mut self, adress: u16) -> u16 {
        unsafe {
            if adress == Self::MR_KBSR as u16 {
                if check_key() != 0 {
                    self[Self::MR_KBSR] = 1 << 15;
                    self[Self::MR_KBDR] = libc::getchar() as u16;
                } else {
                    self[Self::MR_KBSR] = 0;
                }
            }
            self[adress as usize]
        }
    }

    pub fn write(&mut self, adress: u16, value: u16) {
        self[adress as usize] = value;
    }

    pub fn load_file(&mut self, file: &mut File) -> io::Result<()> {
        let mut origin = [0u8; 2];
        file.read_exact(&mut origin)?;
        let origin: u16 = swap16(&origin);
        println!("origin: {:X}", origin);
        let mut vec = Vec::new();
        file.read_to_end(&mut vec)?;
        for i in 0..vec.len() / 2 {
            self[origin as usize + i] = swap16(&vec[2 * i..]);
        }
        Ok(())
    }
}

// Because LC-3 files are little endian
#[inline]
fn swap16(arr: &[u8]) -> u16 {
    ((arr[0] as u16) << 8) | (arr[1] as u16)
}

impl Index<usize> for Memory {
    type Output = u16;

    fn index(&self, idx: usize) -> &u16 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, idx: usize) -> &mut u16 {
        &mut self.0[idx]
    }
}