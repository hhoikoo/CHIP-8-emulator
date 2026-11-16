mod error;

pub use error::RegisterOutOfRange;

use std::ops::{Index, IndexMut};

use crate::isa::nibble::Nibble;

pub const NUM_REGISTERS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xA,
    VB = 0xB,
    VC = 0xC,
    VD = 0xD,
    VE = 0xE,
    VF = 0xF,
}

impl Register {
    pub const fn new(value: u8) -> Result<Register, RegisterOutOfRange> {
        match value {
            0x0 => Ok(Register::V0),
            0x1 => Ok(Register::V1),
            0x2 => Ok(Register::V2),
            0x3 => Ok(Register::V3),
            0x4 => Ok(Register::V4),
            0x5 => Ok(Register::V5),
            0x6 => Ok(Register::V6),
            0x7 => Ok(Register::V7),
            0x8 => Ok(Register::V8),
            0x9 => Ok(Register::V9),
            0xA => Ok(Register::VA),
            0xB => Ok(Register::VB),
            0xC => Ok(Register::VC),
            0xD => Ok(Register::VD),
            0xE => Ok(Register::VE),
            0xF => Ok(Register::VF),
            _ => Err(RegisterOutOfRange { value }),
        }
    }
}

impl TryFrom<u8> for Register {
    type Error = RegisterOutOfRange;

    fn try_from(value: u8) -> Result<Self, RegisterOutOfRange> {
        Register::new(value)
    }
}

impl From<Nibble> for Register {
    fn from(nibble: Nibble) -> Register {
        Register::new(nibble.get()).expect("every nibble names a register")
    }
}

impl From<Register> for u16 {
    fn from(register: Register) -> u16 {
        register as u16
    }
}

impl From<Register> for usize {
    fn from(register: Register) -> usize {
        register as usize
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Registers([u8; NUM_REGISTERS]);

impl Registers {
    pub fn up_to(&self, last: Register) -> &[u8] {
        &self.0[..=usize::from(last)]
    }

    pub fn up_to_mut(&mut self, last: Register) -> &mut [u8] {
        &mut self.0[..=usize::from(last)]
    }
}

impl Index<Register> for Registers {
    type Output = u8;

    fn index(&self, index: Register) -> &u8 {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut u8 {
        &mut self.0[usize::from(index)]
    }
}

#[cfg(test)]
mod tests;
