use super::*;

#[test]
fn new_and_get_round_trip_every_value() {
    for value in [0, 0x200, 0xFFF, 0x1000, u16::MAX] {
        assert_eq!(ProgramCounter::new(value).get(), value);
        assert_eq!(ProgramCounter::from(value).get(), value);
    }
}

#[test]
fn advance_moves_to_the_next_instruction() {
    let mut pc = ProgramCounter::new(0x200);
    pc.advance();
    assert_eq!(pc.get(), 0x202);
}

#[test]
fn advance_can_step_past_the_end_of_memory() {
    let mut pc = ProgramCounter::new(0xFFE);
    pc.advance();
    assert_eq!(pc.get(), 0x1000);
}

#[test]
fn from_addr_keeps_the_address() {
    assert_eq!(ProgramCounter::from(Addr::MAX).get(), 0xFFF);
}

#[test]
fn converts_into_u16_and_usize() {
    let pc = ProgramCounter::new(0x1000);
    assert_eq!(u16::from(pc), 0x1000);
    assert_eq!(usize::from(pc), 0x1000);
}

#[test]
fn formats_as_lower_hex() {
    assert_eq!(format!("{:#06x}", ProgramCounter::new(0xABC)), "0x0abc");
}
