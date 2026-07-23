use super::StickSide;
use serde::Serialize;

const SECTOR_COUNT: usize = 36;
const SECTOR_COUNT_F64: f64 = 36.0;
const MAX_RANGE_SAMPLES: usize = 4_096;
const EDGE_THRESHOLD: f64 = 0.5;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RangeMetrics {
    pub sample_count: usize,
    pub minimum_x: f64,
    pub maximum_x: f64,
    pub minimum_y: f64,
    pub maximum_y: f64,
    pub minimum_edge_radius: f64,
    pub maximum_edge_radius: f64,
    pub mean_edge_radius: f64,
    pub circularity_deviation: f64,
    pub angular_coverage_percent: f64,
    pub missing_sector_count: usize,
    pub under_range_percent: f64,
    pub over_range_percent: f64,
}

impl RangeMetrics {
    #[must_use]
    pub fn calculate(samples: &[(f32, f32)]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let minimum_x = samples
            .iter()
            .map(|(x, _)| f64::from(*x))
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let maximum_x = samples
            .iter()
            .map(|(x, _)| f64::from(*x))
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let minimum_y = samples
            .iter()
            .map(|(_, y)| f64::from(*y))
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let maximum_y = samples
            .iter()
            .map(|(_, y)| f64::from(*y))
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let edge_radii = samples
            .iter()
            .map(|(x, y)| f64::from(*x).hypot(f64::from(*y)))
            .filter(|radius| *radius >= EDGE_THRESHOLD)
            .collect::<Vec<_>>();
        if edge_radii.is_empty() {
            return None;
        }

        let edge_count = f64::from(u32::try_from(edge_radii.len()).unwrap_or(u32::MAX));
        let minimum_edge_radius = edge_radii
            .iter()
            .copied()
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let maximum_edge_radius = edge_radii
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let mean_edge_radius = edge_radii.iter().sum::<f64>() / edge_count;
        let circularity_deviation = edge_radii
            .iter()
            .map(|radius| (1.0 - radius).abs())
            .sum::<f64>()
            / edge_count;
        let mut sectors = [false; SECTOR_COUNT];
        for (x, y) in samples {
            let x = f64::from(*x);
            let y = f64::from(*y);
            if x.hypot(y) >= EDGE_THRESHOLD {
                sectors[sector_index(x, y)] = true;
            }
        }
        let covered = sectors.iter().filter(|covered| **covered).count();
        let coverage =
            f64::from(u32::try_from(covered).unwrap_or_default()) / SECTOR_COUNT_F64 * 100.0;

        Some(Self {
            sample_count: samples.len(),
            minimum_x,
            maximum_x,
            minimum_y,
            maximum_y,
            minimum_edge_radius,
            maximum_edge_radius,
            mean_edge_radius,
            circularity_deviation,
            angular_coverage_percent: coverage,
            missing_sector_count: SECTOR_COUNT - covered,
            under_range_percent: (1.0 - maximum_edge_radius).max(0.0) * 100.0,
            over_range_percent: (maximum_edge_radius - 1.0).max(0.0) * 100.0,
        })
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the normalized angle is clamped to the fixed sector count"
)]
fn sector_index(x: f64, y: f64) -> usize {
    let angle = y.atan2(x).rem_euclid(std::f64::consts::TAU);
    ((angle / std::f64::consts::TAU * SECTOR_COUNT_F64).floor() as usize).min(SECTOR_COUNT - 1)
}

#[derive(Clone, Copy, Debug)]
pub enum RangeView<'test> {
    Ready,
    Recording {
        sample_count: usize,
        trace: &'test [(f64, f64)],
    },
    Complete {
        metrics: &'test RangeMetrics,
        trace: &'test [(f64, f64)],
    },
}

#[derive(Clone, Debug)]
enum RangeState {
    Ready,
    Recording {
        samples: Vec<(f32, f32)>,
        trace: Vec<(f64, f64)>,
    },
    Complete {
        metrics: RangeMetrics,
        trace: Vec<(f64, f64)>,
    },
}

#[derive(Clone, Debug)]
pub struct RangeTest {
    side: StickSide,
    device_id: Option<usize>,
    state: RangeState,
}

impl Default for RangeTest {
    fn default() -> Self {
        Self {
            side: StickSide::default(),
            device_id: None,
            state: RangeState::Ready,
        }
    }
}

impl RangeTest {
    #[must_use]
    pub const fn side(&self) -> StickSide {
        self.side
    }

    #[must_use]
    pub const fn device_id(&self) -> Option<usize> {
        self.device_id
    }

    #[must_use]
    pub const fn result(&self) -> Option<&RangeMetrics> {
        if let RangeState::Complete { metrics, .. } = &self.state {
            Some(metrics)
        } else {
            None
        }
    }

    pub fn select_side(&mut self, side: StickSide) {
        if !matches!(self.state, RangeState::Recording { .. }) {
            self.side = side;
            self.state = RangeState::Ready;
        }
    }

    pub fn start(&mut self, device_id: usize) {
        self.device_id = Some(device_id);
        self.state = RangeState::Recording {
            samples: Vec::with_capacity(MAX_RANGE_SAMPLES),
            trace: Vec::with_capacity(MAX_RANGE_SAMPLES),
        };
    }

    pub fn record(&mut self, device_id: usize, position: (f32, f32)) {
        if self.device_id != Some(device_id) {
            return;
        }
        let RangeState::Recording { samples, trace } = &mut self.state else {
            return;
        };
        if samples.len() == MAX_RANGE_SAMPLES {
            samples.remove(0);
            trace.remove(0);
        }
        samples.push(position);
        trace.push((f64::from(position.0), f64::from(position.1)));
    }

    pub fn finish(&mut self) -> bool {
        let RangeState::Recording { samples, trace } =
            std::mem::replace(&mut self.state, RangeState::Ready)
        else {
            return false;
        };
        let Some(metrics) = RangeMetrics::calculate(&samples) else {
            self.device_id = None;
            return false;
        };
        self.state = RangeState::Complete { metrics, trace };
        true
    }

    pub fn cancel(&mut self) {
        self.device_id = None;
        self.state = RangeState::Ready;
    }

    #[must_use]
    pub const fn is_recording(&self) -> bool {
        matches!(self.state, RangeState::Recording { .. })
    }

    #[must_use]
    pub fn view(&self) -> RangeView<'_> {
        match &self.state {
            RangeState::Ready => RangeView::Ready,
            RangeState::Recording { samples, trace } => RangeView::Recording {
                sample_count: samples.len(),
                trace,
            },
            RangeState::Complete { metrics, trace } => RangeView::Complete { metrics, trace },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some;

    #[test]
    fn complete_circle_covers_every_sector() {
        let samples = (0_u16..360)
            .map(|degree| {
                let angle = f32::from(degree).to_radians();
                (angle.cos(), angle.sin())
            })
            .collect::<Vec<_>>();

        let metrics = assert_some!(RangeMetrics::calculate(&samples));

        assert_eq!(metrics.missing_sector_count, 0);
        assert!((metrics.angular_coverage_percent - 100.0).abs() < f64::EPSILON);
        assert!(metrics.circularity_deviation < 1e-6);
    }

    #[test]
    fn test_retains_result_after_recording() {
        let mut test = RangeTest::default();
        test.start(4);
        for degree in 0_u16..360 {
            let angle = f32::from(degree).to_radians();
            test.record(4, (angle.cos(), angle.sin()));
        }

        assert!(test.finish());
        assert!(matches!(test.view(), RangeView::Complete { .. }));
    }
}
