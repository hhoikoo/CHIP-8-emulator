mod error;
mod execute;
mod program_counter;
mod quirks;

pub use error::CpuError;
pub use program_counter::ProgramCounter;
pub use quirks::{
    ClipQuirk, DisplayWaitQuirk, JumpQuirk, LogicQuirk, MemoryQuirk, Quirks, ShiftQuirk,
};

use crate::{
    font::{FONT_START, GLYPHS},
    framebuffer::Framebuffer,
    isa::{
        instruction::Instruction,
        opcode::Opcode,
        register::{Register, Registers},
    },
    memory::{Memory, PROGRAM_START},
    rom::Rom,
};

pub const STACK_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Running,
    WaitingForFrame,
}

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    pc: ProgramCounter,
    memory: Memory,
    stack: [ProgramCounter; STACK_SIZE],
    stack_ptr: usize,
    quirks: Quirks,
    i: u16,
    framebuffer: Framebuffer,
    delay_timer: u8,
    sound_timer: u8,
    state: State,
}

impl Cpu {
    pub fn new(rom: &Rom) -> Self {
        Self::with_quirks(rom, Quirks::default())
    }

    pub fn with_quirks(rom: &Rom, quirks: Quirks) -> Self {
        let mut memory = Memory::default();
        memory
            .write(usize::from(FONT_START), GLYPHS.as_flattened())
            .expect("the font fits below the program start");
        rom.load_into_memory(&mut memory);

        Cpu {
            registers: Registers::default(),
            pc: ProgramCounter::from(PROGRAM_START),
            memory,
            stack: [ProgramCounter::new(0); STACK_SIZE],
            stack_ptr: 0,
            quirks,
            i: 0,
            framebuffer: Framebuffer::default(),
            delay_timer: 0,
            sound_timer: 0,
            state: State::Running,
        }
    }

    pub fn get_register_value(&self, register: Register) -> u8 {
        self.registers[register]
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    pub fn sound_active(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn tick_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
        if self.state == State::WaitingForFrame {
            self.state = State::Running;
        }
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
        if self.state != State::Running {
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
