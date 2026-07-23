use super::{super::StickSide, RangeMetrics};

const MAX_RANGE_SAMPLES: usize = 4_096;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
