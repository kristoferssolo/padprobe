use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

pub(super) const ACTIVE_BORDER: Color = Color::Cyan;
pub(super) const WARNING: Color = Color::Yellow;

pub(super) fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
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

pub(super) fn dashboard_sections(area: Rect) -> [Rect; 4] {
    const PRIMARY_HEIGHT: u16 = 15;

    let dashboard = dashboard_content_area(dashboard_area(area));
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(dashboard);
    let primary_height = PRIMARY_HEIGHT.min(dashboard.height);
    let primary = Rect::new(columns[1].x, columns[1].y, columns[1].width, primary_height);
    let lower = Rect::new(
        columns[1].x,
        columns[1].y + primary_height,
        columns[1].width,
        columns[1].height.saturating_sub(primary_height),
    );
    let lower_cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(lower);
    [
        Rect::new(
            columns[0].x,
            columns[0].y,
            columns[0].width,
            columns[0].height,
        ),
        primary,
        lower_cards[0],
        lower_cards[1],
    ]
}

fn dashboard_area(area: Rect) -> Rect {
    const MAX_WIDTH: u16 = 180;
    const MAX_HEIGHT: u16 = 29;

    let width = area.width.min(MAX_WIDTH);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y,
        width,
        area.height.min(MAX_HEIGHT),
    )
}

const fn dashboard_content_area(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_dashboard_is_constrained_to_content_dimensions() {
        assert_eq!(
            dashboard_area(Rect::new(0, 0, 240, 70)),
            Rect::new(30, 0, 180, 29)
        );
        assert_eq!(
            dashboard_area(Rect::new(0, 0, 120, 30)),
            Rect::new(0, 0, 120, 29)
        );
    }

    #[test]
    fn large_dashboard_keeps_events_short_and_wider() {
        let [controller, primary, raw, events] = dashboard_sections(Rect::new(0, 0, 240, 70));

        assert_eq!(controller.height, 28);
        assert_eq!(primary.height, 15);
        assert_eq!(raw.height, 13);
        assert_eq!(events.height, 13);
        assert!(raw.width < events.width);
        assert!(events.width >= 60);
    }
}
