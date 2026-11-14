use super::*;

#[test]
fn max_is_the_last_12_bit_address() {
    assert_eq!(Addr::MAX.get(), 0xFFF);
}

#[test]
fn new_accepts_the_whole_12_bit_range() {
    assert_eq!(Addr::new(0x000).map(Addr::get), Ok(0x000));
    assert_eq!(Addr::new(0xABC).map(Addr::get), Ok(0xABC));
    assert_eq!(Addr::new(0xFFF).map(Addr::get), Ok(0xFFF));
}

#[test]
fn new_rejects_anything_past_0xfff() {
    for value in [0x1000, 0x2ABC, u16::MAX] {
        assert_eq!(Addr::new(value), Err(AddrOutOfRange { value }));
    }
}

#[test]
fn try_from_agrees_with_new_for_every_u16() {
    for value in 0..=u16::MAX {
        assert_eq!(Addr::try_from(value), Addr::new(value), "{value:#06x}");
    }
}

#[test]
fn converts_back_to_u16_and_usize() {
    let addr = Addr::new(0x2A0).unwrap();
    assert_eq!(u16::from(addr), 0x2A0);
    assert_eq!(usize::from(addr), 0x2A0);
}

#[test]
fn orders_by_address() {
    assert!(Addr::new(0x100).unwrap() < Addr::new(0x200).unwrap());
    assert!(Addr::new(0x000).unwrap() < Addr::MAX);
}

#[test]
fn formats_as_hex() {
    let addr = Addr::new(0xABC).unwrap();
    assert_eq!(format!("{addr:#06x}"), "0x0abc");
    assert_eq!(format!("{addr:x}"), "abc");
}

#[test]
fn out_of_range_error_message() {
    assert_eq!(
        AddrOutOfRange { value: 0x1000 }.to_string(),
        "Address 0x1000 is out of range (expected 0x000 to 0xFFF)"
    );
}
