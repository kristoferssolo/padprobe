use super::layout::{ACTIVE_BORDER, WARNING, centered_rect};
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub(super) fn render_device_selector(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let height = (app.device_order.len() as u16 + 2).clamp(5, 14);
    let popup = centered_rect(62, height, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select controller — ↑↓/jk move, Enter/Esc close ")
        .border_style(Style::default().fg(ACTIVE_BORDER));
    if app.devices.is_empty() {
        frame.render_widget(
            Paragraph::new("No controllers detected.").block(block),
            popup,
        );
    } else {
        frame.render_widget(List::new(device_items(app)).block(block), popup);
    }
}

fn device_items(app: &App) -> Vec<ListItem<'static>> {
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
    items.collect()
}
