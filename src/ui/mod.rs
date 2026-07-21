mod devices;
mod events;
mod footer;
mod gamepad;
mod layout;
mod live_state;
mod overlays;

use self::{
    devices::render_device_selector, events::render_events, footer::render_footer,
    live_state::render_live_state, overlays::render_help,
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
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(cmp::min(10, area.height / 3)),
            Constraint::Length(1),
        ])
        .split(area);
    render_live_state(frame, app, vertical[0]);
    render_events(frame, app, vertical[1]);
    render_footer(frame, app, vertical[2]);
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
