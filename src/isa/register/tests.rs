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
