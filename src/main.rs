use chip8_emulator::{
    cpu::Cpu,
    examples::{FN_ADD_TWICE_ADDR, fn_add_twice, fn_main},
    isa::register::Register,
    memory::PROGRAM_START,
    rom::Rom,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom = Rom::new(&[
        (usize::from(PROGRAM_START), &fn_main()),
        (usize::from(FN_ADD_TWICE_ADDR), &fn_add_twice()),
    ])?;
    let mut cpu = Cpu::new(&rom);

    cpu.run()?;

    let register_0 = cpu.get_register_value(Register::V0);
    assert_eq!(register_0, 45);
    println!("5 + (2 * 10) + (2 * 10) = {}", register_0);

    Ok(())
}
