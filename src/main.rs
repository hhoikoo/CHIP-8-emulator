use chip8_emulator::{
    cpu::Cpu,
    examples::{fn_add_twice, fn_main},
    isa::register::Register,
    rom::Rom,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom = Rom::new(&[(0x0000, &fn_main()), (0x0100, &fn_add_twice())])?;
    let mut cpu = Cpu::new(&rom);

    cpu.run()?;

    let register_0 = cpu.get_register_value(Register::V0);
    assert_eq!(register_0, 45);
    println!("5 + (2 * 10) + (2 * 10) = {}", register_0);

    Ok(())
}
