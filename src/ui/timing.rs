use crate::{
    analysis::{TimingHistogram, TimingMetrics},
    app::App,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Wrap},
};

pub(super) fn render_timing(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Observed event timing ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(metrics) = app.selected_event_timing() else {
        frame.render_widget(
            Paragraph::new(
                "Not enough input events yet.\n\nMove a stick or press buttons on the selected controller.",
            )
            .alignment(Alignment::Center),
            inner,
        );
        return;
    };
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(7),
            Constraint::Length(4),
        ])
        .split(inner);
    render_summary(frame, &metrics, sections[0]);
    render_histogram(frame, &metrics.histogram, sections[1]);
    frame.render_widget(
        Paragraph::new(
            "Observed event timing is not the controller's exact polling rate. Driver buffering, OS scheduling, Bluetooth, API processing, and event coalescing can affect it.\n\nc clears the event sample history.",
        )
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true }),
        sections[2],
    );
}

fn render_summary(frame: &mut Frame<'_>, metrics: &TimingMetrics, area: Rect) {
    let lines = vec![
        Line::styled(
            format!(
                "{} events over {:.2} s · observed rate {:.1} events/s",
                metrics.event_count, metrics.observed_duration_seconds, metrics.events_per_second
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(format!(
            "Intervals ms  min {:.3} · median {:.3} · max {:.3}",
            metrics.minimum_interval_ms, metrics.median_interval_ms, metrics.maximum_interval_ms
        )),
        Line::from(format!(
            "Percentiles   p95 {:.3} ms · p99 {:.3} ms",
            metrics.percentile_95_interval_ms, metrics.percentile_99_interval_ms
        )),
        Line::from(format!(
            "Long gaps     {} (> max(2×p95, 50 ms))",
            metrics.long_gap_count
        )),
        Line::from(format!(
            "Duplicates    {} ({:.1}% of consecutive observations)",
            metrics.duplicate_value_count, metrics.duplicate_value_percent
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_histogram(frame: &mut Frame<'_>, histogram: &TimingHistogram, area: Rect) {
    let bars = [
        ("<2", histogram.under_2_ms),
        ("2–5", histogram.from_2_to_5_ms),
        ("5–10", histogram.from_5_to_10_ms),
        ("10–20", histogram.from_10_to_20_ms),
        ("20–50", histogram.from_20_to_50_ms),
        ("≥50", histogram.at_least_50_ms),
    ]
    .map(|(label, value)| {
        Bar::default()
            .label(Line::from(label))
            .value(u64::try_from(value).unwrap_or(u64::MAX))
    });
    frame.render_widget(
        BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(Style::default().fg(Color::White))
            .label_style(Style::default().fg(Color::DarkGray))
            .bar_width(7)
            .bar_gap(2)
            .group_gap(0)
            .direction(Direction::Vertical),
        area,
    );
}
