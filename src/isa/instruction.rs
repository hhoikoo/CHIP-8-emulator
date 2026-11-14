mod error;

pub use error::UnknownOpcodeError;

use crate::isa::register::Register;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,                                      // 0x0000
    Return,                                    // 0x00EE
    Call { addr: u16 },                        // 0x2nnn
    LoadImmediate { x: Register, nn: u8 },     // 0x6xnn
    AddImmediate { x: Register, nn: u8 },      // 0x7xnn
    AddWithCarry { x: Register, y: Register }, // 0x8xy4
}

impl TryFrom<u16> for Instruction {
    type Error = UnknownOpcodeError;

    fn try_from(opcode: u16) -> Result<Self, UnknownOpcodeError> {
        match Instruction::extract_opcode_values(opcode) {
            (0, 0, 0, 0) => Ok(Self::Halt),
            (0x0, 0x0, 0xE, 0xE) => Ok(Self::Return),
            (0x2, _, _, _) => Ok(Self::Call {
                addr: Self::get_addr(opcode),
            }),
            (0x6, x, _, _) => Ok(Self::LoadImmediate {
                x: Self::register(x),
                nn: Self::get_nn(opcode),
            }),
            (0x7, x, _, _) => Ok(Self::AddImmediate {
                x: Self::register(x),
                nn: Self::get_nn(opcode),
            }),
            (0x8, x, y, 0x4) => Ok(Self::AddWithCarry {
                x: Self::register(x),
                y: Self::register(y),
            }),
            _ => Err(UnknownOpcodeError { opcode }),
        }
    }
}

impl Instruction {
    fn extract_opcode_values(opcode: u16) -> (u8, u8, u8, u8) {
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = (opcode & 0x000F) as u8;

        (c, x, y, d)
    }

    fn register(nibble: u8) -> Register {
        Register::try_from(nibble).expect("a 4-bit field always names a register")
    }

    fn get_addr(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn get_nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }
}

#[cfg(test)]
mod tests;
