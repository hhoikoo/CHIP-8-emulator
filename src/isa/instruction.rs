mod error;

pub use error::UnknownOpcodeError;

use crate::isa::{
    addr::Addr,
    opcode::{Nnn, Opcode, Xnn, Xyn},
    register::Register,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,                                      // 0x0000
    Return,                                    // 0x00EE
    Call { addr: Addr },                       // 0x2nnn
    LoadImmediate { x: Register, nn: u8 },     // 0x6xnn
    AddImmediate { x: Register, nn: u8 },      // 0x7xnn
    AddWithCarry { x: Register, y: Register }, // 0x8xy4
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

#[cfg(test)]
mod tests;
