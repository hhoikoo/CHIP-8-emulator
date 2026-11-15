use super::{Cpu, CpuError, CpuState, JumpQuirk, ProgramCounter};
use crate::isa::{addr::Addr, instruction::Instruction, nibble::Nibble, register::Register};

impl Cpu {
    pub(super) fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        match instruction {
            Instruction::Halt => self.state = CpuState::Halt,
            Instruction::Return => self.ret()?,
            Instruction::Jump { addr } => self.jump(addr),
            Instruction::JumpWithOffset { addr } => self.jump_with_offset(addr),
            Instruction::Call { addr } => self.call(addr)?,
            Instruction::LoadImmediate { x, nn } => self.load_immediate(x, nn),
            Instruction::AddImmediate { x, nn } => self.add_immediate(x, nn),
            Instruction::AddWithCarry { x, y } => self.add_with_carry(x, y),
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<(), CpuError> {
        match self.stack_ptr.checked_sub(1) {
            Some(new_stack_ptr) => {
                self.stack_ptr = new_stack_ptr;
                self.pc = self.stack[self.stack_ptr];
                Ok(())
            }
            None => Err(CpuError::StackUnderflow { pc: self.pc }),
        }
    }

    fn jump(&mut self, addr: Addr) {
        self.pc = ProgramCounter::from(addr);
    }

    fn jump_with_offset(&mut self, addr: Addr) {
        let new_addr_value = match self.quirks.jump {
            JumpQuirk::AddV0 => addr.get() + u16::from(self.registers[Register::V0]),
            JumpQuirk::AddVx => {
                let [register, nn] = addr.get().to_be_bytes();
                let register = Register::from(Nibble::expect_new(register));
                u16::from(nn) + u16::from(self.registers[register])
            }
        };
        self.pc = ProgramCounter::from(new_addr_value);
    }

    fn call(&mut self, addr: Addr) -> Result<(), CpuError> {
        if self.stack_ptr >= self.stack.len() {
            return Err(CpuError::StackOverflow {
                pc: self.pc,
                addr,
                stack: self.stack[..self.stack_ptr].to_vec(),
            });
        }

        self.stack[self.stack_ptr] = self.pc;
        self.stack_ptr += 1;
        self.pc = ProgramCounter::from(addr);
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
