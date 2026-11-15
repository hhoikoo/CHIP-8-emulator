use super::*;

#[test]
fn max_is_0xf() {
    assert_eq!(Nibble::MAX.get(), 0xF);
}

#[test]
fn new_accepts_0x0_to_0xf() {
    for value in 0x0..=0xF {
        assert_eq!(Nibble::new(value).map(Nibble::get), Ok(value));
    }
}

#[test]
fn new_rejects_0x10_and_above() {
    for value in 0x10..=u8::MAX {
        assert_eq!(Nibble::new(value), Err(NibbleOutOfRange { value }));
    }
}

#[test]
fn try_from_agrees_with_new_for_every_u8() {
    for value in 0..=u8::MAX {
        assert_eq!(Nibble::try_from(value), Nibble::new(value), "{value:#04x}");
    }
}

#[test]
fn converts_back_to_u8_and_u16() {
    let nibble = Nibble::new(0xA).unwrap();
    assert_eq!(u8::from(nibble), 0xA);
    assert_eq!(u16::from(nibble), 0xA);
}

#[test]
fn out_of_range_error_message() {
    assert_eq!(
        NibbleOutOfRange { value: 0x1A }.to_string(),
        "Nibble 0x1a is out of range (expected 0x0 to 0xF)"
    );
}

#[test]
fn expect_new_accepts_0x0_to_0xf() {
    for value in 0x0..=0xF {
        assert_eq!(Nibble::expect_new(value).get(), value);
    }
}

#[test]
#[should_panic(expected = "value must be 0x0 to 0xF")]
fn expect_new_panics_above_0xf() {
    Nibble::expect_new(0x10);
}

#[test]
fn expect_new_works_in_const_context() {
    const PREFIX: Nibble = Nibble::expect_new(0x6);
    assert_eq!(PREFIX.get(), 0x6);
}
