use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{Block, Borders},
};

pub(super) const ACTIVE_BORDER: Color = Color::Cyan;
pub(super) const WARNING: Color = Color::Yellow;

pub(super) fn panel_block(title: &str) -> Block<'_> {
    Block::default().borders(Borders::ALL).title(title)
}

pub(super) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
