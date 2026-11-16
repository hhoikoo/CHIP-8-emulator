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
    ClearScreen,
    Return,
    Jump { addr: Addr },
    JumpWithOffset { addr: Addr },
    Call { addr: Addr },
    LoadImmediate { x: Register, nn: u8 },
    LoadRegister { x: Register, y: Register },
    LoadIndex { addr: Addr },
    AddImmediate { x: Register, nn: u8 },
    AddWithCarry { x: Register, y: Register },
    AddToIndex { x: Register },
    Or { x: Register, y: Register },
    And { x: Register, y: Register },
    Xor { x: Register, y: Register },
    SubtractWithBorrow { x: Register, y: Register },
    SubtractReversed { x: Register, y: Register },
    ShiftRight { x: Register, y: Register },
    ShiftLeft { x: Register, y: Register },
    SkipIfEqualImmediate { x: Register, nn: u8 },
    SkipIfNotEqualImmediate { x: Register, nn: u8 },
    SkipIfEqual { x: Register, y: Register },
    SkipIfNotEqual { x: Register, y: Register },
    StoreBcd { x: Register },
    StoreRegisters { x: Register },
    LoadRegisters { x: Register },
    LoadFontCharacter { x: Register },
    Draw { x: Register, y: Register, n: Nibble },
}

const CLEAR_SCREEN: [Nibble; 4] = Opcode::new(0x00E0).nibbles();
const RETURN: [Nibble; 4] = Opcode::new(0x00EE).nibbles();
const JUMP: Nibble = Nibble::expect_new(0x1);
const CALL: Nibble = Nibble::expect_new(0x2);
const LOAD_IMMEDIATE: Nibble = Nibble::expect_new(0x6);
const LOAD_REGISTER: Nibble = Nibble::expect_new(0x0);
const ADD_IMMEDIATE: Nibble = Nibble::expect_new(0x7);
const REGISTER_OP: Nibble = Nibble::expect_new(0x8);
const ADD_WITH_CARRY: Nibble = Nibble::expect_new(0x4);
const OR: Nibble = Nibble::expect_new(0x1);
const AND: Nibble = Nibble::expect_new(0x2);
const XOR: Nibble = Nibble::expect_new(0x3);
const SUBTRACT_WITH_BORROW: Nibble = Nibble::expect_new(0x5);
const SUBTRACT_REVERSED: Nibble = Nibble::expect_new(0x7);
const SHIFT_RIGHT: Nibble = Nibble::expect_new(0x6);
const SHIFT_LEFT: Nibble = Nibble::expect_new(0xE);
const JUMP_WITH_OFFSET: Nibble = Nibble::expect_new(0xB);
const LOAD_INDEX: Nibble = Nibble::expect_new(0xA);
const SKIP_IF_EQUAL_IMMEDIATE: Nibble = Nibble::expect_new(0x3);
const SKIP_IF_NOT_EQUAL_IMMEDIATE: Nibble = Nibble::expect_new(0x4);
const SKIP_IF_EQUAL: Nibble = Nibble::expect_new(0x5);
const SKIP_IF_NOT_EQUAL: Nibble = Nibble::expect_new(0x9);
const DRAW: Nibble = Nibble::expect_new(0xD);
const MISC: Nibble = Nibble::expect_new(0xF);
const LAST_ZERO: Nibble = Nibble::expect_new(0x0);
const ADD_TO_INDEX: u8 = 0x1E;
const STORE_BCD: u8 = 0x33;
const STORE_REGISTERS: u8 = 0x55;
const LOAD_REGISTERS: u8 = 0x65;
const LOAD_FONT_CHARACTER: u8 = 0x29;

impl TryFrom<Opcode> for Instruction {
    type Error = UnknownOpcodeError;

    fn try_from(opcode: Opcode) -> Result<Self, UnknownOpcodeError> {
        let Nnn { addr, .. } = opcode.into();
        let Xnn { x, nn, .. } = opcode.into();
        let Xyn { y, n, .. } = opcode.into();

        match opcode.nibbles() {
            CLEAR_SCREEN => Ok(Self::ClearScreen),
            RETURN => Ok(Self::Return),
            [JUMP, ..] => Ok(Self::Jump { addr }),
            [CALL, ..] => Ok(Self::Call { addr }),
            [LOAD_IMMEDIATE, ..] => Ok(Self::LoadImmediate { x, nn }),
            [REGISTER_OP, ..] => match n {
                LOAD_REGISTER => Ok(Self::LoadRegister { x, y }),
                ADD_WITH_CARRY => Ok(Self::AddWithCarry { x, y }),
                OR => Ok(Self::Or { x, y }),
                AND => Ok(Self::And { x, y }),
                XOR => Ok(Self::Xor { x, y }),
                SUBTRACT_WITH_BORROW => Ok(Self::SubtractWithBorrow { x, y }),
                SUBTRACT_REVERSED => Ok(Self::SubtractReversed { x, y }),
                SHIFT_RIGHT => Ok(Self::ShiftRight { x, y }),
                SHIFT_LEFT => Ok(Self::ShiftLeft { x, y }),
                _ => Err(UnknownOpcodeError { opcode }),
            },
            [LOAD_INDEX, ..] => Ok(Self::LoadIndex { addr }),
            [ADD_IMMEDIATE, ..] => Ok(Self::AddImmediate { x, nn }),
            [JUMP_WITH_OFFSET, ..] => Ok(Self::JumpWithOffset { addr }),
            [SKIP_IF_EQUAL_IMMEDIATE, ..] => Ok(Self::SkipIfEqualImmediate { x, nn }),
            [SKIP_IF_NOT_EQUAL_IMMEDIATE, ..] => Ok(Self::SkipIfNotEqualImmediate { x, nn }),
            [SKIP_IF_EQUAL, _, _, LAST_ZERO] => Ok(Self::SkipIfEqual { x, y }),
            [SKIP_IF_NOT_EQUAL, _, _, LAST_ZERO] => Ok(Self::SkipIfNotEqual { x, y }),
            [DRAW, ..] => Ok(Self::Draw { x, y, n }),
            [MISC, ..] => match nn {
                ADD_TO_INDEX => Ok(Self::AddToIndex { x }),
                STORE_BCD => Ok(Self::StoreBcd { x }),
                STORE_REGISTERS => Ok(Self::StoreRegisters { x }),
                LOAD_REGISTERS => Ok(Self::LoadRegisters { x }),
                LOAD_FONT_CHARACTER => Ok(Self::LoadFontCharacter { x }),
                _ => Err(UnknownOpcodeError { opcode }),
            },
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
            Self::ClearScreen => Opcode::from(CLEAR_SCREEN),
            Self::Return => Opcode::from(RETURN),
            Self::Jump { addr } => nnn(JUMP, addr),
            Self::Call { addr } => nnn(CALL, addr),
            Self::LoadImmediate { x, nn } => xnn(LOAD_IMMEDIATE, x, nn),
            Self::LoadRegister { x, y } => xyn(REGISTER_OP, x, y, LOAD_REGISTER),
            Self::LoadIndex { addr } => nnn(LOAD_INDEX, addr),
            Self::AddImmediate { x, nn } => xnn(ADD_IMMEDIATE, x, nn),
            Self::AddWithCarry { x, y } => xyn(REGISTER_OP, x, y, ADD_WITH_CARRY),
            Self::AddToIndex { x } => xnn(MISC, x, ADD_TO_INDEX),
            Self::Or { x, y } => xyn(REGISTER_OP, x, y, OR),
            Self::And { x, y } => xyn(REGISTER_OP, x, y, AND),
            Self::Xor { x, y } => xyn(REGISTER_OP, x, y, XOR),
            Self::SubtractWithBorrow { x, y } => xyn(REGISTER_OP, x, y, SUBTRACT_WITH_BORROW),
            Self::SubtractReversed { x, y } => xyn(REGISTER_OP, x, y, SUBTRACT_REVERSED),
            Self::ShiftRight { x, y } => xyn(REGISTER_OP, x, y, SHIFT_RIGHT),
            Self::ShiftLeft { x, y } => xyn(REGISTER_OP, x, y, SHIFT_LEFT),
            Self::JumpWithOffset { addr } => nnn(JUMP_WITH_OFFSET, addr),
            Self::SkipIfEqualImmediate { x, nn } => xnn(SKIP_IF_EQUAL_IMMEDIATE, x, nn),
            Self::SkipIfNotEqualImmediate { x, nn } => xnn(SKIP_IF_NOT_EQUAL_IMMEDIATE, x, nn),
            Self::SkipIfEqual { x, y } => xyn(SKIP_IF_EQUAL, x, y, LAST_ZERO),
            Self::SkipIfNotEqual { x, y } => xyn(SKIP_IF_NOT_EQUAL, x, y, LAST_ZERO),
            Self::StoreBcd { x } => xnn(MISC, x, STORE_BCD),
            Self::StoreRegisters { x } => xnn(MISC, x, STORE_REGISTERS),
            Self::LoadRegisters { x } => xnn(MISC, x, LOAD_REGISTERS),
            Self::LoadFontCharacter { x } => xnn(MISC, x, LOAD_FONT_CHARACTER),
            Self::Draw { x, y, n } => xyn(DRAW, x, y, n),
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
