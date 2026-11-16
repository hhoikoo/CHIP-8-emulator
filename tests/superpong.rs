use std::path::PathBuf;

use chip8_emulator::{
    cpu::{Cpu, DisplayWaitQuirk, Quirks},
    isa::nibble::Nibble,
    rom::Rom,
};

const INSTRUCTIONS_PER_FRAME: usize = 30;
const SERVE: u8 = 0x6;
const PADDLE_LEFT: u8 = 0x7;

fn superpong() -> Cpu {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/roms/chip8archive/superpong.ch8");
    let quirks = Quirks {
        display_wait: DisplayWaitQuirk::DrawImmediately,
        ..Quirks::default()
    };
    Cpu::with_quirks(&Rom::from_file(path).unwrap(), quirks).seeded(1)
}

fn run_frames(cpu: &mut Cpu, frames: usize) {
    for _ in 0..frames {
        cpu.run_for(INSTRUCTIONS_PER_FRAME).unwrap();
        cpu.tick_timers();
    }
}

fn tap(cpu: &mut Cpu, key: u8) {
    let key = Nibble::new(key).unwrap();
    cpu.press_key(key);
    run_frames(cpu, 2);
    cpu.release_key(key);
    run_frames(cpu, 2);
}

#[test]
fn the_title_screen_waits_for_key_6() {
    let mut cpu = superpong();
    run_frames(&mut cpu, 60);
    let title = cpu.framebuffer().to_string();
    run_frames(&mut cpu, 600);
    assert_eq!(cpu.framebuffer().to_string(), title);

    tap(&mut cpu, 0x5);
    assert_eq!(cpu.framebuffer().to_string(), title);

    tap(&mut cpu, SERVE);
    assert_ne!(cpu.framebuffer().to_string(), title);
}

#[test]
fn a_game_moves_the_ball_and_ends_on_the_game_over_screen() {
    let mut cpu = superpong();
    run_frames(&mut cpu, 10);
    tap(&mut cpu, SERVE);

    let first = cpu.framebuffer().to_string();
    run_frames(&mut cpu, 5);
    assert_ne!(cpu.framebuffer().to_string(), first);

    cpu.press_key(Nibble::new(PADDLE_LEFT).unwrap());
    run_frames(&mut cpu, 60 * 60);
    let game_over = cpu.framebuffer().to_string();
    run_frames(&mut cpu, 600);
    assert_eq!(cpu.framebuffer().to_string(), game_over);
}

#[test]
fn key_6_after_game_over_starts_a_new_game() {
    let mut cpu = superpong();
    run_frames(&mut cpu, 10);
    tap(&mut cpu, SERVE);
    run_frames(&mut cpu, 60 * 60);
    let game_over = cpu.framebuffer().to_string();

    tap(&mut cpu, SERVE);
    assert_ne!(cpu.framebuffer().to_string(), game_over);
    run_frames(&mut cpu, 5);
}
