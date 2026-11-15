use crate::isa::addr::Addr;
use crate::isa::instruction::Instruction;
use crate::isa::register::Register;

pub fn fn_add_twice() -> [Instruction; 3] {
    let add = Instruction::AddWithCarry {
        x: Register::V0,
        y: Register::V1,
    };
    [add, add, Instruction::Return]
}

pub fn fn_main() -> [Instruction; 5] {
    [
        Instruction::LoadImmediate {
            x: Register::V0,
            nn: 5,
        },
        Instruction::LoadImmediate {
            x: Register::V1,
            nn: 10,
        },
        Instruction::Call {
            addr: Addr::new(0x100).expect("Address in range"),
        },
        Instruction::Call {
            addr: Addr::new(0x100).expect("Address in range"),
        },
        Instruction::Halt,
    ]
}
