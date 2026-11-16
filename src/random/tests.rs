use super::*;

#[test]
fn matches_the_splitmix64_reference_output() {
    let mut random = Random::new(0);
    assert_eq!(random.next_u64(), 0xE220_A839_7B1D_CDAF);
    assert_eq!(random.next_u64(), 0x6E78_9E6A_A1B9_65F4);
}

#[test]
fn next_byte_is_the_high_byte_of_the_next_word() {
    let mut words = Random::new(7);
    let mut bytes = Random::new(7);
    for _ in 0..100 {
        assert_eq!(bytes.next_byte(), words.next_u64().to_be_bytes()[0]);
    }
}

#[test]
fn the_same_seed_gives_the_same_bytes() {
    let mut a = Random::new(42);
    let mut b = Random::new(42);
    let a: Vec<u8> = (0..32).map(|_| a.next_byte()).collect();
    let b: Vec<u8> = (0..32).map(|_| b.next_byte()).collect();
    assert_eq!(a, b);
}

#[test]
fn different_seeds_give_different_bytes() {
    let mut a = Random::new(1);
    let mut b = Random::new(2);
    let a: Vec<u8> = (0..32).map(|_| a.next_byte()).collect();
    let b: Vec<u8> = (0..32).map(|_| b.next_byte()).collect();
    assert_ne!(a, b);
}

#[test]
fn every_byte_value_comes_up() {
    let mut random = Random::new(0);
    let mut seen = [false; 256];
    for _ in 0..10_000 {
        seen[usize::from(random.next_byte())] = true;
    }
    assert!(seen.iter().all(|&s| s));
}
