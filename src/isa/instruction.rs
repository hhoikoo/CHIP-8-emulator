mod error;

pub use error::UnknownOpcodeError;

use crate::isa::{addr::Addr, opcode::Opcode, register::Register};

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
            (0x0, 0x0, 0x0, 0x0) => Ok(Self::Halt),
            (0x0, 0x0, 0xE, 0xE) => Ok(Self::Return),
            (0x2, _, _, _) => Ok(Self::Call { addr: opcode.nnn() }),
            (0x6, _, _, _) => Ok(Self::LoadImmediate {
                x: opcode.x(),
                nn: opcode.nn(),
            }),
            (0x7, _, _, _) => Ok(Self::AddImmediate {
                x: opcode.x(),
                nn: opcode.nn(),
            }),
            (0x8, _, _, 0x4) => Ok(Self::AddWithCarry {
                x: opcode.x(),
                y: opcode.y(),
            }),
            _ => Err(UnknownOpcodeError { opcode }),
        }
    }
}

#[cfg(test)]
mod tests;
