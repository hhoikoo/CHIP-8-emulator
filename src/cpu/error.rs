use std::{error::Error, fmt::Display, write};

#[derive(Debug)]
pub enum CpuError {
    PcOutOfBounds {
        pc: usize,
    },
    UnknownOpcode {
        opcode: u16,
        pc: usize,
    },
    StackOverflow {
        pc: usize,
        addr: u16,
        stack: Vec<u16>,
    },
    StackUnderflow {
        pc: usize,
    },
    CycleLimitReached {
        pc: usize,
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
            CpuError::StackOverflow { pc, addr, stack } => write!(
                f,
                "Stack overflow: CALL {addr:#06x} at pc {pc:#06x}\ncall stack (oldest first): {stack:#06x?}"
            ),
            CpuError::StackUnderflow { pc } => write!(f, "Stack underflow at pc {pc:#06x}"),
            CpuError::CycleLimitReached { pc } => {
                write!(f, "Cycle limit reached before halting, at pc {pc:#06x}")
            }
        }
    }
}

impl Error for CpuError {}
