use super::layout::{ACTIVE_BORDER, centered_rect};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub(super) fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered_rect(64, 22, area);
    frame.render_widget(Clear, popup);
    let help = Paragraph::new(vec![
        Line::from("Keyboard controls"),
        Line::from(""),
        Line::from("  q / Ctrl-C       Quit"),
        Line::from("  d                 Open controller selector"),
        Line::from("  Tab / Shift-Tab   Change diagnostic tab"),
        Line::from("  1–5               Select a diagnostic tab"),
        Line::from("  ↑ ↓ / j k        Select a connected controller"),
        Line::from("  r                 Run a 300 ms rumble test"),
        Line::from("  x                 Reset selected-device observations"),
        Line::from("  e                 Export JSON and text reports"),
        Line::from("  p                 Pause event auto-scrolling"),
        Line::from("  ↑ / ↓             Scroll paused events"),
        Line::from("  f / v             Filter event kind / device"),
        Line::from("  /                 Filter events by control text"),
        Line::from("  c                 Clear event history on dashboard"),
        Line::from("  Esc               Cancel rumble / close help"),
        Line::from("  ?                 Close this help"),
        Line::from(""),
        Line::from("Disconnected selections are retained until you choose another device."),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(ACTIVE_BORDER)),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(help, popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn help_popup_keeps_the_final_explanation_visible() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| render_help(frame, frame.area()))
            .expect("help should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("Disconnected selections are retained"));
    }
}
