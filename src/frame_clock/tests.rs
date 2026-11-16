use super::*;

const PERIOD: Duration = Duration::from_millis(10);

fn clock() -> (FrameClock, Instant) {
    let start = Instant::now();
    (FrameClock::new(PERIOD, 5, start), start)
}

#[test]
fn no_frame_is_due_before_a_period_passes() {
    let (mut c, start) = clock();
    assert_eq!(c.frames_due(start), 0);
    assert_eq!(c.frames_due(start + PERIOD - Duration::from_nanos(1)), 0);
}

#[test]
fn one_frame_is_due_per_period() {
    let (mut c, start) = clock();
    assert_eq!(c.frames_due(start + PERIOD), 1);
    assert_eq!(c.frames_due(start + PERIOD * 2), 1);
}

#[test]
fn leftover_time_carries_over_to_the_next_call() {
    let (mut c, start) = clock();
    assert_eq!(c.frames_due(start + PERIOD * 5 / 2), 2);
    assert_eq!(c.frames_due(start + PERIOD * 3), 1);
}

#[test]
fn a_display_twice_as_fast_gets_a_frame_every_other_refresh() {
    let (mut c, start) = clock();
    let frames: Vec<u32> = (1..=8)
        .map(|refresh| c.frames_due(start + PERIOD / 2 * refresh))
        .collect();
    assert_eq!(frames, [0, 1, 0, 1, 0, 1, 0, 1]);
}

#[test]
fn a_stall_runs_at_most_max_frames_and_drops_the_rest() {
    let (mut c, start) = clock();
    assert_eq!(c.frames_due(start + PERIOD * 100), 5);
    assert_eq!(c.frames_due(start + PERIOD * 100), 0);
    assert_eq!(c.frames_due(start + PERIOD * 101), 1);
}

#[test]
fn a_clock_going_backwards_counts_as_no_time() {
    let (mut c, start) = clock();
    assert_eq!(c.frames_due(start + PERIOD), 1);
    assert_eq!(c.frames_due(start), 0);
    assert_eq!(c.frames_due(start + PERIOD), 1);
}
