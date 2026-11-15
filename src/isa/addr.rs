mod error;

pub use error::AddrOutOfRange;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Addr(u16);

impl Addr {
    pub const MAX: Addr = Addr(0x0FFF);

    pub const fn new(value: u16) -> Result<Addr, AddrOutOfRange> {
        if value <= Addr::MAX.0 {
            Ok(Addr(value))
        } else {
            Err(AddrOutOfRange { value })
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Addr {
    type Error = AddrOutOfRange;

    fn try_from(value: u16) -> Result<Self, AddrOutOfRange> {
        Addr::new(value)
    }
}

impl From<Addr> for u16 {
    fn from(addr: Addr) -> u16 {
        addr.get()
    }
}

impl From<Addr> for usize {
    fn from(addr: Addr) -> usize {
        usize::from(addr.get())
    }
}

impl fmt::LowerHex for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests;
