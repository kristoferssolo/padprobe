use crate::app::{App, AppTab};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const TABS: [AppTab; 5] = [
    AppTab::Dashboard,
    AppTab::Drift,
    AppTab::Range,
    AppTab::Controls,
    AppTab::Timing,
];

pub(super) fn render_tabs(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let spans = TABS.iter().enumerate().flat_map(|(index, tab)| {
        (index > 0)
            .then(|| Span::raw("  "))
            .into_iter()
            .chain(std::iter::once(Span::styled(
                format!("{} {}", index + 1, tab.title()),
                if *tab == app.active_tab {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            )))
    });
    frame.render_widget(
        Paragraph::new(Line::from(spans.collect::<Vec<_>>()).alignment(Alignment::Center)),
        area,
    );
}
