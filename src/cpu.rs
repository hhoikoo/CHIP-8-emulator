use std::{error::Error, fmt::Display, write};

use crate::{
    constant::{MEMORY_SIZE, STACK_SIZE},
    instruction::Instruction,
    register::{Register, Registers},
    rom::Rom,
};

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    pc: usize,
    memory: [u8; MEMORY_SIZE],
    stack: [u16; STACK_SIZE],
    stack_ptr: usize,
    state: CpuState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halt,
}

#[derive(Debug)]
pub enum CpuError {
    PcOutOfBounds {
        pc: usize,
    },
    UnknownOpcode {
        opcode: u16,
        pc: usize,
    },
    StackOverflow {
        pc: usize,
        addr: u16,
        stack: Vec<u16>,
    },
    StackUnderflow {
        pc: usize,
    },
    CycleLimitReached {
        pc: usize,
    },
}

impl Cpu {
    pub fn new(rom: &Rom) -> Self {
        let mut memory = [0; MEMORY_SIZE];
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

        match instruction {
            Instruction::Halt => self.state = CpuState::Halt,
            Instruction::Return => self.ret()?,
            Instruction::Call { addr } => self.call(addr)?,
            Instruction::LoadImmediate { x, nn } => self.load_immediate(x, nn),
            Instruction::AddImmediate { x, nn } => self.add_immediate(x, nn),
            Instruction::AddWithCarry { x, y } => self.add_with_carry(x, y),
        }
        Ok(())
    }

    fn read_opcode(&self) -> Result<u16, CpuError> {
        let pc = self.pc;
        match self.memory.get(pc..pc + 2) {
            Some(&[high, low]) => Ok(u16::from_be_bytes([high, low])),
            _ => Err(CpuError::PcOutOfBounds { pc }),
        }
    }

    fn ret(&mut self) -> Result<(), CpuError> {
        match self.stack_ptr.checked_sub(1) {
            Some(new_stack_ptr) => {
                self.stack_ptr = new_stack_ptr;
                self.pc = self.stack[self.stack_ptr] as usize;
                Ok(())
            }
            None => Err(CpuError::StackUnderflow { pc: self.pc }),
        }
    }

    fn call(&mut self, addr: u16) -> Result<(), CpuError> {
        if self.stack_ptr >= self.stack.len() {
            return Err(CpuError::StackOverflow {
                pc: self.pc,
                addr,
                stack: self.stack[..self.stack_ptr].to_vec(),
            });
        }

        self.stack[self.stack_ptr] = self.pc as u16;
        self.stack_ptr += 1;
        self.pc = addr as usize;
        Ok(())
    }

    fn load_immediate(&mut self, x: Register, nn: u8) {
        self.registers[x] = nn;
    }

    fn add_immediate(&mut self, x: Register, nn: u8) {
        let register: &mut u8 = &mut self.registers[x];
        *register = register.wrapping_add(nn);
    }

    fn add_with_carry(&mut self, x: Register, y: Register) {
        let arg1 = self.registers[x];
        let arg2 = self.registers[y];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x] = val;
        self.registers[Register::VF] = u8::from(overflow);
    }
}

impl Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuError::PcOutOfBounds { pc } => write!(f, "Program counter {pc:#06x} out of bounds"),
            CpuError::UnknownOpcode { opcode, pc } => {
                write!(
                    f,
                    "Opcode {opcode:#06x} at pc {pc:#06x} is unrecognized or is not supported yet"
                )
            }
            CpuError::StackOverflow { pc, addr, stack } => write!(
                f,
                "Stack overflow: CALL {addr:#06x} at pc {pc:#06x}\ncall stack (oldest first): {stack:#06x?}"
            ),
            CpuError::StackUnderflow { pc } => write!(f, "Stack underflow at pc {pc:#06x}"),
            CpuError::CycleLimitReached { pc } => {
                write!(f, "Cycle limit reached before halting, at pc {pc:#06x}")
            }
        }
    }
}

impl Error for CpuError {}

#[cfg(test)]
mod tests;
