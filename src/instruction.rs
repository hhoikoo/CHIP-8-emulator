use std::{error::Error, fmt::Display, write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,                            // 0x0000
    Return,                          // 0x00EE
    Call { addr: u16 },              // 0x2nnn
    LoadImmediate { x: u8, nn: u8 }, // 0x6xnn
    AddImmediate { x: u8, nn: u8 },  // 0x7xnn
    AddWithCarry { x: u8, y: u8 },   // 0x8xy4
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownOpcodeError {
    pub opcode: u16,
}

impl TryFrom<u16> for Instruction {
    type Error = UnknownOpcodeError;

    fn try_from(opcode: u16) -> Result<Self, UnknownOpcodeError> {
        match Instruction::extract_opcode_values(opcode) {
            (0, 0, 0, 0) => Ok(Self::Halt),
            (0x0, 0x0, 0xE, 0xE) => Ok(Self::Return),
            (0x2, _, _, _) => Ok(Self::Call {
                addr: Self::get_addr(opcode),
            }),
            (0x6, x, _, _) => Ok(Self::LoadImmediate {
                x,
                nn: Self::get_nn(opcode),
            }),
            (0x7, x, _, _) => Ok(Self::AddImmediate {
                x,
                nn: Self::get_nn(opcode),
            }),
            (0x8, x, y, 0x4) => Ok(Self::AddWithCarry { x, y }),
            _ => Err(UnknownOpcodeError { opcode }),
        }
    }
}

impl Instruction {
    fn extract_opcode_values(opcode: u16) -> (u8, u8, u8, u8) {
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = (opcode & 0x000F) as u8;

        (c, x, y, d)
    }

    fn get_addr(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn get_nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }
}

impl Display for UnknownOpcodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Opcode {opcode:#06x} is not recognized or is not supported yet",
            opcode = self.opcode
        )
    }
}

impl Error for UnknownOpcodeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::{OP_HALT, OP_RETURN};

    fn decode(opcode: u16) -> Result<Instruction, UnknownOpcodeError> {
        Instruction::try_from(opcode)
    }

    #[test]
    fn decodes_each_instruction() {
        assert_eq!(decode(0x0000), Ok(Instruction::Halt));
        assert_eq!(decode(0x00EE), Ok(Instruction::Return));
        assert_eq!(decode(0x2ABC), Ok(Instruction::Call { addr: 0xABC }));
        assert_eq!(
            decode(0x6A05),
            Ok(Instruction::LoadImmediate { x: 0xA, nn: 0x05 })
        );
        assert_eq!(
            decode(0x7B7F),
            Ok(Instruction::AddImmediate { x: 0xB, nn: 0x7F })
        );
        assert_eq!(decode(0x8124), Ok(Instruction::AddWithCarry { x: 1, y: 2 }));
    }

    #[test]
    fn func_constants_decode_to_halt_and_return() {
        assert_eq!(decode(OP_HALT), Ok(Instruction::Halt));
        assert_eq!(decode(OP_RETURN), Ok(Instruction::Return));
    }

    #[test]
    fn operand_extremes() {
        assert_eq!(decode(0x2000), Ok(Instruction::Call { addr: 0x000 }));
        assert_eq!(decode(0x2FFF), Ok(Instruction::Call { addr: 0xFFF }));
        assert_eq!(
            decode(0x6000),
            Ok(Instruction::LoadImmediate { x: 0, nn: 0 })
        );
        assert_eq!(
            decode(0x6FFF),
            Ok(Instruction::LoadImmediate { x: 0xF, nn: 0xFF })
        );
        assert_eq!(
            decode(0x8FF4),
            Ok(Instruction::AddWithCarry { x: 0xF, y: 0xF })
        );
    }

    #[test]
    fn unrecognized_opcodes_are_errors_carrying_the_opcode() {
        for opcode in [
            0x00E0, // CLS, not implemented
            0x0001, 0x0123, // SYS addr, not implemented
            0x1000, // JP, not implemented
            0x8120, 0x8125, // 8xy0 / 8xy5, not implemented
            0xF000,
        ] {
            assert_eq!(decode(opcode), Err(UnknownOpcodeError { opcode }));
        }
    }

    #[test]
    fn near_misses_of_halt_and_return_are_unknown() {
        assert!(decode(0x0001).is_err());
        assert!(decode(0x00EF).is_err());
        assert!(decode(0x00ED).is_err());
        assert!(decode(0x01EE).is_err());
    }

    #[test]
    fn only_8xy4_is_decoded_among_the_8_family() {
        let decoded: Vec<u16> = (0x8000..=0x8FFF).filter(|&op| decode(op).is_ok()).collect();
        assert_eq!(decoded.len(), 256);
        assert!(decoded.iter().all(|op| op & 0xF == 4));
    }

    #[test]
    fn exhaustive_decode_matches_the_opcode_bits() {
        let mut decoded = 0;
        for opcode in 0..=u16::MAX {
            let x = ((opcode >> 8) & 0xF) as u8;
            let y = ((opcode >> 4) & 0xF) as u8;
            let nn = (opcode & 0xFF) as u8;
            let addr = opcode & 0x0FFF;

            let expected = match opcode {
                0x0000 => Some(Instruction::Halt),
                0x00EE => Some(Instruction::Return),
                0x2000..=0x2FFF => Some(Instruction::Call { addr }),
                0x6000..=0x6FFF => Some(Instruction::LoadImmediate { x, nn }),
                0x7000..=0x7FFF => Some(Instruction::AddImmediate { x, nn }),
                0x8000..=0x8FFF if opcode & 0xF == 4 => Some(Instruction::AddWithCarry { x, y }),
                _ => None,
            };

            match (decode(opcode), expected) {
                (Ok(got), Some(want)) => {
                    assert_eq!(got, want, "opcode {opcode:#06x}");
                    decoded += 1;
                }
                (Err(err), None) => assert_eq!(err.opcode, opcode),
                (got, want) => panic!("opcode {opcode:#06x}: got {got:?}, expected {want:?}"),
            }
        }
        // Halt + Return + Call + LoadImmediate + AddImmediate + AddWithCarry
        assert_eq!(decoded, 1 + 1 + 4096 * 3 + 256);
    }

    #[test]
    fn unknown_opcode_error_message() {
        assert_eq!(
            UnknownOpcodeError { opcode: 0xF000 }.to_string(),
            "Opcode 0xf000 is not recognized or is not supported yet"
        );
    }
}
