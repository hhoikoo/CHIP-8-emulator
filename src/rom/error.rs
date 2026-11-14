use core::fmt;
use std::write;

#[derive(Debug)]
pub enum LoadError {
    OutOfBounds {
        addr: usize,
        byte_len: usize,
    },
    Overlap {
        first_start: usize,
        second_start: usize,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::OutOfBounds { addr, byte_len } => write!(
                f,
                "Segment at {addr:#06x} ({byte_len} bytes) doesn't fit in memory"
            ),
            LoadError::Overlap {
                first_start,
                second_start,
            } => write!(
                f,
                "Segments at {first_start:#06x} and {second_start:#06x} overlap"
            ),
        }
    }
}

impl std::error::Error for LoadError {}
