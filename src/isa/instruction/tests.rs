use super::*;
use crate::op::{OP_HALT, OP_RETURN};
use Register::*;

fn decode(opcode: u16) -> Result<Instruction, UnknownOpcodeError> {
    Instruction::try_from(Opcode::new(opcode))
}

fn addr(value: u16) -> Addr {
    Addr::new(value).unwrap()
}

#[test]
fn decodes_each_instruction() {
    assert_eq!(decode(0x0000), Ok(Instruction::Halt));
    assert_eq!(decode(0x00EE), Ok(Instruction::Return));
    assert_eq!(decode(0x2ABC), Ok(Instruction::Call { addr: addr(0xABC) }));
    assert_eq!(
        decode(0x6A05),
        Ok(Instruction::LoadImmediate { x: VA, nn: 0x05 })
    );
    assert_eq!(
        decode(0x7B7F),
        Ok(Instruction::AddImmediate { x: VB, nn: 0x7F })
    );
    assert_eq!(
        decode(0x8124),
        Ok(Instruction::AddWithCarry { x: V1, y: V2 })
    );
}

#[test]
fn func_constants_decode_to_halt_and_return() {
    assert_eq!(decode(OP_HALT), Ok(Instruction::Halt));
    assert_eq!(decode(OP_RETURN), Ok(Instruction::Return));
}

#[test]
fn operand_extremes() {
    assert_eq!(decode(0x2000), Ok(Instruction::Call { addr: addr(0x000) }));
    assert_eq!(decode(0x2FFF), Ok(Instruction::Call { addr: addr(0xFFF) }));
    assert_eq!(
        decode(0x6000),
        Ok(Instruction::LoadImmediate { x: V0, nn: 0 })
    );
    assert_eq!(
        decode(0x6FFF),
        Ok(Instruction::LoadImmediate { x: VF, nn: 0xFF })
    );
    assert_eq!(
        decode(0x8FF4),
        Ok(Instruction::AddWithCarry { x: VF, y: VF })
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
        assert_eq!(
            decode(opcode),
            Err(UnknownOpcodeError {
                opcode: Opcode::new(opcode)
            })
        );
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
        let x = Register::try_from(((opcode >> 8) & 0xF) as u8).unwrap();
        let y = Register::try_from(((opcode >> 4) & 0xF) as u8).unwrap();
        let nn = (opcode & 0xFF) as u8;
        let addr = Addr::new(opcode & 0x0FFF).unwrap();

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
            (Err(err), None) => assert_eq!(err.opcode.get(), opcode),
            (got, want) => panic!("opcode {opcode:#06x}: got {got:?}, expected {want:?}"),
        }
    }
    // Halt + Return + Call + LoadImmediate + AddImmediate + AddWithCarry
    assert_eq!(decoded, 1 + 1 + 4096 * 3 + 256);
}

#[test]
fn unknown_opcode_error_message() {
    assert_eq!(
        UnknownOpcodeError {
            opcode: Opcode::new(0xF000)
        }
        .to_string(),
        "Opcode 0xf000 is not recognized or is not supported yet"
    );
}

#[test]
fn encodes_each_instruction() {
    let cases = [
        (Instruction::Halt, 0x0000),
        (Instruction::Return, 0x00EE),
        (Instruction::Call { addr: addr(0xABC) }, 0x2ABC),
        (Instruction::LoadImmediate { x: VA, nn: 0x05 }, 0x6A05),
        (Instruction::AddImmediate { x: VB, nn: 0x7F }, 0x7B7F),
        (Instruction::AddWithCarry { x: V1, y: V2 }, 0x8124),
    ];
    for (instruction, opcode) in cases {
        assert_eq!(instruction.encode(), Opcode::new(opcode), "{instruction:?}");
    }
}

#[test]
fn encode_is_the_reverse_of_decode_for_every_opcode() {
    let mut round_tripped = 0;
    for opcode in 0..=u16::MAX {
        if let Ok(instruction) = decode(opcode) {
            assert_eq!(instruction.encode().get(), opcode, "{instruction:?}");
            round_tripped += 1;
        }
    }
    assert_eq!(round_tripped, 1 + 1 + 4096 * 3 + 256);
}

#[test]
fn from_instruction_matches_encode() {
    let instruction = Instruction::AddWithCarry { x: VF, y: V0 };
    assert_eq!(Opcode::from(instruction), instruction.encode());
}
