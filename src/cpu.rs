mod error;
mod execute;

pub use error::CpuError;

use crate::{
    isa::{
        instruction::Instruction,
        opcode::Opcode,
        register::{Register, Registers},
    },
    memory::Memory,
    rom::Rom,
};

pub const STACK_SIZE: usize = 16;

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    pc: usize,
    memory: Memory,
    stack: [u16; STACK_SIZE],
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
            pc: 0,
            memory,
            stack: [0; STACK_SIZE],
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

        self.pc += 2;

        self.execute(instruction)
    }

    fn read_opcode(&self) -> Result<Opcode, CpuError> {
        self.memory
            .read_opcode(self.pc)
            .ok_or(CpuError::PcOutOfBounds { pc: self.pc })
    }
}

#[cfg(test)]
mod tests;
