use std::{
    env,
    error::Error,
    path::Path,
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use chip8_emulator::{
    cpu::Cpu,
    examples::{FN_ADD_TWICE_ADDR, fn_add_twice, fn_main},
    frame_clock::FrameClock,
    framebuffer::{Framebuffer, HEIGHT, WIDTH},
    isa::register::Register,
    memory::PROGRAM_START,
    rom::Rom,
};

use minifb::{Key, Scale, Window, WindowOptions};

const DEMO_CYCLE_BUDGET: usize = 100;
const FRAME_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 60);
const INSTRUCTIONS_PER_FRAME: usize = 12;
const MAX_CATCH_UP_FRAMES: u32 = 5;
const LIT: u32 = 0x00E8_E8E8;
const DARK: u32 = 0x0018_1818;

fn main() -> ExitCode {
    let result = match env::args_os().nth(1) {
        Some(path) => run_rom_file(Path::new(&path)),
        None => run_demo(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_rom_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut cpu = Cpu::new(&Rom::from_file(path)?);

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

    let outcome = run_in_window(&mut cpu, &mut window, &stop);
    print_registers(&cpu)?;
    outcome
}

fn run_in_window(
    cpu: &mut Cpu,
    window: &mut Window,
    stop: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    let mut clock = FrameClock::new(FRAME_PERIOD, MAX_CATCH_UP_FRAMES, Instant::now());
    let mut pixels = vec![DARK; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) && !stop.load(Ordering::Relaxed) {
        for _ in 0..clock.frames_due(Instant::now()) {
            cpu.run_for(INSTRUCTIONS_PER_FRAME)?;
        }
        paint(cpu.framebuffer(), &mut pixels);
        window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
    }
    Ok(())
}

fn paint(framebuffer: &Framebuffer, pixels: &mut [u32]) {
    for (pixel, &lit) in pixels.iter_mut().zip(framebuffer.rows().flatten()) {
        *pixel = if lit { LIT } else { DARK };
    }
}

fn print_registers(cpu: &Cpu) -> Result<(), Box<dyn Error>> {
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
