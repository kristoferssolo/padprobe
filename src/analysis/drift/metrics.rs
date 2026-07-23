use serde::Serialize;
use std::time::Duration;

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
}
