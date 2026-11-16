mod error;

pub use error::NibbleOutOfRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nibble(u8);

impl Nibble {
    pub const MAX: Nibble = Nibble(0xF);

    pub const fn new(value: u8) -> Result<Nibble, NibbleOutOfRange> {
        if value <= Nibble::MAX.0 {
            Ok(Nibble(value))
        } else {
            Err(NibbleOutOfRange { value })
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub(crate) const fn expect_new(value: u8) -> Nibble {
        match Nibble::new(value) {
            Ok(nibble) => nibble,
            Err(_) => panic!("value must be 0x0 to 0xF"),
        }
    }
}

impl TryFrom<u8> for Nibble {
    type Error = NibbleOutOfRange;

    fn try_from(value: u8) -> Result<Self, NibbleOutOfRange> {
        Nibble::new(value)
    }
}

impl From<Nibble> for u8 {
    fn from(nibble: Nibble) -> u8 {
        nibble.get()
    }
}

impl From<Nibble> for u16 {
    fn from(nibble: Nibble) -> u16 {
        u16::from(nibble.get())
    }
}

impl From<Nibble> for usize {
    fn from(nibble: Nibble) -> usize {
        usize::from(nibble.get())
    }
}

#[cfg(test)]
mod tests;
