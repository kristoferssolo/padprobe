use super::{
    diagnostics::{render_primary_diagnostics, render_raw_data},
    events::render_events,
    layout::dashboard_sections,
    live_state::render_live_state,
};
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

pub(super) fn render_dashboard(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let [controller, primary, raw, events] = dashboard_sections(area);
    render_live_state(frame, app, controller);
    render_primary_diagnostics(frame, app, primary);
    render_raw_data(frame, app, raw);
    render_events(frame, app, events);
    frame.render_widget(
        Block::default().borders(Borders::ALL).title(" Dashboard "),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_app;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_text(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_dashboard(frame, &test_app(), area);
            })
            .expect("test frame should render");
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn wide_terminal_renders_dashboard_cards() {
        let rendered = render_text(120, 30);

        assert!(rendered.contains("Controller · Fixture Controller"));
        assert!(rendered.contains("Analog sticks"));
        assert!(rendered.contains("Triggers · 0–1"));
        assert!(rendered.contains("Raw mapped data"));
        assert!(rendered.contains("Recent events"));
        assert!(rendered.contains("L3 ○"));
        assert!(!rendered.contains("VID:PID"));
        assert!(
            rendered
                .chars()
                .filter(|symbol| ('\u{2800}'..='\u{28ff}').contains(symbol))
                .count()
                > 40
        );
    }

    #[test]
    fn dashboard_has_a_complete_outer_contour() {
        let backend = TestBackend::new(240, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| render_dashboard(frame, &test_app(), Rect::new(0, 1, 240, 38)))
            .expect("dashboard should render");

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 1)].symbol(), "┌");
        assert_eq!(buffer[(239, 1)].symbol(), "┐");
        assert_eq!(buffer[(0, 38)].symbol(), "└");
        assert_eq!(buffer[(239, 38)].symbol(), "┘");
        assert_eq!(buffer[(0, 1)].fg, ratatui::style::Color::Reset);
    }
}
