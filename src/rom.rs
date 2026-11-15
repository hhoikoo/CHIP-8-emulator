mod error;

pub use error::LoadError;

use crate::{
    isa::opcode::Opcode,
    memory::{MEMORY_SIZE, Memory},
};

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
    pub fn new<T: Copy + Into<Opcode>>(segments: &[(usize, &[T])]) -> Result<Self, LoadError> {
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

    pub fn load_into_memory(&self, memory: &mut Memory) {
        for segment in self.segments.iter() {
            memory
                .write(segment.start, &segment.bytes)
                .expect("Rom::new checked that every segment fits in memory");
        }
    }

    fn parse_segments<T: Copy + Into<Opcode>>(
        segments: &[(usize, &[T])],
    ) -> Result<Vec<Segment>, LoadError> {
        segments
            .iter()
            .filter(|&(_, words)| !words.is_empty())
            .map(|&(start, words)| Segment::new(start, words))
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
    fn new<T: Copy + Into<Opcode>>(start: usize, words: &[T]) -> Result<Self, LoadError> {
        let bytes: Vec<u8> = words
            .iter()
            .flat_map(|&word| Into::<Opcode>::into(word).get().to_be_bytes())
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
}

#[cfg(test)]
mod tests;
