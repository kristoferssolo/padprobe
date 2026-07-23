use super::{DriftMetrics, StickSide};
use std::time::{Duration, Instant};

pub(super) const COUNTDOWN_DURATION: Duration = Duration::from_secs(3);
pub(super) const SAMPLE_DURATION: Duration = Duration::from_secs(10);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(10);
const MAX_SAMPLES: usize = 2_000;

#[derive(Debug, Clone, Copy)]
pub enum DriftView<'metrics> {
    Ready,
    Countdown {
        remaining: Duration,
    },
    Sampling {
        elapsed: Duration,
        sample_count: usize,
    },
    Complete(&'metrics DriftMetrics),
}

#[derive(Debug, Clone)]
enum DriftState {
    Ready,
    Countdown {
        started_at: Instant,
    },
    Sampling {
        started_at: Instant,
        next_sample_at: Instant,
        samples: Vec<(f32, f32)>,
    },
    Complete(DriftMetrics),
}

#[derive(Debug, Clone)]
pub struct DriftTest {
    side: StickSide,
    device_id: Option<usize>,
    state: DriftState,
}

impl Default for DriftTest {
    fn default() -> Self {
        Self {
            side: StickSide::default(),
            device_id: None,
            state: DriftState::Ready,
        }
    }
}

impl DriftTest {
    #[must_use]
    pub const fn side(&self) -> StickSide {
        self.side
    }

    #[must_use]
    pub const fn device_id(&self) -> Option<usize> {
        self.device_id
    }

    #[must_use]
    pub const fn result(&self) -> Option<&DriftMetrics> {
        if let DriftState::Complete(metrics) = &self.state {
            Some(metrics)
        } else {
            None
        }
    }

    pub fn select_side(&mut self, side: StickSide) {
        if matches!(self.state, DriftState::Ready | DriftState::Complete(_)) {
            self.side = side;
            self.state = DriftState::Ready;
        }
    }

    pub fn start(&mut self, device_id: usize, now: Instant) {
        self.device_id = Some(device_id);
        self.state = DriftState::Countdown { started_at: now };
    }

    pub fn cancel(&mut self) {
        self.device_id = None;
        self.state = DriftState::Ready;
    }

    pub fn tick(&mut self, now: Instant, position: Option<(f32, f32)>) -> bool {
        if let DriftState::Countdown { started_at } = &self.state
            && now.saturating_duration_since(*started_at) >= COUNTDOWN_DURATION
        {
            self.state = DriftState::Sampling {
                started_at: now,
                next_sample_at: now,
                samples: Vec::with_capacity(
                    MAX_SAMPLES.min(
                        usize::try_from(SAMPLE_DURATION.as_millis() / SAMPLE_INTERVAL.as_millis())
                            .unwrap_or(MAX_SAMPLES),
                    ),
                ),
            };
        }

        let DriftState::Sampling {
            started_at,
            next_sample_at,
            samples,
        } = &mut self.state
        else {
            return false;
        };
        let elapsed = now.saturating_duration_since(*started_at);
        if elapsed >= SAMPLE_DURATION {
            let metrics = DriftMetrics::calculate(samples, SAMPLE_DURATION);
            if let Some(metrics) = metrics {
                self.state = DriftState::Complete(metrics);
                return true;
            }
            self.cancel();
            return false;
        }

        if now >= *next_sample_at
            && samples.len() < MAX_SAMPLES
            && let Some(position) = position
        {
            samples.push(position);
            *next_sample_at += SAMPLE_INTERVAL;
            while *next_sample_at <= now {
                *next_sample_at += SAMPLE_INTERVAL;
            }
        }
        false
    }

    #[must_use]
    pub fn view(&self, now: Instant) -> DriftView<'_> {
        match &self.state {
            DriftState::Ready => DriftView::Ready,
            DriftState::Countdown { started_at } => DriftView::Countdown {
                remaining: COUNTDOWN_DURATION
                    .saturating_sub(now.saturating_duration_since(*started_at)),
            },
            DriftState::Sampling {
                started_at,
                samples,
                ..
            } => DriftView::Sampling {
                elapsed: now
                    .saturating_duration_since(*started_at)
                    .min(SAMPLE_DURATION),
                sample_count: samples.len(),
            },
            DriftState::Complete(metrics) => DriftView::Complete(metrics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples_current_state_after_countdown() {
        let start = Instant::now();
        let mut test = DriftTest::default();
        test.start(7, start);

        test.tick(start + COUNTDOWN_DURATION, Some((0.01, 0.02)));
        assert!(matches!(
            test.view(start + COUNTDOWN_DURATION),
            DriftView::Sampling {
                sample_count: 1,
                ..
            }
        ));
        assert!(test.tick(
            start + COUNTDOWN_DURATION + SAMPLE_DURATION,
            Some((0.01, 0.02))
        ));
        assert!(matches!(
            test.view(start + COUNTDOWN_DURATION + SAMPLE_DURATION),
            DriftView::Complete(_)
        ));
    }
}
