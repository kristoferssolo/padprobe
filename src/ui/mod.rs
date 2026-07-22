mod devices;
mod diagnostics;
mod events;
mod footer;
mod gamepad;
mod layout;
mod live_state;
mod overlays;

use self::{
    devices::render_device_selector,
    diagnostics::{render_primary_diagnostics, render_raw_data},
    events::render_events,
    footer::render_footer,
    live_state::render_live_state,
    overlays::render_help,
};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::cmp;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    if area.width < 54 || area.height < 16 {
        render_compact(frame, app, area);
    } else {
        render_full(frame, app, area);
    }

    if app.help_visible {
        render_help(frame, area);
    } else if app.device_selector_visible {
        render_device_selector(frame, app, area);
    }
}

fn render_full(frame: &mut Frame<'_>, app: &App, area: Rect) {
    if area.width >= 96 && area.height >= 26 {
        render_dashboard(frame, app, area);
        return;
    }

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(cmp::min(6, area.height / 4)),
            Constraint::Length(1),
        ])
        .split(area);
    render_live_state(frame, app, vertical[0]);
    render_events(frame, app, vertical[1]);
    render_footer(frame, app, vertical[2]);
}

fn render_dashboard(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(25), Constraint::Length(1)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(vertical[0]);
    let diagnostics = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[1]);
    let lower_cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(diagnostics[1]);

    render_live_state(frame, app, columns[0]);
    render_primary_diagnostics(frame, app, diagnostics[0]);
    render_raw_data(frame, app, lower_cards[0]);
    render_events(frame, app, lower_cards[1]);
    render_footer(frame, app, vertical[1]);
}

fn render_compact(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let message = app.selected_device().map_or_else(
        || {
            "No controllers detected.\n\nConnect a controller, then wait for PadProbe to detect it."
                .to_owned()
        },
        |(_, device)| {
            format!(
                "PadProbe — {}\n\nTerminal is too small for the diagnostic view.\nResize to at least 54×16.",
                device.metadata.name
            )
        },
    );
    frame.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(" PadProbe "))
            .wrap(Wrap { trim: true }),
        chunks[0],
    );
    render_footer(frame, app, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::DeviceMetadata;
    use ratatui::{Terminal, backend::TestBackend};

    fn app() -> App {
        let mut app = App::new();
        app.connect(
            0,
            DeviceMetadata {
                name: "Fixture Controller".to_owned(),
                vendor_id: Some(0x1234),
                product_id: Some(0x5678),
                uuid: "fixture".to_owned(),
                mapping: "standard".to_owned(),
                power: "unknown".to_owned(),
                rumble_supported: false,
            },
        );
        app
    }

    fn render_text(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| render(frame, &app()))
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
    }

    #[test]
    fn medium_terminal_uses_stacked_view() {
        let rendered = render_text(80, 24);

        assert!(rendered.contains("Controller · Fixture Controller"));
        assert!(rendered.contains("Recent events"));
        assert!(!rendered.contains("Analog sticks"));
    }

    #[test]
    fn small_terminal_uses_compact_view() {
        let rendered = render_text(50, 15);

        assert!(rendered.contains("Terminal is too small"));
    }
}
