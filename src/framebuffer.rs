use std::fmt;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer([[bool; WIDTH]; HEIGHT]);

impl Framebuffer {
    pub fn pixel(&self, x: usize, y: usize) -> Option<bool> {
        self.0.get(y)?.get(x).copied()
    }

    pub fn rows(&self) -> impl Iterator<Item = &[bool; WIDTH]> {
        self.0.iter()
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn flip(&mut self, x: usize, y: usize) -> bool {
        let pixel = &mut self.0[y % HEIGHT][x % WIDTH];
        let was_lit = *pixel;
        *pixel = !was_lit;
        was_lit
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self([[false; WIDTH]; HEIGHT])
    }
}

impl fmt::Display for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.rows() {
            let line: String = row.iter().map(|&lit| if lit { '#' } else { '.' }).collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
