use std::{error::Error, fmt::Display, write};

use crate::isa::opcode::Opcode;

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownOpcodeError {
    pub opcode: Opcode,
}

impl Display for UnknownOpcodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Opcode {opcode:#06x} is not recognized or is not supported yet",
            opcode = self.opcode
        )
    }
}

impl Error for UnknownOpcodeError {}
