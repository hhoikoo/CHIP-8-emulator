use super::*;
use crate::memory::PROGRAM_START;

#[test]
fn glyphs_are_packed_from_the_font_start() {
    assert_eq!(glyph_addr(Nibble::expect_new(0x0)), 0x050);
    assert_eq!(glyph_addr(Nibble::expect_new(0x1)), 0x055);
    assert_eq!(glyph_addr(Nibble::expect_new(0xF)), 0x09B);
}

#[test]
fn the_font_ends_before_the_program_start() {
    let end = usize::from(FONT_START) + GLYPHS.as_flattened().len();
    assert_eq!(end, 0x0A0);
    assert!(end <= usize::from(PROGRAM_START));
}

#[test]
fn glyphs_are_four_pixels_wide() {
    assert!(GLYPHS.as_flattened().iter().all(|row| row & 0x0F == 0));
}

#[test]
fn every_glyph_has_a_lit_top_and_bottom_row() {
    for (digit, glyph) in GLYPHS.iter().enumerate() {
        assert_ne!(glyph[0], 0, "digit {digit:x}");
        assert_ne!(glyph[4], 0, "digit {digit:x}");
    }
}
