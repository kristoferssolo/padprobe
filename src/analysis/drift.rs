use serde::Serialize;
use std::time::{Duration, Instant};

const COUNTDOWN_DURATION: Duration = Duration::from_secs(3);
const SAMPLE_DURATION: Duration = Duration::from_secs(10);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(10);
const MAX_SAMPLES: usize = 2_000;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub enum StickSide {
    #[default]
    Left,
    Right,
}

impl StickSide {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DriftMetrics {
    pub duration_seconds: f64,
    pub sample_count: usize,
    pub mean_x: f64,
    pub mean_y: f64,
    pub median_x: f64,
    pub median_y: f64,
    pub mean_radial: f64,
    pub percentile_95_radial: f64,
    pub maximum_radial: f64,
    pub standard_deviation_x: f64,
    pub standard_deviation_y: f64,
    pub directional_bias_degrees: Option<f64>,
    pub minimum_x: f64,
    pub maximum_x: f64,
    pub minimum_y: f64,
    pub maximum_y: f64,
    pub suggested_inner_deadzone: f64,
}

impl DriftMetrics {
    #[must_use]
    pub fn calculate(samples: &[(f32, f32)], duration: Duration) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }

        let count = f64::from(u32::try_from(samples.len()).unwrap_or(u32::MAX));
        let mean_x = samples.iter().map(|(x, _)| f64::from(*x)).sum::<f64>() / count;
        let mean_y = samples.iter().map(|(_, y)| f64::from(*y)).sum::<f64>() / count;
        let mut xs = samples
            .iter()
            .map(|(x, _)| f64::from(*x))
            .collect::<Vec<_>>();
        let mut ys = samples
            .iter()
            .map(|(_, y)| f64::from(*y))
            .collect::<Vec<_>>();
        let mut radii = samples
            .iter()
            .map(|(x, y)| f64::from(*x).hypot(f64::from(*y)))
            .collect::<Vec<_>>();
        xs.sort_by(f64::total_cmp);
        ys.sort_by(f64::total_cmp);
        radii.sort_by(f64::total_cmp);
        let mean_radial = radii.iter().sum::<f64>() / count;
        let percentile_95_radial = percentile_95(&radii);
        let bias_magnitude = mean_x.hypot(mean_y);

        Some(Self {
            duration_seconds: duration.as_secs_f64(),
            sample_count: samples.len(),
            mean_x,
            mean_y,
            median_x: median(&xs),
            median_y: median(&ys),
            mean_radial,
            percentile_95_radial,
            maximum_radial: radii.last().copied().unwrap_or_default(),
            standard_deviation_x: standard_deviation(&xs, mean_x),
            standard_deviation_y: standard_deviation(&ys, mean_y),
            directional_bias_degrees: (bias_magnitude > f64::EPSILON)
                .then(|| mean_y.atan2(mean_x).to_degrees()),
            minimum_x: xs.first().copied().unwrap_or_default(),
            maximum_x: xs.last().copied().unwrap_or_default(),
            minimum_y: ys.first().copied().unwrap_or_default(),
            maximum_y: ys.last().copied().unwrap_or_default(),
            suggested_inner_deadzone: (((percentile_95_radial + 0.01) * 100.0).ceil() / 100.0)
                .clamp(0.0, 1.0),
        })
    }
}

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

fn median(sorted: &[f64]) -> f64 {
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        sorted[middle - 1].midpoint(sorted[middle])
    } else {
        sorted[middle]
    }
}

fn percentile_95(sorted: &[f64]) -> f64 {
    let index = (sorted.len().saturating_sub(1) * 95 + 50) / 100;
    sorted[index]
}

fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / f64::from(u32::try_from(values.len()).unwrap_or(u32::MAX));
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some;

    #[test]
    fn drift_metrics_describe_resting_samples() {
        let metrics = assert_some!(DriftMetrics::calculate(
            &[(0.01, -0.02), (0.03, 0.0), (0.02, -0.01)],
            Duration::from_secs(10),
        ));

        assert_eq!(metrics.sample_count, 3);
        assert!((metrics.mean_x - 0.02).abs() < 1e-6);
        assert!((metrics.median_y - -0.01).abs() < 1e-6);
        assert!(metrics.maximum_radial > metrics.mean_radial);
        assert!(metrics.suggested_inner_deadzone >= metrics.percentile_95_radial);
    }

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
