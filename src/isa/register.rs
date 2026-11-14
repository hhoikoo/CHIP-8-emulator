mod error;

pub use error::RegisterOutOfRange;

use std::ops::{Index, IndexMut};

pub const NUM_REGISTERS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xA,
    VB = 0xB,
    VC = 0xC,
    VD = 0xD,
    VE = 0xE,
    VF = 0xF,
}

impl TryFrom<u8> for Register {
    type Error = RegisterOutOfRange;

    fn try_from(value: u8) -> Result<Self, RegisterOutOfRange> {
        match value {
            0x0 => Ok(Register::V0),
            0x1 => Ok(Register::V1),
            0x2 => Ok(Register::V2),
            0x3 => Ok(Register::V3),
            0x4 => Ok(Register::V4),
            0x5 => Ok(Register::V5),
            0x6 => Ok(Register::V6),
            0x7 => Ok(Register::V7),
            0x8 => Ok(Register::V8),
            0x9 => Ok(Register::V9),
            0xA => Ok(Register::VA),
            0xB => Ok(Register::VB),
            0xC => Ok(Register::VC),
            0xD => Ok(Register::VD),
            0xE => Ok(Register::VE),
            0xF => Ok(Register::VF),
            _ => Err(RegisterOutOfRange { value }),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Registers([u8; NUM_REGISTERS]);

impl Index<Register> for Registers {
    type Output = u8;

    fn index(&self, index: Register) -> &u8 {
        &self.0[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut u8 {
        &mut self.0[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Register::*;

    const IN_ORDER: [Register; NUM_REGISTERS] = [
        V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, VA, VB, VC, VD, VE, VF,
    ];

    #[test]
    fn discriminants_match_register_numbers() {
        for (n, r) in IN_ORDER.into_iter().enumerate() {
            assert_eq!(r as usize, n);
        }
    }

    #[test]
    fn try_from_u8_maps_each_number_to_its_register() {
        for (n, r) in IN_ORDER.into_iter().enumerate() {
            assert_eq!(Register::try_from(n as u8), Ok(r));
        }
    }

    #[test]
    fn try_from_u8_rejects_16_and_above() {
        for value in 0x10..=u8::MAX {
            assert_eq!(
                Register::try_from(value),
                Err(RegisterOutOfRange { value }),
                "{value:#04x}"
            );
        }
    }

    #[test]
    fn out_of_range_error_message() {
        assert_eq!(
            RegisterOutOfRange { value: 0x1A }.to_string(),
            "Register number 0x1a is out of range (expected 0x0 to 0xF)"
        );
    }

    #[test]
    fn new_registers_are_zeroed() {
        let regs = Registers::default();
        assert!(IN_ORDER.into_iter().all(|r| regs[r] == 0));
    }

    #[test]
    fn each_register_is_its_own_slot() {
        for target in IN_ORDER {
            let mut regs = Registers::default();
            regs[target] = 0xAB;
            for r in IN_ORDER {
                let want = if r == target { 0xAB } else { 0 };
                assert_eq!(regs[r], want, "wrote {target:?}, read {r:?}");
            }
        }
    }

    #[test]
    fn registers_compare_by_value() {
        let (mut a, mut b) = (Registers::default(), Registers::default());
        assert_eq!(a, b);

        a[V7] = 1;
        assert_ne!(a, b);

        b[V7] = 1;
        assert_eq!(a, b);
    }

    #[test]
    fn index_mut_updates_in_place() {
        let mut regs = Registers::default();
        regs[V3] = 10;
        regs[V3] += 5;
        assert_eq!(regs[V3], 15);
    }
}
