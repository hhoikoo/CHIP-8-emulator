use super::*;
use Register::*;

const DECODABLE_OPCODE_COUNT: usize = 2 + 10 * 0x1000 + 11 * 0x100 + 10 * 0x10;

fn decode(opcode: u16) -> Result<Instruction, UnknownOpcodeError> {
    Instruction::try_from(Opcode::new(opcode))
}

fn addr(value: u16) -> Addr {
    Addr::new(value).unwrap()
}

#[test]
fn decodes_each_instruction() {
    assert_eq!(decode(0x00E0), Ok(Instruction::ClearScreen));
    assert_eq!(decode(0x00EE), Ok(Instruction::Return));
    assert_eq!(
        decode(0xDAB5),
        Ok(Instruction::Draw {
            x: VA,
            y: VB,
            n: Nibble::expect_new(5)
        })
    );
    assert_eq!(decode(0x1ABC), Ok(Instruction::Jump { addr: addr(0xABC) }));
    assert_eq!(
        decode(0xBABC),
        Ok(Instruction::JumpWithOffset { addr: addr(0xABC) })
    );
    assert_eq!(decode(0x2ABC), Ok(Instruction::Call { addr: addr(0xABC) }));
    assert_eq!(
        decode(0x3A05),
        Ok(Instruction::SkipIfEqualImmediate { x: VA, nn: 0x05 })
    );
    assert_eq!(
        decode(0x4A05),
        Ok(Instruction::SkipIfNotEqualImmediate { x: VA, nn: 0x05 })
    );
    assert_eq!(
        decode(0x5120),
        Ok(Instruction::SkipIfEqual { x: V1, y: V2 })
    );
    assert_eq!(
        decode(0x9120),
        Ok(Instruction::SkipIfNotEqual { x: V1, y: V2 })
    );
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
    assert_eq!(
        decode(0x8120),
        Ok(Instruction::LoadRegister { x: V1, y: V2 })
    );
    assert_eq!(decode(0x8121), Ok(Instruction::Or { x: V1, y: V2 }));
    assert_eq!(decode(0x8122), Ok(Instruction::And { x: V1, y: V2 }));
    assert_eq!(decode(0x8123), Ok(Instruction::Xor { x: V1, y: V2 }));
    assert_eq!(
        decode(0x8125),
        Ok(Instruction::SubtractWithBorrow { x: V1, y: V2 })
    );
    assert_eq!(decode(0x8126), Ok(Instruction::ShiftRight { x: V1, y: V2 }));
    assert_eq!(
        decode(0x8127),
        Ok(Instruction::SubtractReversed { x: V1, y: V2 })
    );
    assert_eq!(decode(0x812E), Ok(Instruction::ShiftLeft { x: V1, y: V2 }));
    assert_eq!(
        decode(0xAABC),
        Ok(Instruction::LoadIndex { addr: addr(0xABC) })
    );
    assert_eq!(decode(0xFA1E), Ok(Instruction::AddToIndex { x: VA }));
    assert_eq!(decode(0xFA33), Ok(Instruction::StoreBcd { x: VA }));
    assert_eq!(decode(0xFA55), Ok(Instruction::StoreRegisters { x: VA }));
    assert_eq!(decode(0xFA65), Ok(Instruction::LoadRegisters { x: VA }));
    assert_eq!(decode(0xFA29), Ok(Instruction::LoadFontCharacter { x: VA }));
    assert_eq!(decode(0xFA07), Ok(Instruction::ReadDelayTimer { x: VA }));
    assert_eq!(decode(0xFA15), Ok(Instruction::SetDelayTimer { x: VA }));
    assert_eq!(decode(0xFA18), Ok(Instruction::SetSoundTimer { x: VA }));
    assert_eq!(
        decode(0xCA0F),
        Ok(Instruction::LoadRandom { x: VA, nn: 0x0F })
    );
    assert_eq!(decode(0xEA9E), Ok(Instruction::SkipIfKeyPressed { x: VA }));
    assert_eq!(
        decode(0xEAA1),
        Ok(Instruction::SkipIfKeyNotPressed { x: VA })
    );
}

#[test]
fn operand_extremes() {
    assert_eq!(decode(0x2000), Ok(Instruction::Call { addr: addr(0x000) }));
    assert_eq!(decode(0x2FFF), Ok(Instruction::Call { addr: addr(0xFFF) }));
    assert_eq!(decode(0x1000), Ok(Instruction::Jump { addr: addr(0x000) }));
    assert_eq!(
        decode(0xBFFF),
        Ok(Instruction::JumpWithOffset { addr: addr(0xFFF) })
    );
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
    assert_eq!(
        decode(0xA000),
        Ok(Instruction::LoadIndex { addr: addr(0x000) })
    );
    assert_eq!(
        decode(0xAFFF),
        Ok(Instruction::LoadIndex { addr: addr(0xFFF) })
    );
    assert_eq!(decode(0xF01E), Ok(Instruction::AddToIndex { x: V0 }));
    assert_eq!(decode(0xFF1E), Ok(Instruction::AddToIndex { x: VF }));
}

#[test]
fn unrecognized_opcodes_are_errors_carrying_the_opcode() {
    for opcode in [
        0x0000, 0x00E1, 0x0001, 0x0123, 0x5121, 0x512F, 0x9121, 0x912F, 0x8128, 0x812D, 0x812F,
        0xF000, 0xF01F, 0xF02E, 0xF0E1, 0xF034, 0xF056, 0xF066, 0xF053, 0xF028, 0xF030, 0xF008,
        0xF017, 0xF014, 0xF019, 0xF016, 0xE000, 0xE09F, 0xE09D, 0xE0A0, 0xE0A2, 0xE01E,
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
fn near_misses_of_return_are_unknown() {
    assert!(decode(0x00EF).is_err());
    assert!(decode(0x00ED).is_err());
    assert!(decode(0x01EE).is_err());
}

#[test]
fn only_the_known_suffixes_decode_in_the_8_family() {
    let known_suffixes = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0xE];
    let decoded: Vec<u16> = (0x8000..=0x8FFF).filter(|&op| decode(op).is_ok()).collect();
    assert_eq!(decoded.len(), known_suffixes.len() * 0x100);
    assert!(
        decoded
            .iter()
            .all(|op| known_suffixes.contains(&(op & 0xF)))
    );
}

#[test]
fn only_the_known_operations_decode_in_the_e_family() {
    let known_operations = [0x9E, 0xA1];
    let decoded: Vec<u16> = (0xE000..=0xEFFF).filter(|&op| decode(op).is_ok()).collect();
    assert_eq!(decoded.len(), known_operations.len() * 0x10);
    assert!(
        decoded
            .iter()
            .all(|op| known_operations.contains(&(op & 0xFF)))
    );
}

#[test]
fn only_the_known_operations_decode_in_the_f_family() {
    let known_operations = [0x07, 0x15, 0x18, 0x1E, 0x29, 0x33, 0x55, 0x65];
    let decoded: Vec<u16> = (0xF000..=0xFFFF).filter(|&op| decode(op).is_ok()).collect();
    assert_eq!(decoded.len(), known_operations.len() * 0x10);
    assert!(
        decoded
            .iter()
            .all(|op| known_operations.contains(&(op & 0xFF)))
    );
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
            0x00E0 => Some(Instruction::ClearScreen),
            0x00EE => Some(Instruction::Return),
            0x1000..=0x1FFF => Some(Instruction::Jump { addr }),
            0x2000..=0x2FFF => Some(Instruction::Call { addr }),
            0x3000..=0x3FFF => Some(Instruction::SkipIfEqualImmediate { x, nn }),
            0x4000..=0x4FFF => Some(Instruction::SkipIfNotEqualImmediate { x, nn }),
            0x5000..=0x5FFF if opcode & 0xF == 0 => Some(Instruction::SkipIfEqual { x, y }),
            0x6000..=0x6FFF => Some(Instruction::LoadImmediate { x, nn }),
            0x7000..=0x7FFF => Some(Instruction::AddImmediate { x, nn }),
            0x8000..=0x8FFF if opcode & 0xF == 0 => Some(Instruction::LoadRegister { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 1 => Some(Instruction::Or { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 2 => Some(Instruction::And { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 3 => Some(Instruction::Xor { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 4 => Some(Instruction::AddWithCarry { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 5 => Some(Instruction::SubtractWithBorrow { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 6 => Some(Instruction::ShiftRight { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 7 => Some(Instruction::SubtractReversed { x, y }),
            0x8000..=0x8FFF if opcode & 0xF == 0xE => Some(Instruction::ShiftLeft { x, y }),
            0x9000..=0x9FFF if opcode & 0xF == 0 => Some(Instruction::SkipIfNotEqual { x, y }),
            0xA000..=0xAFFF => Some(Instruction::LoadIndex { addr }),
            0xB000..=0xBFFF => Some(Instruction::JumpWithOffset { addr }),
            0xC000..=0xCFFF => Some(Instruction::LoadRandom { x, nn }),
            0xD000..=0xDFFF => Some(Instruction::Draw {
                x,
                y,
                n: Nibble::new((opcode & 0xF) as u8).unwrap(),
            }),
            0xE000..=0xEFFF if opcode & 0xFF == 0x9E => Some(Instruction::SkipIfKeyPressed { x }),
            0xE000..=0xEFFF if opcode & 0xFF == 0xA1 => {
                Some(Instruction::SkipIfKeyNotPressed { x })
            }
            0xF000..=0xFFFF if opcode & 0xFF == 0x07 => Some(Instruction::ReadDelayTimer { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x15 => Some(Instruction::SetDelayTimer { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x18 => Some(Instruction::SetSoundTimer { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x1E => Some(Instruction::AddToIndex { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x29 => Some(Instruction::LoadFontCharacter { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x33 => Some(Instruction::StoreBcd { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x55 => Some(Instruction::StoreRegisters { x }),
            0xF000..=0xFFFF if opcode & 0xFF == 0x65 => Some(Instruction::LoadRegisters { x }),
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
    assert_eq!(decoded, DECODABLE_OPCODE_COUNT);
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
        (Instruction::ClearScreen, 0x00E0),
        (Instruction::Return, 0x00EE),
        (
            Instruction::Draw {
                x: VA,
                y: VB,
                n: Nibble::expect_new(5),
            },
            0xDAB5,
        ),
        (Instruction::Jump { addr: addr(0xABC) }, 0x1ABC),
        (Instruction::JumpWithOffset { addr: addr(0xABC) }, 0xBABC),
        (Instruction::Call { addr: addr(0xABC) }, 0x2ABC),
        (
            Instruction::SkipIfEqualImmediate { x: VA, nn: 0x05 },
            0x3A05,
        ),
        (
            Instruction::SkipIfNotEqualImmediate { x: VA, nn: 0x05 },
            0x4A05,
        ),
        (Instruction::SkipIfEqual { x: V1, y: V2 }, 0x5120),
        (Instruction::SkipIfNotEqual { x: V1, y: V2 }, 0x9120),
        (Instruction::LoadImmediate { x: VA, nn: 0x05 }, 0x6A05),
        (Instruction::AddImmediate { x: VB, nn: 0x7F }, 0x7B7F),
        (Instruction::AddWithCarry { x: V1, y: V2 }, 0x8124),
        (Instruction::LoadRegister { x: V1, y: V2 }, 0x8120),
        (Instruction::Or { x: V1, y: V2 }, 0x8121),
        (Instruction::And { x: V1, y: V2 }, 0x8122),
        (Instruction::Xor { x: V1, y: V2 }, 0x8123),
        (Instruction::SubtractWithBorrow { x: V1, y: V2 }, 0x8125),
        (Instruction::ShiftRight { x: V1, y: V2 }, 0x8126),
        (Instruction::SubtractReversed { x: V1, y: V2 }, 0x8127),
        (Instruction::ShiftLeft { x: V1, y: V2 }, 0x812E),
        (Instruction::LoadIndex { addr: addr(0xABC) }, 0xAABC),
        (Instruction::AddToIndex { x: VA }, 0xFA1E),
        (Instruction::StoreBcd { x: VA }, 0xFA33),
        (Instruction::StoreRegisters { x: VA }, 0xFA55),
        (Instruction::LoadRegisters { x: VA }, 0xFA65),
        (Instruction::LoadFontCharacter { x: VA }, 0xFA29),
        (Instruction::ReadDelayTimer { x: VA }, 0xFA07),
        (Instruction::SetDelayTimer { x: VA }, 0xFA15),
        (Instruction::SetSoundTimer { x: VA }, 0xFA18),
        (Instruction::LoadRandom { x: VA, nn: 0x0F }, 0xCA0F),
        (Instruction::SkipIfKeyPressed { x: VA }, 0xEA9E),
        (Instruction::SkipIfKeyNotPressed { x: VA }, 0xEAA1),
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
    assert_eq!(round_tripped, DECODABLE_OPCODE_COUNT);
}

#[test]
fn from_instruction_matches_encode() {
    let instruction = Instruction::AddWithCarry { x: VF, y: V0 };
    assert_eq!(Opcode::from(instruction), instruction.encode());
}
