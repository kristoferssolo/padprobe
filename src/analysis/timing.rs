use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct TimingSample {
    pub elapsed_seconds: f64,
    pub signature: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
pub struct TimingHistogram {
    pub under_2_ms: usize,
    pub from_2_to_5_ms: usize,
    pub from_5_to_10_ms: usize,
    pub from_10_to_20_ms: usize,
    pub from_20_to_50_ms: usize,
    pub at_least_50_ms: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimingMetrics {
    pub event_count: usize,
    pub interval_count: usize,
    pub observed_duration_seconds: f64,
    pub events_per_second: f64,
    pub minimum_interval_ms: f64,
    pub median_interval_ms: f64,
    pub maximum_interval_ms: f64,
    pub percentile_95_interval_ms: f64,
    pub percentile_99_interval_ms: f64,
    pub long_gap_count: usize,
    pub duplicate_value_count: usize,
    pub duplicate_value_percent: f64,
    pub histogram: TimingHistogram,
}

impl TimingMetrics {
    #[must_use]
    pub fn calculate(samples: &[TimingSample]) -> Option<Self> {
        if samples.len() < 2 {
            return None;
        }
        let mut intervals = samples
            .windows(2)
            .map(|pair| (pair[1].elapsed_seconds - pair[0].elapsed_seconds).max(0.0) * 1_000.0)
            .collect::<Vec<_>>();
        let duplicate_value_count = samples
            .windows(2)
            .filter(|pair| pair[0].signature == pair[1].signature)
            .count();
        let observed_duration_seconds =
            (samples.last()?.elapsed_seconds - samples.first()?.elapsed_seconds).max(0.0);
        intervals.sort_by(f64::total_cmp);
        let interval_count = intervals.len();
        let interval_count_f64 = f64::from(u32::try_from(interval_count).unwrap_or(u32::MAX));
        let p95 = percentile(&intervals, 95);
        let long_gap_threshold = (p95 * 2.0).max(50.0);
        let histogram = histogram(&intervals);

        Some(Self {
            event_count: samples.len(),
            interval_count,
            observed_duration_seconds,
            events_per_second: if observed_duration_seconds > f64::EPSILON {
                interval_count_f64 / observed_duration_seconds
            } else {
                0.0
            },
            minimum_interval_ms: intervals.first().copied().unwrap_or_default(),
            median_interval_ms: percentile(&intervals, 50),
            maximum_interval_ms: intervals.last().copied().unwrap_or_default(),
            percentile_95_interval_ms: p95,
            percentile_99_interval_ms: percentile(&intervals, 99),
            long_gap_count: intervals
                .iter()
                .filter(|interval| **interval > long_gap_threshold)
                .count(),
            duplicate_value_count,
            duplicate_value_percent: f64::from(
                u32::try_from(duplicate_value_count).unwrap_or(u32::MAX),
            ) / interval_count_f64
                * 100.0,
            histogram,
        })
    }
}

fn percentile(sorted: &[f64], percentile: usize) -> f64 {
    let index = (sorted.len().saturating_sub(1) * percentile + 50) / 100;
    sorted[index]
}

fn histogram(intervals: &[f64]) -> TimingHistogram {
    intervals
        .iter()
        .fold(TimingHistogram::default(), |mut histogram, interval| {
            match *interval {
                value if value < 2.0 => histogram.under_2_ms += 1,
                value if value < 5.0 => histogram.from_2_to_5_ms += 1,
                value if value < 10.0 => histogram.from_5_to_10_ms += 1,
                value if value < 20.0 => histogram.from_10_to_20_ms += 1,
                value if value < 50.0 => histogram.from_20_to_50_ms += 1,
                _ => histogram.at_least_50_ms += 1,
            }
            histogram
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some;

    #[test]
    fn timing_metrics_use_observed_intervals() {
        let samples = [
            TimingSample {
                elapsed_seconds: 0.000,
                signature: "x=0".to_owned(),
            },
            TimingSample {
                elapsed_seconds: 0.010,
                signature: "x=1".to_owned(),
            },
            TimingSample {
                elapsed_seconds: 0.020,
                signature: "x=1".to_owned(),
            },
            TimingSample {
                elapsed_seconds: 0.120,
                signature: "x=2".to_owned(),
            },
        ];

        let metrics = assert_some!(TimingMetrics::calculate(&samples));

        assert_eq!(metrics.event_count, 4);
        assert!((metrics.median_interval_ms - 10.0).abs() < 1e-9);
        assert!((metrics.maximum_interval_ms - 100.0).abs() < 1e-9);
        assert_eq!(metrics.duplicate_value_count, 1);
        assert_eq!(metrics.histogram.from_10_to_20_ms, 2);
        assert_eq!(metrics.histogram.at_least_50_ms, 1);
    }
}
