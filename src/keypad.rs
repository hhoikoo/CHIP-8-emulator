use crate::isa::nibble::Nibble;

pub const KEY_COUNT: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Keypad([bool; KEY_COUNT]);

impl Keypad {
    pub fn press(&mut self, key: Nibble) {
        self.0[usize::from(key)] = true;
    }

    pub fn release(&mut self, key: Nibble) {
        self.0[usize::from(key)] = false;
    }

    pub fn is_pressed(&self, key: Nibble) -> bool {
        self.0[usize::from(key)]
    }
}

#[cfg(test)]
mod tests;
