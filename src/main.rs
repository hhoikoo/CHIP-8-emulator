mod beeper;
mod cli;

use std::{
    env,
    error::Error,
    num::NonZeroUsize,
    path::Path,
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chip8_emulator::{
    cpu::{Cpu, Quirks},
    examples::{FN_ADD_TWICE_ADDR, fn_add_twice, fn_main},
    frame_clock::FrameClock,
    framebuffer::{Framebuffer, HEIGHT, WIDTH},
    isa::{nibble::Nibble, register::Register},
    memory::PROGRAM_START,
    rom::Rom,
};

use beeper::Beeper;
use minifb::{Key, Scale, Window, WindowOptions};

const DEMO_CYCLE_BUDGET: usize = 100;
const FRAME_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 60);
const MAX_CATCH_UP_FRAMES: u32 = 5;
const LIT: u32 = 0x00E8_E8E8;
const DARK: u32 = 0x0018_1818;
const KEYMAP: [(Key, Nibble); 16] = [
    (Key::Key1, keypad_key(0x1)),
    (Key::Key2, keypad_key(0x2)),
    (Key::Key3, keypad_key(0x3)),
    (Key::Key4, keypad_key(0xC)),
    (Key::Q, keypad_key(0x4)),
    (Key::W, keypad_key(0x5)),
    (Key::E, keypad_key(0x6)),
    (Key::R, keypad_key(0xD)),
    (Key::A, keypad_key(0x7)),
    (Key::S, keypad_key(0x8)),
    (Key::D, keypad_key(0x9)),
    (Key::F, keypad_key(0xE)),
    (Key::Z, keypad_key(0xA)),
    (Key::X, keypad_key(0x0)),
    (Key::C, keypad_key(0xB)),
    (Key::V, keypad_key(0xF)),
];

const fn keypad_key(value: u8) -> Nibble {
    match Nibble::new(value) {
        Ok(key) => key,
        Err(_) => panic!("keypad keys go from 0x0 to 0xF"),
    }
}

fn main() -> ExitCode {
    let result = match cli::Options::parse(env::args_os().skip(1)) {
        Ok(cli::Options {
            rom: Some(path),
            instructions_per_frame,
            quirks,
        }) => run_rom_file(&path, instructions_per_frame, quirks),
        Ok(cli::Options { rom: None, .. }) => run_demo(),
        Err(err) => Err(err.into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_rom_file(
    path: &Path,
    instructions_per_frame: NonZeroUsize,
    quirks: Quirks,
) -> Result<(), Box<dyn Error>> {
    let mut cpu = Cpu::with_quirks(&Rom::from_file(path)?, quirks).seeded(seed_from_clock());

    let stop = Arc::new(AtomicBool::new(false));
    let stop_from_handler = Arc::clone(&stop);
    ctrlc::set_handler(move || stop_from_handler.store(true, Ordering::Relaxed))?;

    let mut window = Window::new(
        &format!("CHIP-8: {}", path.display()),
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )?;

    let outcome = run_in_window(&mut cpu, &mut window, instructions_per_frame, &stop);
    print_registers(&cpu)?;
    outcome
}

fn run_in_window(
    cpu: &mut Cpu,
    window: &mut Window,
    instructions_per_frame: NonZeroUsize,
    stop: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    let mut clock = FrameClock::new(FRAME_PERIOD, MAX_CATCH_UP_FRAMES, Instant::now());
    let mut pixels = vec![DARK; WIDTH * HEIGHT];
    let beeper = Beeper::open()
        .inspect_err(|err| eprintln!("No sound: {err}"))
        .ok();

    while window.is_open() && !window.is_key_down(Key::Escape) && !stop.load(Ordering::Relaxed) {
        update_keypad(cpu, window);
        for _ in 0..clock.frames_due(Instant::now()) {
            cpu.run_for(instructions_per_frame.get())?;
            cpu.tick_timers();
        }
        if let Some(beeper) = &beeper {
            beeper.set_active(cpu.sound_active());
        }
        paint(cpu.framebuffer(), &mut pixels);
        window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
    }
    Ok(())
}

fn update_keypad(cpu: &mut Cpu, window: &Window) {
    for (key, keypad_key) in KEYMAP {
        if window.is_key_down(key) {
            cpu.press_key(keypad_key);
        } else {
            cpu.release_key(keypad_key);
        }
    }
}

fn seed_from_clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| {
            u64::try_from(since_epoch.as_nanos()).unwrap_or(u64::MAX)
        })
}

fn paint(framebuffer: &Framebuffer, pixels: &mut [u32]) {
    for (pixel, &lit) in pixels.iter_mut().zip(framebuffer.rows().flatten()) {
        *pixel = if lit { LIT } else { DARK };
    }
}

fn print_registers(cpu: &Cpu) -> Result<(), Box<dyn Error>> {
    println!("PC = {:#06x}", cpu.pc());
    for value in 0..=0xF {
        let register = Register::try_from(value)?;
        println!("{register:?} = {:#04x}", cpu.get_register_value(register));
    }
    Ok(())
}

fn run_demo() -> Result<(), Box<dyn Error>> {
    let rom = Rom::new(&[
        (usize::from(PROGRAM_START), &fn_main(PROGRAM_START)),
        (usize::from(FN_ADD_TWICE_ADDR), &fn_add_twice()),
    ])?;
    let mut cpu = Cpu::new(&rom);

    cpu.run_for(DEMO_CYCLE_BUDGET)?;

    let register_0 = cpu.get_register_value(Register::V0);
    assert_eq!(register_0, 45);
    println!("5 + (2 * 10) + (2 * 10) = {}", register_0);

    Ok(())
}
