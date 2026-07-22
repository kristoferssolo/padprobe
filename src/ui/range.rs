use crate::{
    analysis::{RangeMetrics, RangeView},
    app::App,
};
use gamepad_widget::StickGauge;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(super) fn render_range(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(format!(
        " Observed stick range · {} stick ",
        app.range_test.side().label()
    ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    match app.range_test.view() {
        RangeView::Ready => render_ready(frame, inner),
        RangeView::Recording {
            sample_count,
            trace,
        } => render_recording(frame, sample_count, trace, inner),
        RangeView::Complete { metrics, trace } => render_results(frame, metrics, trace, inner),
    }
}

fn render_ready(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "Move the selected stick slowly around its full outer edge.",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("l / r  choose left or right stick"),
            Line::from("s      start recording"),
            Line::from("s      finish after tracing the edge"),
            Line::from("Esc    cancel"),
            Line::from(""),
            Line::styled(
                "This describes reported input range behavior, not the exact physical gate shape.",
                Style::default().fg(Color::DarkGray),
            ),
        ])
        .alignment(Alignment::Center),
        area,
    );
}

fn render_recording(frame: &mut Frame<'_>, sample_count: usize, trace: &[(f64, f64)], area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);
    frame.render_widget(
        StickGauge::new("TRACE", 0.0, 0.0)
            .trace(trace)
            .trace_style(Style::default().fg(Color::Green)),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "Recording {sample_count} events · trace the full edge · press s to finish"
        ))
        .alignment(Alignment::Center),
        chunks[1],
    );
}

fn render_results(frame: &mut Frame<'_>, metrics: &RangeMetrics, trace: &[(f64, f64)], area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);
    frame.render_widget(
        StickGauge::new("OBSERVED FIGURE", 0.0, 0.0)
            .trace(trace)
            .trace_style(Style::default().fg(Color::Green)),
        columns[0],
    );
    let lines = vec![
        Line::styled(
            "Reported input range behavior",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(format!("Samples             {}", metrics.sample_count)),
        Line::from(format!(
            "X range             {:+.3} … {:+.3}",
            metrics.minimum_x, metrics.maximum_x
        )),
        Line::from(format!(
            "Y range             {:+.3} … {:+.3}",
            metrics.minimum_y, metrics.maximum_y
        )),
        Line::from(format!(
            "Edge radius         {:.3} … {:.3}",
            metrics.minimum_edge_radius, metrics.maximum_edge_radius
        )),
        Line::from(format!(
            "Mean edge radius    {:.3}",
            metrics.mean_edge_radius
        )),
        Line::from(format!(
            "Circularity dev.    {:.2}%",
            metrics.circularity_deviation * 100.0
        )),
        Line::from(format!(
            "Angular coverage    {:.1}% ({} sectors missing)",
            metrics.angular_coverage_percent, metrics.missing_sector_count
        )),
        Line::from(format!(
            "Under / over range  {:.1}% / {:.1}%",
            metrics.under_range_percent, metrics.over_range_percent
        )),
        Line::from(""),
        Line::styled(
            "Results can be affected by mapping, normalization, and configured deadzones.",
            Style::default().fg(Color::DarkGray),
        ),
        Line::from("Press s to record again."),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), columns[1]);
}
