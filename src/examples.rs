use crate::isa::addr::Addr;
use crate::isa::instruction::Instruction;
use crate::isa::register::Register;

pub const FN_ADD_TWICE_ADDR: Addr = Addr::expect_new(0x300);

pub fn fn_add_twice() -> [Instruction; 3] {
    let add = Instruction::AddWithCarry {
        x: Register::V0,
        y: Register::V1,
    };
    [add, add, Instruction::Return]
}

pub fn fn_main(start_addr: Addr) -> [Instruction; 5] {
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
            addr: FN_ADD_TWICE_ADDR,
        },
        Instruction::Call {
            addr: FN_ADD_TWICE_ADDR,
        },
        Instruction::Jump {
            addr: Addr::expect_new(start_addr.get() + 8),
        },
    ]
}
