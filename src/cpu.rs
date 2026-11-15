mod error;
mod execute;
mod program_counter;

pub use error::CpuError;
pub use program_counter::ProgramCounter;

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
    state: CpuState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halt,
}

impl Cpu {
    pub fn new(rom: &Rom) -> Self {
        let mut memory = Memory::default();
        rom.load_into_memory(&mut memory);

        Cpu {
            registers: Registers::default(),
            pc: ProgramCounter::from(PROGRAM_START),
            memory,
            stack: [ProgramCounter::new(0); STACK_SIZE],
            stack_ptr: 0,
            state: CpuState::Running,
        }
    }

    pub fn get_register_value(&self, register: Register) -> u8 {
        self.registers[register]
    }

    pub fn run(&mut self) -> Result<(), CpuError> {
        loop {
            self.step()?;
            if self.state == CpuState::Halt {
                return Ok(());
            }
        }
    }

    pub fn run_for(&mut self, cycle_limit: usize) -> Result<(), CpuError> {
        for _ in 0..cycle_limit {
            self.step()?;
            if self.state == CpuState::Halt {
                return Ok(());
            }
        }
        Err(CpuError::CycleLimitReached { pc: self.pc })
    }

    pub fn step(&mut self) -> Result<(), CpuError> {
        if self.state == CpuState::Halt {
            return Ok(());
        }

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
