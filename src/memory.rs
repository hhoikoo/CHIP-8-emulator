use crate::isa::{addr::Addr, opcode::Opcode};

pub const MEMORY_SIZE: usize = 0x1000;
pub const PROGRAM_START: Addr = Addr::expect_new(0x200);

#[derive(Debug)]
pub struct Memory([u8; MEMORY_SIZE]);

impl Memory {
    pub fn read(&self, start: usize, len: usize) -> Option<&[u8]> {
        let end = end(start, len)?;
        self.0.get(start..end)
    }

    pub fn read_opcode(&self, addr: usize) -> Option<Opcode> {
        self.read_array(addr)
            .map(u16::from_be_bytes)
            .map(Opcode::new)
    }

    pub fn write(&mut self, start: usize, bytes: &[u8]) -> Option<()> {
        let end = end(start, bytes.len())?;
        self.0.get_mut(start..end)?.copy_from_slice(bytes);
        Some(())
    }

    fn read_array<const N: usize>(&self, start: usize) -> Option<[u8; N]> {
        self.read(start, N)?.try_into().ok()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self([0; MEMORY_SIZE])
    }
}

fn end(start: usize, len: usize) -> Option<usize> {
    start.checked_add(len)
}

#[cfg(test)]
mod tests;
