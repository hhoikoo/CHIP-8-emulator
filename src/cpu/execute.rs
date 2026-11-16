use super::{
    ClipQuirk, Cpu, CpuError, DisplayWaitQuirk, JumpQuirk, LogicQuirk, MemoryQuirk, ProgramCounter,
    ShiftQuirk, State,
};
use crate::{
    font,
    framebuffer::{HEIGHT, WIDTH},
    isa::{addr::Addr, instruction::Instruction, nibble::Nibble, register::Register},
};

impl Cpu {
    pub(super) fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        match instruction {
            Instruction::ClearScreen => self.framebuffer.clear(),
            Instruction::Return => self.ret()?,
            Instruction::Jump { addr } => self.jump(addr),
            Instruction::JumpWithOffset { addr } => self.jump_with_offset(addr),
            Instruction::Call { addr } => self.call(addr)?,
            Instruction::LoadImmediate { x, nn } => self.load_immediate(x, nn),
            Instruction::LoadRegister { x, y } => self.load_register(x, y),
            Instruction::LoadIndex { addr } => self.load_index(addr),
            Instruction::AddImmediate { x, nn } => self.add_immediate(x, nn),
            Instruction::AddWithCarry { x, y } => self.add_with_carry(x, y),
            Instruction::AddToIndex { x } => self.add_to_index(x),
            Instruction::Or { x, y } => self.or(x, y),
            Instruction::And { x, y } => self.and(x, y),
            Instruction::Xor { x, y } => self.xor(x, y),
            Instruction::SubtractWithBorrow { x, y } => self.subtract_with_borrow(x, y),
            Instruction::SubtractReversed { x, y } => self.subtract_reversed(x, y),
            Instruction::ShiftRight { x, y } => self.shift_right(x, y),
            Instruction::ShiftLeft { x, y } => self.shift_left(x, y),
            Instruction::SkipIfEqualImmediate { x, nn } => self.skip_if_equal_immediate(x, nn),
            Instruction::SkipIfNotEqualImmediate { x, nn } => {
                self.skip_if_not_equal_immediate(x, nn)
            }
            Instruction::SkipIfEqual { x, y } => self.skip_if_equal(x, y),
            Instruction::SkipIfNotEqual { x, y } => self.skip_if_not_equal(x, y),
            Instruction::StoreBcd { x } => self.store_bcd(x)?,
            Instruction::StoreRegisters { x } => self.store_registers(x)?,
            Instruction::LoadRegisters { x } => self.load_registers(x)?,
            Instruction::LoadFontCharacter { x } => self.load_font_character(x)?,
            Instruction::Draw { x, y, n } => self.draw(x, y, n)?,
            Instruction::ReadDelayTimer { x } => self.registers[x] = self.delay_timer,
            Instruction::SetDelayTimer { x } => self.delay_timer = self.registers[x],
            Instruction::SetSoundTimer { x } => self.sound_timer = self.registers[x],
            Instruction::LoadRandom { x, nn } => self.registers[x] = self.random.next_byte() & nn,
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

    fn load_register(&mut self, x: Register, y: Register) {
        self.registers[x] = self.registers[y];
    }

    fn load_index(&mut self, addr: Addr) {
        self.i = addr.get();
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

    fn add_to_index(&mut self, x: Register) {
        self.i = self.i.wrapping_add(u16::from(self.registers[x]));
    }

    fn or(&mut self, x: Register, y: Register) {
        self.registers[x] |= self.registers[y];
        self.reset_vf();
    }

    fn and(&mut self, x: Register, y: Register) {
        self.registers[x] &= self.registers[y];
        self.reset_vf();
    }

    fn xor(&mut self, x: Register, y: Register) {
        self.registers[x] ^= self.registers[y];
        self.reset_vf();
    }

    fn subtract_with_borrow(&mut self, x: Register, y: Register) {
        let (val, borrowed) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = val;
        self.registers[Register::VF] = u8::from(!borrowed);
    }

    fn subtract_reversed(&mut self, x: Register, y: Register) {
        let (val, borrowed) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = val;
        self.registers[Register::VF] = u8::from(!borrowed);
    }

    fn shift_right(&mut self, x: Register, y: Register) {
        let source_register = self.get_shift_source_register(x, y);
        let source = self.registers[source_register];
        let flag = source & 0x1;

        self.registers[x] = source >> 1;
        self.registers[Register::VF] = flag;
    }

    fn shift_left(&mut self, x: Register, y: Register) {
        let source_register = self.get_shift_source_register(x, y);
        let source = self.registers[source_register];
        let flag = source >> 7;

        self.registers[x] = source << 1;
        self.registers[Register::VF] = flag;
    }

    fn get_shift_source_register(&self, x: Register, y: Register) -> Register {
        match self.quirks.shift {
            ShiftQuirk::ShiftVy => y,
            ShiftQuirk::ShiftVxInPlace => x,
        }
    }

    fn reset_vf(&mut self) {
        match self.quirks.logic {
            LogicQuirk::ResetVf => self.registers[Register::VF] = 0,
            LogicQuirk::KeepVf => (),
        }
    }

    fn skip_if_equal_immediate(&mut self, x: Register, nn: u8) {
        self.skip_if(self.registers[x] == nn);
    }

    fn skip_if_not_equal_immediate(&mut self, x: Register, nn: u8) {
        self.skip_if(self.registers[x] != nn);
    }

    fn skip_if_equal(&mut self, x: Register, y: Register) {
        self.skip_if(self.registers[x] == self.registers[y]);
    }

    fn skip_if_not_equal(&mut self, x: Register, y: Register) {
        self.skip_if(self.registers[x] != self.registers[y]);
    }

    fn skip_if(&mut self, skip: bool) {
        if skip {
            self.pc.advance();
        }
    }

    fn store_bcd(&mut self, x: Register) -> Result<(), CpuError> {
        let value = self.registers[x];
        let digits = [value / 100, value / 10 % 10, value % 10];
        self.memory
            .write(usize::from(self.i), &digits)
            .ok_or_else(|| self.index_out_of_bounds(digits.len()))
    }

    fn store_registers(&mut self, x: Register) -> Result<(), CpuError> {
        let values = self.registers.up_to(x);
        self.memory
            .write(usize::from(self.i), values)
            .ok_or_else(|| self.index_out_of_bounds(values.len()))?;
        self.advance_index_past(x);
        Ok(())
    }

    fn load_registers(&mut self, x: Register) -> Result<(), CpuError> {
        let len = self.registers.up_to(x).len();
        let values = self
            .memory
            .read(usize::from(self.i), len)
            .ok_or_else(|| self.index_out_of_bounds(len))?;
        self.registers.up_to_mut(x).copy_from_slice(values);
        self.advance_index_past(x);
        Ok(())
    }

    fn load_font_character(&mut self, x: Register) -> Result<(), CpuError> {
        let value = self.registers[x];
        let digit =
            Nibble::new(value).map_err(|_| CpuError::FontDigitOutOfRange { pc: self.pc, value })?;
        self.i = font::glyph_addr(digit);
        Ok(())
    }

    fn draw(&mut self, x: Register, y: Register, n: Nibble) -> Result<(), CpuError> {
        let len = usize::from(n);
        let sprite = self
            .memory
            .read(usize::from(self.i), len)
            .ok_or_else(|| self.index_out_of_bounds(len))?;
        let left = usize::from(self.registers[x]) % WIDTH;
        let top = usize::from(self.registers[y]) % HEIGHT;

        let mut turned_off = false;
        for (row, &bits) in sprite.iter().enumerate() {
            for column in (0..8).filter(|column| bits & (0x80 >> column) != 0) {
                let (px, py) = (left + column, top + row);
                if self.quirks.clip == ClipQuirk::Clip && (px >= WIDTH || py >= HEIGHT) {
                    continue;
                }
                turned_off |= self.framebuffer.flip(px, py);
            }
        }
        self.registers[Register::VF] = u8::from(turned_off);
        if self.quirks.display_wait == DisplayWaitQuirk::WaitForFrame {
            self.state = State::WaitingForFrame;
        }
        Ok(())
    }

    fn advance_index_past(&mut self, x: Register) {
        match self.quirks.memory {
            MemoryQuirk::AdvanceI => self.i = self.i.wrapping_add(u16::from(x) + 1),
            MemoryQuirk::LeaveI => (),
        }
    }

    fn index_out_of_bounds(&self, len: usize) -> CpuError {
        CpuError::IndexOutOfBounds {
            pc: self.pc,
            i: self.i,
            len,
        }
    }
}
