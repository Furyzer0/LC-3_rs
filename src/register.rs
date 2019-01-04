use std::ops::{Index, IndexMut};
use enum_primitive_derive::Primitive;
use num_traits::ToPrimitive;

pub struct Registers([u16; 10]);

const PC: usize = 8;
const COND: usize = 9;
const PC_START: u16 = 0x3000;

#[repr(u16)]
#[derive(Primitive)]
enum CondFlag {
    POS = 0b_001,
    ZRO = 0b_010,
    NEG = 0b_100,
}

impl Registers {
    pub fn new() -> Self {
        let mut arr = [0u16; 10];
        arr[PC] = PC_START;
        Registers(arr)
    }

    pub fn update_cond_flags(&mut self, reg: usize) {
        if self[reg] == 0 {
            self[COND] = CondFlag::ZRO.to_u16().unwrap()
        } else if (self[reg] >> 15) == 1 {
            self[COND] = CondFlag::NEG.to_u16().unwrap()
        } else {
            self[COND] = CondFlag::POS.to_u16().unwrap()
        }
    }

    pub fn pc_reg(&mut self) -> &mut u16 {
        &mut self[PC]
    }
}

impl Index<usize> for Registers {
    type Output = u16;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    } 
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    } 
}