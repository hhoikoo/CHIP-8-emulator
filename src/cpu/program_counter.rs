use std::fmt;

use crate::isa::addr::Addr;

const INSTRUCTION_SIZE: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramCounter(u16);

impl ProgramCounter {
    pub const fn new(value: u16) -> ProgramCounter {
        ProgramCounter(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub(crate) fn advance(&mut self) {
        self.0 = self
            .0
            .checked_add(INSTRUCTION_SIZE)
            .expect("a fetched program counter lies inside memory, far below u16::MAX");
    }
}

impl From<u16> for ProgramCounter {
    fn from(value: u16) -> ProgramCounter {
        ProgramCounter(value)
    }
}

impl From<Addr> for ProgramCounter {
    fn from(addr: Addr) -> ProgramCounter {
        ProgramCounter(addr.get())
    }
}

impl From<ProgramCounter> for u16 {
    fn from(pc: ProgramCounter) -> u16 {
        pc.get()
    }
}

impl From<ProgramCounter> for usize {
    fn from(pc: ProgramCounter) -> usize {
        usize::from(pc.get())
    }
}

impl fmt::LowerHex for ProgramCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests;
