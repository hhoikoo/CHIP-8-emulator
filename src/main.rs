use chip8_emulator::{
    cpu::Cpu, example_fn::FN_ADD_TWICE, op::OP_HALT, register::Register, rom::Rom,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fn_main = [
        0x6005, // LOAD IMMEDIATE to register 0 value 5
        0x610A, // LOAD IMMEDIATE to register 1 value 10 (0xA)
        0x2100, // CALL function at 0x100
        0x2100, // CALL function at 0x100
        OP_HALT,
    ];
    let rom = Rom::new(&[
        (0x0000, &fn_main),
        (0x0100, &FN_ADD_TWICE), // Load function FN_ADD_TWICE at address 0x0100
    ])?;
    let mut cpu = Cpu::new(&rom);

    cpu.run()?;

    let register_0 = cpu.get_register_value(Register::V0);
    assert_eq!(register_0, 45);
    println!("5 + (2 * 10) + (2 * 10) = {}", register_0);

    Ok(())
}
