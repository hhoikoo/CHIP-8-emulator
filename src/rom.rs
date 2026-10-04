use core::fmt;
use std::{ops::Range, write};

use crate::cpu::Cpu;

#[derive(Debug)]
pub struct Rom {
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct Segment {
    start: usize,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum LoadError {
    OutOfBounds {
        addr: usize,
        byte_len: usize,
    },
    Overlap {
        first_start: usize,
        second_start: usize,
    },
}

impl Rom {
    pub fn new(segments: &[(usize, &[u16])]) -> Result<Self, LoadError> {
        let mut segments = Rom::parse_segments(segments)?;
        segments.sort_unstable_by_key(|segment| segment.start);

        if let Some((first_start, second_start)) = Rom::find_overlapping_segments(&segments) {
            return Err(LoadError::Overlap {
                first_start,
                second_start,
            });
        }
        Ok(Self { segments })
    }

    pub fn load_into_memory(&self, memory: &mut [u8; Cpu::MEMORY_SIZE]) {
        for segment in self.segments.iter() {
            memory[segment.range()].copy_from_slice(&segment.bytes);
        }
    }

    fn parse_segments(segments: &[(usize, &[u16])]) -> Result<Vec<Segment>, LoadError> {
        segments
            .iter()
            .filter(|&(_, opcodes)| !opcodes.is_empty())
            .map(|&(start, opcodes)| Segment::new(start, opcodes))
            .collect::<Result<_, _>>()
    }

    fn find_overlapping_segments(segments: &[Segment]) -> Option<(usize, usize)> {
        segments.windows(2).find_map(|window| {
            let segment_0 = &window[0];
            let segment_1 = &window[1];
            if segment_1.start < segment_0.end() {
                return Some((segment_0.start, segment_1.start));
            }
            None
        })
    }
}

impl Segment {
    fn new(start: usize, opcodes: &[u16]) -> Result<Self, LoadError> {
        let bytes: Vec<u8> = opcodes
            .iter()
            .flat_map(|opcode| opcode.to_be_bytes())
            .collect();

        start
            .checked_add(bytes.len())
            .filter(|&end| end <= Cpu::MEMORY_SIZE)
            .ok_or(LoadError::OutOfBounds {
                addr: start,
                byte_len: bytes.len(),
            })?;

        Ok(Self { start, bytes })
    }

    fn end(&self) -> usize {
        self.start + self.bytes.len()
    }

    fn range(&self) -> Range<usize> {
        self.start..self.end()
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::OutOfBounds { addr, byte_len } => write!(
                f,
                "Segment at {addr:#06x} ({byte_len} bytes) doesn't fit in memory"
            ),
            LoadError::Overlap {
                first_start,
                second_start,
            } => write!(
                f,
                "Segments at {first_start:#06x} and {second_start:#06x} overlap"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests {
    use std::matches;

    use super::*;

    fn build(s: &[(usize, &[u16])]) -> Result<Rom, LoadError> {
        Rom::new(s)
    }

    fn loaded(s: &[(usize, &[u16])]) -> [u8; Cpu::MEMORY_SIZE] {
        let mut mem = [0u8; Cpu::MEMORY_SIZE];
        build(s).unwrap().load_into_memory(&mut mem);
        mem
    }

    fn is_overlap(r: Result<Rom, LoadError>, a: usize, b: usize) -> bool {
        matches!(r, Err(LoadError::Overlap { first_start, second_start }) if first_start == a && second_start == b)
    }

    #[test]
    fn empty_rom_loads_nothing() {
        assert_eq!(loaded(&[]), [0u8; Cpu::MEMORY_SIZE]);
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
        // 100 two-byte segments spaced 4 apart, in reverse order
        let ops = [0xABCDu16];
        let segs: Vec<(usize, &[u16])> =
            (0..100).rev().map(|i| (0x200 + i * 4, &ops[..])).collect();
        assert!(build(&segs).is_ok());
    }

    #[test]
    fn exact_fit_at_end_of_memory() {
        assert!(build(&[(Cpu::MEMORY_SIZE - 2, &[1])]).is_ok());
    }

    #[test]
    fn one_byte_past_end() {
        assert!(matches!(
            build(&[(Cpu::MEMORY_SIZE - 1, &[1])]),
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
        // 0x60, 0xA2, 0x2A, 0x12, 0x34
        assert_eq!(mem.iter().filter(|&&b| b != 0).count(), 5);
    }

    #[test]
    fn last_two_bytes_of_memory() {
        let mem = loaded(&[(Cpu::MEMORY_SIZE - 2, &[0xBEEF])]);
        assert_eq!(&mem[Cpu::MEMORY_SIZE - 2..], &[0xBE, 0xEF]);
    }

    #[test]
    fn load_does_not_clear_existing_memory() {
        let rom = build(&[(0x10, &[0x1111])]).unwrap();
        let mut mem = [0xEEu8; Cpu::MEMORY_SIZE];
        rom.load_into_memory(&mut mem);
        assert_eq!(&mem[0x10..0x12], &[0x11, 0x11]);
        assert_eq!(mem[0x0F], 0xEE);
        assert_eq!(mem[0x12], 0xEE);
    }

    #[test]
    fn rom_is_reusable_across_loads() {
        let rom = build(&[(0x200, &[0x1234])]).unwrap();
        let (mut a, mut b) = ([0u8; Cpu::MEMORY_SIZE], [0u8; Cpu::MEMORY_SIZE]);
        rom.load_into_memory(&mut a);
        rom.load_into_memory(&mut b);
        assert_eq!(a, b);
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
}
