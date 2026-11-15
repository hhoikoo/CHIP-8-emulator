use std::{error::Error, fmt::Display, write};

use super::ProgramCounter;
use crate::isa::{addr::Addr, opcode::Opcode};

#[derive(Debug)]
pub enum CpuError {
    PcOutOfBounds {
        pc: ProgramCounter,
    },
    UnknownOpcode {
        opcode: Opcode,
        pc: ProgramCounter,
    },
    StackOverflow {
        pc: ProgramCounter,
        addr: Addr,
        stack: Vec<ProgramCounter>,
    },
    StackUnderflow {
        pc: ProgramCounter,
    },
}

impl Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuError::PcOutOfBounds { pc } => write!(f, "Program counter {pc:#06x} out of bounds"),
            CpuError::UnknownOpcode { opcode, pc } => {
                write!(
                    f,
                    "Opcode {opcode:#06x} at pc {pc:#06x} is unrecognized or is not supported yet"
                )
            }
            CpuError::StackOverflow { pc, addr, stack } => {
                let stack: Vec<u16> = stack.iter().copied().map(u16::from).collect();
                write!(
                    f,
                    "Stack overflow: CALL {addr:#06x} at pc {pc:#06x}\ncall stack (oldest first): {stack:#06x?}"
                )
            }
            CpuError::StackUnderflow { pc } => write!(f, "Stack underflow at pc {pc:#06x}"),
        }
    }
}

impl Error for CpuError {}
