use super::*;
use crate::isa::{addr::Addr, nibble::Nibble, register::Register};

fn nibble(value: u8) -> Nibble {
    Nibble::new(value).unwrap()
}

fn all_nibbles() -> impl Iterator<Item = Nibble> {
    (0..=0xF).map(nibble)
}

fn all_registers() -> impl Iterator<Item = Register> {
    (0..=0xF).map(|n| Register::try_from(n).unwrap())
}

#[test]
fn nibbles_are_high_to_low() {
    assert_eq!(Opcode::new(0xABCD).nibbles(), [0xA, 0xB, 0xC, 0xD]);
    assert_eq!(Opcode::new(0x0000).nibbles(), [0, 0, 0, 0]);
    assert_eq!(Opcode::new(0xFFFF).nibbles(), [0xF, 0xF, 0xF, 0xF]);
}

#[test]
fn nibbles_rebuild_every_opcode() {
    for value in 0..=u16::MAX {
        let [a, b, c, d] = Opcode::new(value).nibbles();
        let rebuilt = (a as u16) << 12 | (b as u16) << 8 | (c as u16) << 4 | d as u16;
        assert_eq!(rebuilt, value, "{value:#06x}");
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

#[test]
fn reads_each_layout() {
    let op = Opcode::new(0xABCD);
    let (a, b, c, d) = (nibble(0xA), Register::VB, Register::VC, nibble(0xD));

    assert_eq!(
        Nnn::from(op),
        Nnn {
            prefix: a,
            addr: Addr::new(0xBCD).unwrap()
        }
    );
    assert_eq!(
        Xnn::from(op),
        Xnn {
            prefix: a,
            x: b,
            nn: 0xCD
        }
    );
    assert_eq!(
        Xyn::from(op),
        Xyn {
            prefix: a,
            x: b,
            y: c,
            n: d
        }
    );
}

#[test]
fn every_opcode_survives_each_layout() {
    for value in 0..=u16::MAX {
        let op = Opcode::new(value);
        assert_eq!(Opcode::from(Nnn::from(op)), op, "Nnn {value:#06x}");
        assert_eq!(Opcode::from(Xnn::from(op)), op, "Xnn {value:#06x}");
        assert_eq!(Opcode::from(Xyn::from(op)), op, "Xyn {value:#06x}");
    }
}

#[test]
fn every_nnn_survives_packing() {
    for prefix in all_nibbles() {
        for addr in 0..=Addr::MAX.get() {
            let fields = Nnn {
                prefix,
                addr: Addr::new(addr).unwrap(),
            };
            assert_eq!(Nnn::from(Opcode::from(fields)), fields);
        }
    }
}

#[test]
fn every_xnn_survives_packing() {
    for prefix in all_nibbles() {
        for x in all_registers() {
            for nn in 0..=u8::MAX {
                let fields = Xnn { prefix, x, nn };
                assert_eq!(Xnn::from(Opcode::from(fields)), fields);
            }
        }
    }
}

#[test]
fn every_xyn_survives_packing() {
    for prefix in all_nibbles() {
        for x in all_registers() {
            for y in all_registers() {
                for n in all_nibbles() {
                    let fields = Xyn { prefix, x, y, n };
                    assert_eq!(Xyn::from(Opcode::from(fields)), fields);
                }
            }
        }
    }
}
