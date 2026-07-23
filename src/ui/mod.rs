mod controls;
mod dashboard;
mod devices;
mod diagnostics;
mod drift;
mod events;
mod footer;
mod gamepad;
mod layout;
mod live_state;
mod overlays;
mod range;
mod tabs;
mod timing;

use self::{
    controls::render_controls, dashboard::render_dashboard, devices::render_device_selector,
    drift::render_drift, events::render_events, footer::render_footer,
    live_state::render_live_state, overlays::render_help, range::render_range, tabs::render_tabs,
    timing::render_timing,
};
use crate::app::App;
#[cfg(test)]
use crate::app::DeviceMetadata;
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
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);
    render_tabs(frame, app, vertical[0]);
    render_footer(frame, app, vertical[2]);

    if app.active_tab != crate::app::AppTab::Dashboard {
        match app.active_tab {
            crate::app::AppTab::Drift => render_drift(frame, app, vertical[1]),
            crate::app::AppTab::Range => render_range(frame, app, vertical[1]),
            crate::app::AppTab::Controls => render_controls(frame, app, vertical[1]),
            crate::app::AppTab::Timing => render_timing(frame, app, vertical[1]),
            crate::app::AppTab::Dashboard => {}
        }
        return;
    }

    if area.width >= 96 && area.height >= 26 {
        render_dashboard(frame, app, vertical[1]);
    } else {
        let block = Block::default().borders(Borders::ALL).title(" Dashboard ");
        let inner = block.inner(vertical[1]);
        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(cmp::min(6, inner.height / 4)),
            ])
            .split(inner);
        render_live_state(frame, app, content[0]);
        render_events(frame, app, content[1]);
        frame.render_widget(block, vertical[1]);
    }
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
fn test_app() -> App {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_text(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| render(frame, &test_app()))
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
