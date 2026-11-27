use std::time::Instant;

const BLINK_PERIOD: u128 = 666;

pub(super) struct BlinkTimer {
    started_at: Instant,
}

impl BlinkTimer {
    pub(super) fn start() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub(super) fn is_on(&self) -> bool {
        self.started_at.elapsed().as_millis() % BLINK_PERIOD < BLINK_PERIOD / 2
    }
}
