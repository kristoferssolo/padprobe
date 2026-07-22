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
    let [controller, primary, raw, events] = dashboard_sections(area);
    render_live_state(frame, app, controller);
    render_primary_diagnostics(frame, app, primary);
    render_raw_data(frame, app, raw);
    render_events(frame, app, events);
    render_footer(
        frame,
        app,
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
    );
}

fn dashboard_sections(area: Rect) -> [Rect; 4] {
    const CARD_HEIGHT: u16 = 29;
    const PRIMARY_HEIGHT: u16 = 15;
    const RAW_HEIGHT: u16 = 14;

    let dashboard = dashboard_area(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(dashboard);
    let primary_height = if dashboard.height >= CARD_HEIGHT {
        PRIMARY_HEIGHT
    } else {
        dashboard.height / 2
    };
    let primary = Rect::new(columns[1].x, columns[1].y, columns[1].width, primary_height);
    let lower = Rect::new(
        columns[1].x,
        columns[1].y + primary_height,
        columns[1].width,
        columns[1].height.saturating_sub(primary_height),
    );
    let lower_cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(lower);
    [
        Rect::new(
            columns[0].x,
            columns[0].y,
            columns[0].width,
            columns[0].height.min(CARD_HEIGHT),
        ),
        primary,
        Rect::new(
            lower_cards[0].x,
            lower_cards[0].y,
            lower_cards[0].width,
            lower_cards[0].height.min(RAW_HEIGHT),
        ),
        lower_cards[1],
    ]
}

fn dashboard_area(area: Rect) -> Rect {
    const MAX_WIDTH: u16 = 180;

    let width = area.width.min(MAX_WIDTH);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y,
        width,
        area.height.saturating_sub(1),
    )
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
        assert!(rendered.contains("LEFT STICK"));
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

    #[test]
    fn large_dashboard_is_constrained_to_content_dimensions() {
        assert_eq!(
            dashboard_area(Rect::new(0, 0, 240, 70)),
            Rect::new(30, 0, 180, 69)
        );
        assert_eq!(
            dashboard_area(Rect::new(0, 0, 120, 30)),
            Rect::new(0, 0, 120, 29)
        );
    }

    #[test]
    fn large_dashboard_gives_remaining_height_to_events() {
        let [controller, primary, raw, events] = dashboard_sections(Rect::new(0, 0, 240, 70));

        assert_eq!(controller.height, 29);
        assert_eq!(primary.height, 15);
        assert_eq!(raw.height, 14);
        assert_eq!(events.height, 54);
    }
}
