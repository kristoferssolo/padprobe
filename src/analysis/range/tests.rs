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
