mod error;
mod execute;
mod program_counter;
mod quirks;

pub use error::CpuError;
pub use program_counter::ProgramCounter;
pub use quirks::{JumpQuirk, LogicQuirk, Quirks};

use crate::{
    isa::{
        instruction::Instruction,
        opcode::Opcode,
        register::{Register, Registers},
    },
    memory::{Memory, PROGRAM_START},
    rom::Rom,
};

pub const STACK_SIZE: usize = 16;

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    pc: ProgramCounter,
    memory: Memory,
    stack: [ProgramCounter; STACK_SIZE],
    stack_ptr: usize,
    quirks: Quirks,
}

impl Cpu {
    pub fn new(rom: &Rom) -> Self {
        Self::with_quirks(rom, Quirks::default())
    }

    pub fn with_quirks(rom: &Rom, quirks: Quirks) -> Self {
        let mut memory = Memory::default();
        rom.load_into_memory(&mut memory);

        Cpu {
            registers: Registers::default(),
            pc: ProgramCounter::from(PROGRAM_START),
            memory,
            stack: [ProgramCounter::new(0); STACK_SIZE],
            stack_ptr: 0,
            quirks,
        }
    }

    pub fn get_register_value(&self, register: Register) -> u8 {
        self.registers[register]
    }

    pub fn run(&mut self, mut should_stop: impl FnMut() -> bool) -> Result<(), CpuError> {
        while !should_stop() {
            self.step()?;
        }
        Ok(())
    }

    pub fn run_for(&mut self, cycle_limit: usize) -> Result<(), CpuError> {
        for _ in 0..cycle_limit {
            self.step()?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), CpuError> {
        let opcode = self.read_opcode()?;
        let instruction = Instruction::try_from(opcode).map_err(|err| CpuError::UnknownOpcode {
            opcode: err.opcode,
            pc: self.pc,
        })?;

        self.pc.advance();

        self.execute(instruction)
    }

    fn read_opcode(&self) -> Result<Opcode, CpuError> {
        self.memory
            .read_opcode(usize::from(self.pc))
            .ok_or(CpuError::PcOutOfBounds { pc: self.pc })
    }
}

#[cfg(test)]
mod tests;
