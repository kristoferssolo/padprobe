use super::*;
use claims::assert_some;
use std::time::{Duration, Instant};

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

    test.tick(start + session::COUNTDOWN_DURATION, Some((0.01, 0.02)));
    assert!(matches!(
        test.view(start + session::COUNTDOWN_DURATION),
        DriftView::Sampling {
            sample_count: 1,
            ..
        }
    ));
    assert!(test.tick(
        start + session::COUNTDOWN_DURATION + session::SAMPLE_DURATION,
        Some((0.01, 0.02))
    ));
    assert!(matches!(
        test.view(start + session::COUNTDOWN_DURATION + session::SAMPLE_DURATION),
        DriftView::Complete(_)
    ));
}
