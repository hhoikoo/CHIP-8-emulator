use std::matches;

use super::*;

fn build(s: &[(usize, &[u16])]) -> Result<Rom, LoadError> {
    Rom::new(s)
}

fn bytes(memory: &Memory) -> [u8; MEMORY_SIZE] {
    memory.read(0, MEMORY_SIZE).unwrap().try_into().unwrap()
}

fn filled(byte: u8) -> Memory {
    let mut memory = Memory::default();
    memory.write(0, &[byte; MEMORY_SIZE]).unwrap();
    memory
}

fn loaded(s: &[(usize, &[u16])]) -> [u8; MEMORY_SIZE] {
    let mut memory = Memory::default();
    build(s).unwrap().load_into_memory(&mut memory);
    bytes(&memory)
}

fn is_overlap(r: Result<Rom, LoadError>, a: usize, b: usize) -> bool {
    matches!(r, Err(LoadError::Overlap { first_start, second_start }) if first_start == a && second_start == b)
}

#[test]
fn empty_rom_loads_nothing() {
    assert_eq!(loaded(&[]), [0u8; MEMORY_SIZE]);
}

#[test]
fn disjoint_unsorted_ok() {
    assert!(build(&[(0x300, &[1]), (0x200, &[1, 2])]).is_ok());
}

#[test]
fn adjacent_ok() {
    assert!(build(&[(0x200, &[1, 2]), (0x204, &[3])]).is_ok());
}

#[test]
fn one_byte_overlap() {
    assert!(is_overlap(
        build(&[(0x200, &[1, 2]), (0x203, &[3])]),
        0x200,
        0x203
    ));
}

#[test]
fn overlap_input_order_independent() {
    assert!(is_overlap(
        build(&[(0x203, &[3]), (0x200, &[1, 2])]),
        0x200,
        0x203
    ));
}

#[test]
fn identical_start() {
    assert!(matches!(
        build(&[(0x200, &[1]), (0x200, &[2])]),
        Err(LoadError::Overlap { .. })
    ));
}

#[test]
fn contained() {
    assert!(is_overlap(
        build(&[(0x200, &[1, 2, 3, 4]), (0x202, &[9])]),
        0x200,
        0x202
    ));
}

#[test]
fn overlap_not_first_pair() {
    assert!(is_overlap(
        build(&[(0x200, &[1]), (0x300, &[1, 2]), (0x302, &[3])]),
        0x300,
        0x302
    ));
}

#[test]
fn long_segment_vs_non_neighbor() {
    let big = [0u16; 16];
    assert!(matches!(
        build(&[(0x200, &big), (0x210, &[1]), (0x214, &[1])]),
        Err(LoadError::Overlap { .. })
    ));
}

#[test]
fn many_segments_shuffled_no_false_positive() {
    let ops = [0xABCDu16];
    let segs: Vec<(usize, &[u16])> = (0..100).rev().map(|i| (0x200 + i * 4, &ops[..])).collect();
    assert!(build(&segs).is_ok());
}

#[test]
fn exact_fit_at_end_of_memory() {
    assert!(build(&[(MEMORY_SIZE - 2, &[1])]).is_ok());
}

#[test]
fn one_byte_past_end() {
    assert!(matches!(
        build(&[(MEMORY_SIZE - 1, &[1])]),
        Err(LoadError::OutOfBounds {
            addr: 4095,
            byte_len: 2
        })
    ));
}

#[test]
fn usize_max_start_no_panic() {
    assert!(matches!(
        build(&[(usize::MAX, &[1])]),
        Err(LoadError::OutOfBounds { .. })
    ));
}

#[test]
fn empty_segments_ignored_everywhere() {
    assert!(build(&[(0x200, &[]), (0x200, &[1]), (0x201, &[]), (99999, &[])]).is_ok());
    assert!(build(&[(0x200, &[1]), (0x200, &[])]).is_ok());
}

#[test]
fn big_endian_and_untouched_bytes_zero() {
    let mem = loaded(&[(0x300, &[0x1234]), (0x200, &[0x6000, 0xA22A])]);
    assert_eq!(&mem[0x200..0x204], &[0x60, 0x00, 0xA2, 0x2A]);
    assert_eq!(&mem[0x300..0x302], &[0x12, 0x34]);
    let nonzero: Vec<u8> = mem.iter().copied().filter(|&b| b != 0).collect();
    assert_eq!(nonzero, [0x60, 0xA2, 0x2A, 0x12, 0x34]);
}

#[test]
fn last_two_bytes_of_memory() {
    let mem = loaded(&[(MEMORY_SIZE - 2, &[0xBEEF])]);
    assert_eq!(&mem[MEMORY_SIZE - 2..], &[0xBE, 0xEF]);
}

#[test]
fn load_does_not_clear_existing_memory() {
    let rom = build(&[(0x10, &[0x1111])]).unwrap();
    let mut memory = filled(0xEE);
    rom.load_into_memory(&mut memory);
    let mem = bytes(&memory);
    assert_eq!(&mem[0x10..0x12], &[0x11, 0x11]);
    assert_eq!(mem[0x0F], 0xEE);
    assert_eq!(mem[0x12], 0xEE);
}

#[test]
fn rom_is_reusable_across_loads() {
    let rom = build(&[(0x200, &[0x1234])]).unwrap();
    let (mut a, mut b) = (Memory::default(), Memory::default());
    rom.load_into_memory(&mut a);
    rom.load_into_memory(&mut b);
    assert_eq!(bytes(&a), bytes(&b));
}

#[test]
fn error_display() {
    let e = build(&[(0x200, &[1, 2]), (0x202, &[3])]).err().unwrap();
    assert_eq!(e.to_string(), "Segments at 0x0200 and 0x0202 overlap");
    let e = build(&[(0xFFF, &[1])]).err().unwrap();
    assert_eq!(
        e.to_string(),
        "Segment at 0x0fff (2 bytes) doesn't fit in memory"
    );
}
