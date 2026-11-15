use crate::isa::instruction::Instruction;

pub fn fn_add_twice() -> [u16; 3] {
    [0x8014, 0x8014, Instruction::Return.encode().get()]
}
