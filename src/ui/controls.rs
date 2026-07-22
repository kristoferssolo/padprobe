use crate::{
    analysis::{ChecklistItem, ChecklistStatus},
    app::App,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(super) fn render_controls(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Guided control checklist ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.control_checklist.items().is_empty() {
        frame.render_widget(
            Paragraph::new(
                "Press s to start.\n\nActivate each button and move each stick axis through at least half of its range.",
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(inner);
    let (observed, pending, skipped) = app.control_checklist.counts();
    frame.render_widget(
        Paragraph::new(format!(
            "Observed {observed} · pending {pending} · skipped {skipped}{}",
            if app.control_checklist.is_active() {
                " · recording"
            } else {
                " · finished"
            }
        ))
        .alignment(Alignment::Center),
        rows[0],
    );
    render_items(frame, app, rows[1]);
    frame.render_widget(
        Paragraph::new(
            "j/k select · Space skip · Enter finish · s restart\nRepeated activations increase the count without creating another control.",
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true }),
        rows[2],
    );
}

fn render_items(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = usize::from((area.width / 32).clamp(1, 3));
    let rows = app.control_checklist.items().len().div_ceil(columns);
    let visible_rows = usize::from(area.height);
    let selected_row = app.control_checklist.selected() % rows.max(1);
    let first_row = selected_row
        .saturating_sub(visible_rows / 2)
        .min(rows.saturating_sub(visible_rows));
    let lines = (first_row..(first_row + visible_rows).min(rows)).map(|row| {
        let spans = (0..columns).flat_map(|column| {
            let index = column * rows + row;
            let item = app.control_checklist.items().get(index);
            (column > 0)
                .then(|| Span::raw("  "))
                .into_iter()
                .chain(std::iter::once(item.map_or_else(
                    || Span::raw(""),
                    |item| item_span(item, index == app.control_checklist.selected()),
                )))
        });
        Line::from(spans.collect::<Vec<_>>())
    });
    frame.render_widget(Paragraph::new(lines.collect::<Vec<_>>()), area);
}

fn item_span(item: &ChecklistItem, selected: bool) -> Span<'static> {
    let marker = match item.status {
        ChecklistStatus::Pending => "○",
        ChecklistStatus::Observed => "●",
        ChecklistStatus::Skipped => "–",
    };
    let extra = if item.unexpected { " +" } else { "" };
    let count = if item.activation_count > 1 {
        format!(" ×{}", item.activation_count)
    } else {
        String::new()
    };
    let style = match item.status {
        ChecklistStatus::Observed => Style::default().fg(Color::Green),
        ChecklistStatus::Skipped => Style::default().fg(Color::DarkGray),
        ChecklistStatus::Pending => Style::default(),
    }
    .add_modifier(if selected {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    });
    Span::styled(format!("{marker} {:<22}{extra}{count}", item.label), style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_item_has_completed_marker() {
        let item = ChecklistItem {
            key: "button:South".to_owned(),
            label: "South".to_owned(),
            status: ChecklistStatus::Observed,
            activation_count: 2,
            unexpected: false,
        };

        assert!(item_span(&item, false).content.starts_with('●'));
        assert!(item_span(&item, false).content.contains("×2"));
    }
}
