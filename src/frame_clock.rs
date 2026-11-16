use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct FrameClock {
    period: Duration,
    max_frames: u32,
    last: Instant,
    lag: Duration,
}

impl FrameClock {
    pub fn new(period: Duration, max_frames: u32, start: Instant) -> Self {
        Self {
            period,
            max_frames,
            last: start,
            lag: Duration::ZERO,
        }
    }

    pub fn frames_due(&mut self, now: Instant) -> u32 {
        self.lag += now.saturating_duration_since(self.last);
        self.last = now;

        let mut frames = 0;
        while self.lag >= self.period && frames < self.max_frames {
            self.lag -= self.period;
            frames += 1;
        }
        if self.lag >= self.period {
            self.lag = Duration::ZERO;
        }
        frames
    }
}

#[cfg(test)]
mod tests;
