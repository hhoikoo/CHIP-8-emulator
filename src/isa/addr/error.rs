use std::{error::Error, fmt::Display, write};

#[derive(Debug, PartialEq, Eq)]
pub struct AddrOutOfRange {
    pub value: u16,
}

impl Display for AddrOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Address {value:#06x} is out of range (expected 0x000 to 0xFFF)",
            value = self.value
        )
    }
}

impl Error for AddrOutOfRange {}
