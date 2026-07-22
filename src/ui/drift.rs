use crate::{
    analysis::{DriftMetrics, DriftView},
    app::App,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

pub(super) fn render_drift(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(format!(
        " Observed resting input · {} stick ",
        app.drift_test.side().label()
    ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let now = Instant::now();
    match app.drift_test.view(now) {
        DriftView::Ready => render_ready(frame, inner),
        DriftView::Countdown { remaining } => render_countdown(frame, remaining, inner),
        DriftView::Sampling {
            elapsed,
            sample_count,
        } => render_sampling(frame, elapsed, sample_count, inner),
        DriftView::Complete(metrics) => render_results(frame, metrics, inner),
    }
}

fn render_ready(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "Release the selected stick and keep the controller still.",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("l / r  choose left or right stick"),
            Line::from("s      begin with a three-second countdown"),
            Line::from("Esc    cancel an active test"),
            Line::from(""),
            Line::styled(
                "This measures input reported while untouched. Driver mappings, deadzones, Steam Input, and transport can influence the result.",
                Style::default().fg(Color::DarkGray),
            ),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_countdown(frame: &mut Frame<'_>, remaining: Duration, area: Rect) {
    let seconds = remaining.as_secs_f64().ceil();
    frame.render_widget(
        Paragraph::new(format!(
            "Release the stick and keep the controller still.\n\nSampling begins in {seconds:.0}…"
        ))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_sampling(frame: &mut Frame<'_>, elapsed: Duration, sample_count: usize, area: Rect) {
    let progress = (elapsed.as_secs_f64() / 10.0).clamp(0.0, 1.0);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(format!(
            "Keep the controller still.\n{sample_count} fixed-interval samples"
        ))
        .alignment(Alignment::Center),
        chunks[0],
    );
    frame.render_widget(
        Gauge::default()
            .ratio(progress)
            .label(format!("{:.1} / 10.0 s", elapsed.as_secs_f64()))
            .gauge_style(Style::default().fg(Color::Cyan)),
        chunks[1],
    );
}

fn render_results(frame: &mut Frame<'_>, metrics: &DriftMetrics, area: Rect) {
    let bias = metrics
        .directional_bias_degrees
        .map_or_else(|| "none".to_owned(), |degrees| format!("{degrees:+.1}°"));
    let lines = vec![
        Line::styled(
            "Observed resting input displacement",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(format!(
            "Duration {:>5.1} s    Samples {}",
            metrics.duration_seconds, metrics.sample_count
        )),
        Line::from(format!(
            "Mean      x {:+.4}   y {:+.4}",
            metrics.mean_x, metrics.mean_y
        )),
        Line::from(format!(
            "Median    x {:+.4}   y {:+.4}",
            metrics.median_x, metrics.median_y
        )),
        Line::from(format!(
            "Range X   {:+.4} … {:+.4}",
            metrics.minimum_x, metrics.maximum_x
        )),
        Line::from(format!(
            "Range Y   {:+.4} … {:+.4}",
            metrics.minimum_y, metrics.maximum_y
        )),
        Line::from(format!(
            "Radial    mean {:.4}   p95 {:.4}   max {:.4}",
            metrics.mean_radial, metrics.percentile_95_radial, metrics.maximum_radial
        )),
        Line::from(format!(
            "Std dev   x {:.4}   y {:.4}   bias {bias}",
            metrics.standard_deviation_x, metrics.standard_deviation_y
        )),
        Line::styled(
            format!(
                "Suggested inner deadzone: {:.2}",
                metrics.suggested_inner_deadzone
            ),
            Style::default().fg(Color::Green),
        ),
        Line::from(""),
        Line::styled(
            "This result does not identify the physical source or declare the controller faulty.",
            Style::default().fg(Color::DarkGray),
        ),
        Line::from("Press s to run again."),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}
