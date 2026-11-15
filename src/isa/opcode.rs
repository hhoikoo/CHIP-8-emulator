use std::fmt;

use crate::isa::{addr::Addr, nibble::Nibble, register::Register};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Opcode(u16);

impl Opcode {
    pub const fn new(value: u16) -> Opcode {
        Opcode(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn nibbles(self) -> [u8; 4] {
        let value = self.get();
        [
            (value >> 12) as u8 & 0xF,
            (value >> 8) as u8 & 0xF,
            (value >> 4) as u8 & 0xF,
            value as u8 & 0xF,
        ]
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

impl fmt::LowerHex for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nnn {
    pub prefix: Nibble,
    pub addr: Addr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xnn {
    pub prefix: Nibble,
    pub x: Register,
    pub nn: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xyn {
    pub prefix: Nibble,
    pub x: Register,
    pub y: Register,
    pub n: Nibble,
}

impl From<Opcode> for Nnn {
    fn from(opcode: Opcode) -> Nnn {
        let [prefix, x, y, n] = opcode.nibbles().map(Nibble::expect_new);
        let addr = u16::from(x) << 8 | u16::from(y) << 4 | u16::from(n);
        Nnn {
            prefix,
            addr: Addr::new(addr).expect("a 12-bit field always fits in an Addr"),
        }
    }
}

impl From<Nnn> for Opcode {
    fn from(Nnn { prefix, addr }: Nnn) -> Opcode {
        Opcode::new(u16::from(prefix) << 12 | addr.get())
    }
}

impl From<Opcode> for Xnn {
    fn from(opcode: Opcode) -> Xnn {
        let [prefix, x, y, n] = opcode.nibbles().map(Nibble::expect_new);
        Xnn {
            prefix,
            x: Register::from(x),
            nn: u8::from(y) << 4 | u8::from(n),
        }
    }
}

impl From<Xnn> for Opcode {
    fn from(Xnn { prefix, x, nn }: Xnn) -> Opcode {
        Opcode::new(u16::from(prefix) << 12 | u16::from(x) << 8 | u16::from(nn))
    }
}

impl From<Opcode> for Xyn {
    fn from(opcode: Opcode) -> Xyn {
        let [prefix, x, y, n] = opcode.nibbles().map(Nibble::expect_new);
        Xyn {
            prefix,
            x: Register::from(x),
            y: Register::from(y),
            n,
        }
    }
}

impl From<Xyn> for Opcode {
    fn from(Xyn { prefix, x, y, n }: Xyn) -> Opcode {
        Opcode::new(u16::from(prefix) << 12 | u16::from(x) << 8 | u16::from(y) << 4 | u16::from(n))
    }
}

#[cfg(test)]
mod tests;
