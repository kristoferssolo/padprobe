use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Focus};

use super::layout::{WARNING, focused_block};

pub(super) fn render_devices(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let items = app.device_order.iter().filter_map(|id| {
        let device = app.devices.get(id)?;
        let selected = app.selected_id == Some(*id);
        let marker = if selected { ">" } else { " " };
        let state = if device.connected {
            "connected"
        } else {
            "disconnected"
        };
        let style = if !device.connected {
            Style::default().fg(WARNING)
        } else if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Some(ListItem::new(format!("{marker} {} [{state}]", device.metadata.name)).style(style))
    });
    let block = focused_block(" Devices ", app.focus == Focus::Devices);
    if app.devices.is_empty() {
        frame.render_widget(
            Paragraph::new(
                "No controllers detected.\n\nConnect a controller, then wait for PadProbe to detect it.",
            )
            .block(block)
            .wrap(Wrap { trim: true }),
            area,
        );
    } else {
        frame.render_widget(List::new(items).block(block), area);
    }
}
