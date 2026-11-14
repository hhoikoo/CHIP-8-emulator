use std::{error::Error, fmt::Display, write};

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterOutOfRange {
    pub value: u8,
}

impl Display for RegisterOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Register number {value:#04x} is out of range (expected 0x0 to 0xF)",
            value = self.value
        )
    }
}

impl Error for RegisterOutOfRange {}
