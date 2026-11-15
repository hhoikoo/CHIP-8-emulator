use std::{
    env,
    error::Error,
    path::Path,
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use chip8_emulator::{
    cpu::Cpu,
    examples::{FN_ADD_TWICE_ADDR, fn_add_twice, fn_main},
    isa::register::Register,
    memory::PROGRAM_START,
    rom::Rom,
};

const DEMO_CYCLE_BUDGET: usize = 100;

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

    let outcome = cpu.run(|| stop.load(Ordering::Relaxed));
    print_registers(&cpu)?;
    Ok(outcome?)
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
