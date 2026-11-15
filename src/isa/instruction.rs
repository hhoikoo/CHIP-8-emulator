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
    Return,
    Jump { addr: Addr },
    JumpWithOffset { addr: Addr },
    Call { addr: Addr },
    LoadImmediate { x: Register, nn: u8 },
    AddImmediate { x: Register, nn: u8 },
    AddWithCarry { x: Register, y: Register },
}

const RETURN: [Nibble; 4] = Opcode::new(0x00EE).nibbles();
const JUMP: Nibble = Nibble::expect_new(0x1);
const CALL: Nibble = Nibble::expect_new(0x2);
const LOAD_IMMEDIATE: Nibble = Nibble::expect_new(0x6);
const ADD_IMMEDIATE: Nibble = Nibble::expect_new(0x7);
const REGISTER_OP: Nibble = Nibble::expect_new(0x8);
const ADD_WITH_CARRY: Nibble = Nibble::expect_new(0x4);
const JUMP_WITH_OFFSET: Nibble = Nibble::expect_new(0xB);

impl TryFrom<Opcode> for Instruction {
    type Error = UnknownOpcodeError;

    fn try_from(opcode: Opcode) -> Result<Self, UnknownOpcodeError> {
        let Nnn { addr, .. } = opcode.into();
        let Xnn { x, nn, .. } = opcode.into();
        let Xyn { y, .. } = opcode.into();

        match opcode.nibbles() {
            RETURN => Ok(Self::Return),
            [JUMP, ..] => Ok(Self::Jump { addr }),
            [CALL, ..] => Ok(Self::Call { addr }),
            [LOAD_IMMEDIATE, ..] => Ok(Self::LoadImmediate { x, nn }),
            [ADD_IMMEDIATE, ..] => Ok(Self::AddImmediate { x, nn }),
            [REGISTER_OP, _, _, ADD_WITH_CARRY] => Ok(Self::AddWithCarry { x, y }),
            [JUMP_WITH_OFFSET, ..] => Ok(Self::JumpWithOffset { addr }),
            _ => Err(UnknownOpcodeError { opcode }),
        }
    }
}

fn nnn(prefix: Nibble, addr: Addr) -> Opcode {
    Nnn { prefix, addr }.into()
}

fn xnn(prefix: Nibble, x: Register, nn: u8) -> Opcode {
    Xnn { prefix, x, nn }.into()
}

fn xyn(prefix: Nibble, x: Register, y: Register, n: Nibble) -> Opcode {
    Xyn { prefix, x, y, n }.into()
}

impl Instruction {
    pub fn encode(self) -> Opcode {
        match self {
            Self::Return => Opcode::from(RETURN),
            Self::Jump { addr } => nnn(JUMP, addr),
            Self::Call { addr } => nnn(CALL, addr),
            Self::LoadImmediate { x, nn } => xnn(LOAD_IMMEDIATE, x, nn),
            Self::AddImmediate { x, nn } => xnn(ADD_IMMEDIATE, x, nn),
            Self::AddWithCarry { x, y } => xyn(REGISTER_OP, x, y, ADD_WITH_CARRY),
            Self::JumpWithOffset { addr } => nnn(JUMP_WITH_OFFSET, addr),
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
