use core::fmt;
use std::{error::Error, io, path::PathBuf, write};

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

impl Error for LoadError {}

#[derive(Debug)]
pub enum RomFileError {
    Read { path: PathBuf, source: io::Error },
    Load { path: PathBuf, source: LoadError },
}

impl fmt::Display for RomFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomFileError::Read { path, source } => {
                write!(f, "Couldn't read ROM file {}: {source}", path.display())
            }
            RomFileError::Load { path, source } => {
                write!(f, "Couldn't load ROM file {}: {source}", path.display())
            }
        }
    }
}

impl Error for RomFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RomFileError::Read { source, .. } => Some(source),
            RomFileError::Load { source, .. } => Some(source),
        }
    }
}
