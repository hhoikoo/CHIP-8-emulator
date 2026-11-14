mod error;

pub use error::LoadError;

use std::ops::Range;

use crate::constant::MEMORY_SIZE;

#[derive(Debug)]
pub struct Rom {
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct Segment {
    start: usize,
    bytes: Vec<u8>,
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

    pub fn load_into_memory(&self, memory: &mut [u8; MEMORY_SIZE]) {
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
            .filter(|&end| end <= MEMORY_SIZE)
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

#[cfg(test)]
mod tests;
