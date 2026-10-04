use std::{error::Error, fmt::Display, write};

use crate::{instruction::Instruction, rom::Rom};

#[derive(Debug)]
pub struct Cpu {
    registers: [u8; Cpu::NUM_REGISTERS],
    pc: usize,
    memory: [u8; Cpu::MEMORY_SIZE],
    stack: [u16; Cpu::STACK_SIZE],
    stack_ptr: usize,
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
}

impl Cpu {
    pub const NUM_REGISTERS: usize = 16;
    pub const MEMORY_SIZE: usize = 0x1000;
    pub const STACK_SIZE: usize = 16;

    pub fn new(rom: &Rom) -> Self {
        let mut memory = [0; Cpu::MEMORY_SIZE];
        rom.load_into_memory(&mut memory);

        Cpu {
            registers: [0; Cpu::NUM_REGISTERS],
            pc: 0,
            memory,
            stack: [0; Cpu::STACK_SIZE],
            stack_ptr: 0,
        }
    }

    pub fn get_register_value(&self, num: usize) -> Option<u8> {
        self.registers.get(num).copied()
    }

    pub fn run(&mut self) -> Result<(), CpuError> {
        loop {
            let opcode = self.read_opcode()?;
            let instruction =
                Instruction::try_from(opcode).map_err(|err| CpuError::UnknownOpcode {
                    opcode: err.opcode,
                    pc: self.pc,
                })?;

            self.pc += 2;

            match instruction {
                Instruction::Halt => return Ok(()),
                Instruction::Return => self.ret()?,
                Instruction::Call { addr } => self.call(addr)?,
                Instruction::LoadImmediate { x, nn } => self.load_immediate(x, nn),
                Instruction::AddImmediate { x, nn } => self.add_immediate(x, nn),
                Instruction::AddWithCarry { x, y } => self.add_with_carry(x, y),
            }
        }
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

    fn load_immediate(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    }

    fn add_immediate(&mut self, x: u8, nn: u8) {
        let register: &mut u8 = &mut self.registers[x as usize];
        *register = register.wrapping_add(nn);
    }

    fn add_with_carry(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;
        self.registers[0xF] = u8::from(overflow);
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
        }
    }
}

impl Error for CpuError {}

#[cfg(test)]
mod tests {
    use crate::{
        example_fn::FN_ADD_TWICE,
        op::{OP_HALT, OP_RETURN},
    };

    use super::*;

    fn cpu(segs: &[(usize, &[u16])]) -> Cpu {
        Cpu::new(&Rom::new(segs).unwrap())
    }

    #[test]
    fn new_cpu_has_zeroed_registers() {
        let c = cpu(&[(0, &[OP_HALT])]);
        assert!((0..Cpu::NUM_REGISTERS).all(|r| c.get_register_value(r) == Some(0)));
    }

    #[test]
    fn execution_starts_at_address_zero() {
        let mut c = cpu(&[(0, &[0x6007, OP_HALT])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(7));
    }

    #[test]
    fn halt_stops_before_later_instructions() {
        let mut c = cpu(&[(0, &[OP_HALT, 0x6005])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(0));
    }

    #[test]
    fn code_in_the_last_memory_slot_runs() {
        // CALL the function at 0xFFE, whose RET is the final opcode in memory
        let mut c = cpu(&[(0, &[0x2FFE, 0x6001, OP_HALT]), (0xFFE, &[OP_RETURN])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(1));
    }

    #[test]
    fn load_and_add_immediate() {
        let mut c = cpu(&[(0, &[0x6005, 0x7003, OP_HALT])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(8));
    }

    #[test]
    fn add_immediate_wraps() {
        let mut c = cpu(&[(0, &[0x60FF, 0x7002, OP_HALT])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(1));
    }

    #[test]
    fn add_with_carry_sets_vf() {
        let mut c = cpu(&[(0, &[0x60F0, 0x6120, 0x8014, OP_HALT])]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(0x10));
        assert_eq!(c.get_register_value(0xF), Some(1));
    }

    #[test]
    fn the_real_main_program_via_two_segments() {
        let main = [0x6005, 0x610A, 0x2100, 0x2100, OP_HALT];
        let mut c = cpu(&[(0, &main), (0x100, &FN_ADD_TWICE)]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(45));
    }

    #[test]
    fn segment_order_in_literal_does_not_matter() {
        let main = [0x6005, 0x610A, 0x2100, 0x2100, OP_HALT];
        let mut c = cpu(&[(0x100, &FN_ADD_TWICE), (0, &main)]);
        c.run().unwrap();
        assert_eq!(c.get_register_value(0), Some(45));
    }

    fn run_err(segs: &[(usize, &[u16])]) -> CpuError {
        cpu(segs).run().unwrap_err()
    }

    #[test]
    fn ret_with_empty_stack_is_underflow() {
        // pc has already advanced past the RET when the error is raised
        let err = run_err(&[(0, &[OP_RETURN])]);
        assert!(matches!(err, CpuError::StackUnderflow { pc: 2 }));
    }

    #[test]
    fn infinite_recursion_is_stack_overflow() {
        let err = run_err(&[(0, &[0x2000])]);
        match err {
            CpuError::StackOverflow { pc, addr, stack } => {
                assert_eq!((pc, addr), (2, 0));
                assert_eq!(stack, vec![2; Cpu::STACK_SIZE]);
            }
            other => panic!("expected StackOverflow, got {other:?}"),
        }
    }

    /// Runs a program that nests `depth` CALLs, unwinds them all, then halts.
    /// Function `i` lives at `i * 0x10` and calls function `i + 1`.
    fn run_nested_calls(depth: usize) -> Result<(), CpuError> {
        let main = [0x2010, OP_HALT];
        let middle: Vec<[u16; 2]> = (1..depth)
            .map(|i| [0x2000 | ((i + 1) * 0x10) as u16, OP_RETURN])
            .collect();
        let last = [OP_RETURN];

        let mut segs: Vec<(usize, &[u16])> = vec![(0, &main)];
        segs.extend(
            middle
                .iter()
                .enumerate()
                .map(|(i, f)| ((i + 1) * 0x10, &f[..])),
        );
        segs.push((depth * 0x10, &last));

        cpu(&segs).run()
    }

    #[test]
    fn call_depth_at_the_stack_limit_is_ok() {
        run_nested_calls(Cpu::STACK_SIZE).unwrap();
    }

    #[test]
    fn one_call_past_the_stack_limit_overflows() {
        let err = run_nested_calls(Cpu::STACK_SIZE + 1).unwrap_err();
        assert!(matches!(err, CpuError::StackOverflow { .. }));
    }

    #[test]
    fn unknown_opcode_reports_its_own_address() {
        let err = run_err(&[(0, &[0x6001, 0xF000])]);
        assert!(matches!(
            err,
            CpuError::UnknownOpcode {
                opcode: 0xF000,
                pc: 2
            }
        ));
    }

    #[test]
    fn running_off_the_end_of_memory_is_pc_out_of_bounds() {
        // CALL the last opcode slot, which has no HALT after it
        let err = run_err(&[(0, &[0x2FFE]), (0xFFE, &[0x6001])]);
        assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0x1000 }));
    }

    #[test]
    fn odd_pc_on_the_last_byte_is_pc_out_of_bounds() {
        // 0xFFF has a first byte but no second byte
        let err = run_err(&[(0, &[0x2FFF])]);
        assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0xFFF }));
    }

    #[test]
    fn error_messages() {
        assert_eq!(
            run_err(&[(0, &[OP_RETURN])]).to_string(),
            "Stack underflow at pc 0x0002"
        );
        assert_eq!(
            run_err(&[(0, &[0x6001, 0xF000])]).to_string(),
            "Opcode 0xf000 at pc 0x0002 is unrecognized or is not supported yet"
        );
        assert_eq!(
            run_err(&[(0, &[0x2FFE]), (0xFFE, &[0x6001])]).to_string(),
            "Program counter 0x1000 out of bounds"
        );
        assert!(
            run_err(&[(0, &[0x2000])])
                .to_string()
                .starts_with("Stack overflow: CALL 0x0000 at pc 0x0002")
        );
    }
}
