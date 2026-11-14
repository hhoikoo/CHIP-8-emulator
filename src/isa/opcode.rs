use std::fmt;

use crate::isa::{addr::Addr, register::Register};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Opcode(u16);

impl Opcode {
    pub const fn new(value: u16) -> Opcode {
        Opcode(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn nibbles(self) -> (u8, u8, u8, u8) {
        let value = self.get();
        (
            (value >> 12) as u8 & 0xF,
            (value >> 8) as u8 & 0xF,
            (value >> 4) as u8 & 0xF,
            value as u8 & 0xF,
        )
    }

    pub fn x(self) -> Register {
        let (_, x, _, _) = self.nibbles();
        Register::try_from(x).expect("a 4-bit field always names a register")
    }

    pub fn y(self) -> Register {
        let (_, _, y, _) = self.nibbles();
        Register::try_from(y).expect("a 4-bit field always names a register")
    }

    pub const fn n(self) -> u8 {
        let (_, _, _, n) = self.nibbles();
        n
    }

    pub const fn nn(self) -> u8 {
        let (_, _, y, n) = self.nibbles();
        y << 4 | n
    }

    pub fn nnn(self) -> Addr {
        let (_, x, y, n) = self.nibbles();
        let nnn = (x as u16) << 8 | (y as u16) << 4 | n as u16;
        Addr::new(nnn).expect("a 12-bit field always fits in an Addr")
    }
}

impl From<u16> for Opcode {
    fn from(value: u16) -> Opcode {
        Opcode(value)
    }
}

impl From<Opcode> for u16 {
    fn from(opcode: Opcode) -> u16 {
        opcode.get()
    }
}

/// Lets opcodes use hex format specs like `{opcode:#06x}`.
impl fmt::LowerHex for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests;
