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

const START: usize = 0x200;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - START;

fn loaded_bytes(rom: &Rom) -> [u8; MEMORY_SIZE] {
    let mut memory = Memory::default();
    rom.load_into_memory(&mut memory);
    bytes(&memory)
}

#[test]
fn from_bytes_loads_at_the_program_start() {
    assert_eq!(usize::from(PROGRAM_START), START);
    let mem = loaded_bytes(&Rom::from_bytes(&[0x60, 0x05, 0x00, 0x00]).unwrap());
    assert_eq!(&mem[START..START + 4], &[0x60, 0x05, 0x00, 0x00]);
    assert!(mem[..START].iter().all(|&b| b == 0));
    assert!(mem[START + 4..].iter().all(|&b| b == 0));
}

#[test]
fn from_bytes_accepts_an_odd_length() {
    let mem = loaded_bytes(&Rom::from_bytes(&[0xAB, 0xCD, 0xEF]).unwrap());
    assert_eq!(&mem[START..START + 4], &[0xAB, 0xCD, 0xEF, 0x00]);
}

#[test]
fn from_bytes_with_no_bytes_loads_nothing() {
    let mem = loaded_bytes(&Rom::from_bytes(&[]).unwrap());
    assert_eq!(mem, [0u8; MEMORY_SIZE]);
}

#[test]
fn from_bytes_fills_memory_up_to_the_end() {
    let rom_bytes = [0x5A; MAX_ROM_SIZE];
    let mem = loaded_bytes(&Rom::from_bytes(&rom_bytes).unwrap());
    assert_eq!(&mem[START..], &rom_bytes[..]);
}

#[test]
fn from_bytes_one_byte_too_big_is_out_of_bounds() {
    assert!(matches!(
        Rom::from_bytes(&[0; MAX_ROM_SIZE + 1]),
        Err(LoadError::OutOfBounds {
            addr: START,
            byte_len
        }) if byte_len == MAX_ROM_SIZE + 1
    ));
}

struct TempRomFile(std::path::PathBuf);

impl TempRomFile {
    fn new(name: &str, contents: &[u8]) -> TempRomFile {
        let path =
            std::env::temp_dir().join(format!("chip8-emulator-{}-{name}.ch8", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        TempRomFile(path)
    }
}

impl Drop for TempRomFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn from_file_reads_raw_bytes() {
    let file = TempRomFile::new("raw-bytes", &[0x12, 0x34, 0x56]);
    let mem = loaded_bytes(&Rom::from_file(&file.0).unwrap());
    assert_eq!(&mem[START..START + 3], &[0x12, 0x34, 0x56]);
}

#[test]
fn from_file_missing_is_a_read_error_naming_the_path() {
    let path = std::env::temp_dir().join("chip8-emulator-does-not-exist.ch8");
    let err = Rom::from_file(&path).unwrap_err();
    assert!(matches!(err, RomFileError::Read { .. }));
    assert!(
        err.to_string()
            .starts_with(&format!("Couldn't read ROM file {}: ", path.display()))
    );
}

#[test]
fn from_file_too_big_is_a_load_error() {
    let file = TempRomFile::new("too-big", &[0; MAX_ROM_SIZE + 1]);
    let err = Rom::from_file(&file.0).unwrap_err();
    assert!(matches!(
        err,
        RomFileError::Load {
            source: LoadError::OutOfBounds { .. },
            ..
        }
    ));
    assert_eq!(
        err.to_string(),
        format!(
            "Couldn't load ROM file {}: Segment at 0x0200 (3585 bytes) doesn't fit in memory",
            file.0.display()
        )
    );
}
