use super::*;
use crate::isa::register::Register::*;

const SAMPLE: Opcode = Opcode::new(0xABCD);

#[test]
fn nibbles_are_high_to_low() {
    assert_eq!(SAMPLE.nibbles(), (0xA, 0xB, 0xC, 0xD));
    assert_eq!(Opcode::new(0x0000).nibbles(), (0, 0, 0, 0));
    assert_eq!(Opcode::new(0xFFFF).nibbles(), (0xF, 0xF, 0xF, 0xF));
}

#[test]
fn named_fields() {
    assert_eq!(SAMPLE.x(), VB);
    assert_eq!(SAMPLE.y(), VC);
    assert_eq!(SAMPLE.n(), 0xD);
    assert_eq!(SAMPLE.nn(), 0xCD);
    assert_eq!(SAMPLE.nnn(), Addr::new(0xBCD).unwrap());
}

#[test]
fn fields_agree_with_the_bits_for_every_opcode() {
    for value in 0..=u16::MAX {
        let op = Opcode::new(value);
        let (a, b, c, d) = op.nibbles();
        let rebuilt = (a as u16) << 12 | (b as u16) << 8 | (c as u16) << 4 | d as u16;
        assert_eq!(rebuilt, value, "{value:#06x}");

        assert_eq!(op.x() as u8, b);
        assert_eq!(op.y() as u8, c);
        assert_eq!(op.n(), d);
        assert_eq!(op.nn(), (value & 0xFF) as u8);
        assert_eq!(op.nnn().get(), value & 0x0FFF);
    }
}

#[test]
fn converts_to_and_from_u16() {
    assert_eq!(Opcode::from(0x6A05), Opcode::new(0x6A05));
    assert_eq!(u16::from(Opcode::new(0x6A05)), 0x6A05);
    assert_eq!(Opcode::new(0x6A05).get(), 0x6A05);
}

#[test]
fn formats_as_hex() {
    let op = Opcode::new(0x00EE);
    assert_eq!(format!("{op:#06x}"), "0x00ee");
    assert_eq!(format!("{op:x}"), "ee");
}
