use super::*;

fn key(value: u8) -> Nibble {
    Nibble::new(value).unwrap()
}

fn pressed(keypad: &Keypad) -> Vec<u8> {
    (0..=0xF).filter(|&k| keypad.is_pressed(key(k))).collect()
}

#[test]
fn new_keypad_has_no_key_pressed() {
    assert_eq!(pressed(&Keypad::default()), []);
}

#[test]
fn press_holds_only_that_key_down() {
    let mut keypad = Keypad::default();
    keypad.press(key(0xA));
    assert_eq!(pressed(&keypad), [0xA]);
}

#[test]
fn keys_stay_down_until_released() {
    let mut keypad = Keypad::default();
    keypad.press(key(0x0));
    keypad.press(key(0xF));
    keypad.release(key(0x0));
    assert_eq!(pressed(&keypad), [0xF]);
}

#[test]
fn pressing_twice_or_releasing_an_up_key_changes_nothing() {
    let mut keypad = Keypad::default();
    keypad.press(key(0x5));
    keypad.press(key(0x5));
    keypad.release(key(0x6));
    assert_eq!(pressed(&keypad), [0x5]);
}
