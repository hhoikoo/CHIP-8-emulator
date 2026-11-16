use super::*;

#[test]
fn new_framebuffer_is_dark() {
    let fb = Framebuffer::default();
    assert!(fb.rows().flatten().all(|&lit| !lit));
    assert_eq!(fb.rows().count(), HEIGHT);
}

#[test]
fn flip_toggles_and_reports_turning_a_lit_pixel_off() {
    let mut fb = Framebuffer::default();
    assert!(!fb.flip(3, 4));
    assert_eq!(fb.pixel(3, 4), Some(true));
    assert!(fb.flip(3, 4));
    assert_eq!(fb.pixel(3, 4), Some(false));
}

#[test]
fn pixel_outside_the_screen_is_none() {
    let fb = Framebuffer::default();
    assert_eq!(fb.pixel(WIDTH - 1, HEIGHT - 1), Some(false));
    assert_eq!(fb.pixel(WIDTH, 0), None);
    assert_eq!(fb.pixel(0, HEIGHT), None);
}

#[test]
fn clear_turns_every_pixel_off() {
    let mut fb = Framebuffer::default();
    fb.flip(0, 0);
    fb.flip(WIDTH - 1, HEIGHT - 1);
    fb.clear();
    assert_eq!(fb, Framebuffer::default());
}

#[test]
fn displays_as_rows_of_hashes_and_dots() {
    let mut fb = Framebuffer::default();
    fb.flip(0, 0);
    fb.flip(WIDTH - 1, 1);
    let text = fb.to_string();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), HEIGHT);
    assert!(lines.iter().all(|line| line.len() == WIDTH));
    assert_eq!(&lines[0][..2], "#.");
    assert_eq!(&lines[1][WIDTH - 2..], ".#");
    assert!(text.ends_with('\n'));
}
