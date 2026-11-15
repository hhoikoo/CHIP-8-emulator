use std::{fs, path::PathBuf};

use chip8_emulator::{
    cpu::{Cpu, CpuError},
    isa::{addr::Addr, instruction::Instruction, register::Register},
    memory,
    rom::Rom,
};

use Instruction::*;
use Register::*;

const PROGRAM_START: u16 = memory::PROGRAM_START.get();
const NESTED_CALL_DEPTH: u16 = 16;
const NESTED_FUNCTION_SIZE: u16 = 6;
const NESTED_FIRST_FUNCTION: u16 = PROGRAM_START + 4;

enum Outcome {
    Halts {
        registers: &'static [(Register, u8)],
    },
    FailsWith {
        message_prefix: &'static str,
    },
}

struct GeneratedRom {
    name: &'static str,
    words: Vec<u16>,
    cycles: usize,
    outcome: Outcome,
}

fn encode(instructions: &[Instruction]) -> Vec<u16> {
    instructions
        .iter()
        .map(|instruction| instruction.encode().get())
        .collect()
}

fn call(addr: u16) -> Instruction {
    Call {
        addr: Addr::new(addr).unwrap(),
    }
}

fn nested_function_addr(depth: u16) -> u16 {
    NESTED_FIRST_FUNCTION + (depth - 1) * NESTED_FUNCTION_SIZE
}

fn nested_calls() -> Vec<Instruction> {
    let mut program = vec![call(nested_function_addr(1)), Halt];
    for depth in 1..NESTED_CALL_DEPTH {
        program.extend([
            AddImmediate { x: V0, nn: 1 },
            call(nested_function_addr(depth + 1)),
            Return,
        ]);
    }
    program.extend([AddImmediate { x: V0, nn: 1 }, Return]);
    program
}

fn generated_roms() -> Vec<GeneratedRom> {
    vec![
        GeneratedRom {
            name: "halt",
            words: encode(&[Halt]),
            cycles: 1,
            outcome: Outcome::Halts { registers: &[] },
        },
        GeneratedRom {
            name: "zero_padding_halts",
            words: encode(&[LoadImmediate { x: V0, nn: 1 }]),
            cycles: 2,
            outcome: Outcome::Halts {
                registers: &[(V0, 1)],
            },
        },
        GeneratedRom {
            name: "load_and_add",
            words: encode(&[
                LoadImmediate { x: V0, nn: 5 },
                AddImmediate { x: V0, nn: 3 },
                Halt,
            ]),
            cycles: 3,
            outcome: Outcome::Halts {
                registers: &[(V0, 8)],
            },
        },
        GeneratedRom {
            name: "add_with_carry",
            words: encode(&[
                LoadImmediate { x: V0, nn: 0xF0 },
                LoadImmediate { x: V1, nn: 0x20 },
                AddWithCarry { x: V0, y: V1 },
                Halt,
            ]),
            cycles: 4,
            outcome: Outcome::Halts {
                registers: &[(V0, 0x10), (V1, 0x20), (VF, 1)],
            },
        },
        GeneratedRom {
            name: "call_and_return",
            words: encode(&[
                LoadImmediate { x: V0, nn: 5 },
                LoadImmediate { x: V1, nn: 10 },
                call(0x20A),
                call(0x20A),
                Halt,
                AddWithCarry { x: V0, y: V1 },
                AddWithCarry { x: V0, y: V1 },
                Return,
            ]),
            cycles: 11,
            outcome: Outcome::Halts {
                registers: &[(V0, 45), (V1, 10), (VF, 0)],
            },
        },
        GeneratedRom {
            name: "nested_calls",
            words: encode(&nested_calls()),
            cycles: 49,
            outcome: Outcome::Halts {
                registers: &[(V0, 16)],
            },
        },
        GeneratedRom {
            name: "stack_overflow",
            words: encode(&[call(PROGRAM_START)]),
            cycles: 17,
            outcome: Outcome::FailsWith {
                message_prefix: "Stack overflow: CALL 0x0200 at pc 0x0202",
            },
        },
        GeneratedRom {
            name: "stack_underflow",
            words: encode(&[Return]),
            cycles: 1,
            outcome: Outcome::FailsWith {
                message_prefix: "Stack underflow at pc 0x0202",
            },
        },
        GeneratedRom {
            name: "unknown_opcode",
            words: vec![LoadImmediate { x: V0, nn: 1 }.encode().get(), 0xF000],
            cycles: 2,
            outcome: Outcome::FailsWith {
                message_prefix: "Opcode 0xf000 at pc 0x0202 is unrecognized",
            },
        },
    ]
}

fn rom_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/roms/generated")
        .join(format!("{name}.ch8"))
}

fn bytes(words: &[u16]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_be_bytes()).collect()
}

fn load(rom: &GeneratedRom) -> Cpu {
    Cpu::new(&Rom::from_file(rom_path(rom.name)).unwrap())
}

#[test]
fn rom_files_match_their_listings() {
    for rom in generated_roms() {
        let on_disk = fs::read(rom_path(rom.name)).unwrap();
        assert_eq!(
            on_disk,
            bytes(&rom.words),
            "{}.ch8 is stale; run `cargo test --test generated_roms -- --ignored`",
            rom.name
        );
    }
}

#[test]
fn roms_reach_their_outcome_in_exactly_their_cycle_budget() {
    for rom in generated_roms() {
        let mut cpu = load(&rom);
        match (cpu.run_for(rom.cycles), &rom.outcome) {
            (Ok(()), Outcome::Halts { registers }) => {
                for &(register, value) in registers.iter() {
                    assert_eq!(
                        cpu.get_register_value(register),
                        value,
                        "{}: {register:?}",
                        rom.name
                    );
                }
            }
            (Err(err), Outcome::FailsWith { message_prefix }) => {
                assert!(
                    err.to_string().starts_with(message_prefix),
                    "{}: unexpected error {err}",
                    rom.name
                );
            }
            (result, _) => panic!("{}: unexpected result {result:?}", rom.name),
        }
    }
}

#[test]
fn roms_need_their_whole_cycle_budget() {
    for rom in generated_roms() {
        let result = load(&rom).run_for(rom.cycles - 1);
        assert!(
            matches!(result, Err(CpuError::CycleLimitReached { .. })),
            "{}: finished early with {result:?}",
            rom.name
        );
    }
}

#[test]
#[ignore = "rewrites tests/roms/generated; run after changing a listing"]
fn regenerate_rom_files() {
    for rom in generated_roms() {
        let path = rom_path(rom.name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes(&rom.words)).unwrap();
    }
}
