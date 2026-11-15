use super::*;

fn memory_with(start: usize, bytes: &[u8]) -> Memory {
    let mut memory = Memory::default();
    memory.write(start, bytes).unwrap();
    memory
}

#[test]
fn new_memory_is_zeroed() {
    let memory = Memory::default();
    assert_eq!(memory.read(0, MEMORY_SIZE), Some(&[0u8; MEMORY_SIZE][..]));
}

#[test]
fn write_then_read_round_trips() {
    let memory = memory_with(0x200, &[1, 2, 3]);
    assert_eq!(memory.read(0x200, 3), Some(&[1, 2, 3][..]));
}

#[test]
fn write_leaves_neighbouring_bytes_untouched() {
    let memory = memory_with(0x200, &[0xAA, 0xBB]);
    assert_eq!(memory.read(0x1FF, 4), Some(&[0, 0xAA, 0xBB, 0][..]));
}

#[test]
fn write_overwrites_existing_bytes() {
    let mut memory = memory_with(0x200, &[1, 2, 3]);
    memory.write(0x201, &[9]).unwrap();
    assert_eq!(memory.read(0x200, 3), Some(&[1, 9, 3][..]));
}

#[test]
fn write_fits_exactly_at_the_end() {
    let memory = memory_with(MEMORY_SIZE - 2, &[0xBE, 0xEF]);
    assert_eq!(memory.read(MEMORY_SIZE - 2, 2), Some(&[0xBE, 0xEF][..]));
}

#[test]
fn write_past_the_end_fails_and_changes_nothing() {
    let mut memory = Memory::default();
    assert_eq!(memory.write(MEMORY_SIZE - 1, &[1, 2]), None);
    assert_eq!(memory.read(MEMORY_SIZE - 1, 1), Some(&[0][..]));
}

#[test]
fn write_with_overflowing_end_fails() {
    let mut memory = Memory::default();
    assert_eq!(memory.write(usize::MAX, &[1]), None);
}

#[test]
fn empty_write_is_ok_up_to_the_end() {
    let mut memory = Memory::default();
    assert_eq!(memory.write(MEMORY_SIZE, &[]), Some(()));
    assert_eq!(memory.write(MEMORY_SIZE + 1, &[]), None);
}

#[test]
fn read_past_the_end_fails() {
    let memory = Memory::default();
    assert_eq!(memory.read(MEMORY_SIZE - 1, 2), None);
    assert_eq!(memory.read(MEMORY_SIZE, 1), None);
    assert_eq!(memory.read(usize::MAX, 1), None);
}

#[test]
fn empty_read_is_ok_up_to_the_end() {
    let memory = Memory::default();
    assert_eq!(memory.read(MEMORY_SIZE, 0), Some(&[][..]));
    assert_eq!(memory.read(MEMORY_SIZE + 1, 0), None);
}

#[test]
fn read_opcode_is_big_endian() {
    let memory = memory_with(0x200, &[0x12, 0x34]);
    assert_eq!(memory.read_opcode(0x200), Some(Opcode::new(0x1234)));
}

#[test]
fn read_opcode_at_an_odd_address() {
    let memory = memory_with(0x201, &[0xAB, 0xCD]);
    assert_eq!(memory.read_opcode(0x201), Some(Opcode::new(0xABCD)));
}

#[test]
fn read_opcode_in_the_last_two_bytes() {
    let memory = memory_with(MEMORY_SIZE - 2, &[0x00, 0xEE]);
    assert_eq!(
        memory.read_opcode(MEMORY_SIZE - 2),
        Some(Opcode::new(0x00EE))
    );
}

#[test]
fn read_opcode_past_the_end_fails() {
    let memory = Memory::default();
    assert_eq!(memory.read_opcode(MEMORY_SIZE - 1), None);
    assert_eq!(memory.read_opcode(MEMORY_SIZE), None);
    assert_eq!(memory.read_opcode(usize::MAX), None);
}
