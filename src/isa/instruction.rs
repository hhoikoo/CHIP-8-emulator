mod error;

pub use error::UnknownOpcodeError;

use crate::isa::{
    addr::Addr,
    nibble::Nibble,
    opcode::{Nnn, Opcode, Xnn, Xyn},
    register::Register,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    Return,
    Call { addr: Addr },
    LoadImmediate { x: Register, nn: u8 },
    AddImmediate { x: Register, nn: u8 },
    AddWithCarry { x: Register, y: Register },
}

impl TryFrom<Opcode> for Instruction {
    type Error = UnknownOpcodeError;

    fn try_from(opcode: Opcode) -> Result<Self, UnknownOpcodeError> {
        match opcode.nibbles() {
            [0x0, 0x0, 0x0, 0x0] => Ok(Self::Halt),
            [0x0, 0x0, 0xE, 0xE] => Ok(Self::Return),
            [0x2, ..] => {
                let Nnn { addr, .. } = opcode.into();
                Ok(Self::Call { addr })
            }
            [0x6, ..] => {
                let Xnn { x, nn, .. } = opcode.into();
                Ok(Self::LoadImmediate { x, nn })
            }
            [0x7, ..] => {
                let Xnn { x, nn, .. } = opcode.into();
                Ok(Self::AddImmediate { x, nn })
            }
            [0x8, _, _, 0x4] => {
                let Xyn { x, y, .. } = opcode.into();
                Ok(Self::AddWithCarry { x, y })
            }
            _ => Err(UnknownOpcodeError { opcode }),
        }
    }
}

impl Instruction {
    pub fn encode(self) -> Opcode {
        match self {
            Self::Halt => Opcode::new(0x0000),
            Self::Return => Opcode::new(0x00EE),
            Self::Call { addr } => Nnn {
                prefix: Nibble::expect_new(0x2),
                addr,
            }
            .into(),
            Self::LoadImmediate { x, nn } => Xnn {
                prefix: Nibble::expect_new(0x6),
                x,
                nn,
            }
            .into(),
            Self::AddImmediate { x, nn } => Xnn {
                prefix: Nibble::expect_new(0x7),
                x,
                nn,
            }
            .into(),
            Self::AddWithCarry { x, y } => Xyn {
                prefix: Nibble::expect_new(0x8),
                x,
                y,
                n: Nibble::expect_new(0x4),
            }
            .into(),
        }
    }
}

impl From<Instruction> for Opcode {
    fn from(instruction: Instruction) -> Opcode {
        instruction.encode()
    }
}

#[cfg(test)]
mod tests;
