mod layout;

use crate::app::{App, AxisState, DeviceState, Focus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};
use std::cmp;

use self::layout::{ACTIVE_BORDER, WARNING, centered_rect, focused_block};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    if area.width < 54 || area.height < 16 {
        render_compact(frame, app, area);
    } else {
        render_full(frame, app, area);
    }

    if app.help_visible {
        render_help(frame, area);
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
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(vertical[0]);

    render_devices(frame, app, top[0]);
    render_live_state(frame, app, top[1]);
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

fn render_devices(frame: &mut Frame<'_>, app: &App, area: Rect) {
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

fn render_live_state(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = focused_block(" Live state ", app.focus == Focus::LiveState);
    let Some((id, device)) = app.selected_device() else {
        frame.render_widget(Paragraph::new("No controller selected.").block(block), area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 5 {
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(inner);
    render_metadata(frame, id, device, sections[0]);
    render_buttons(frame, device, sections[1]);
    render_axes(frame, device, sections[2]);
}

fn render_metadata(frame: &mut Frame<'_>, id: usize, device: &DeviceState, area: Rect) {
    let connected = if device.connected {
        Span::styled("connected", Style::default().fg(Color::Green))
    } else {
        Span::styled("DISCONNECTED", Style::default().fg(WARNING))
    };
    let vendor = device
        .metadata
        .vendor_id
        .map_or_else(|| "unknown".to_owned(), |value| format!("{value:04x}"));
    let product = device
        .metadata
        .product_id
        .map_or_else(|| "unknown".to_owned(), |value| format!("{value:04x}"));
    let rumble = if device.metadata.rumble_supported {
        "available"
    } else {
        "unavailable"
    };
    let lines = vec![
        Line::from(vec![
            Span::styled(
                &device.metadata.name,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  gilrs:{id}  ")),
            connected,
        ]),
        Line::from(format!(
            "VID:PID {vendor}:{product}  mapping: {}  rumble: {rumble}",
            device.metadata.mapping
        )),
        Line::from(format!(
            "power: {}  uuid: {}",
            device.metadata.power, device.metadata.uuid
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_buttons(frame: &mut Frame<'_>, device: &DeviceState, area: Rect) {
    let mut pressed = device
        .buttons
        .iter()
        .filter(|(_, pressed)| **pressed)
        .map(|(button, _)| format!("{button:?}"))
        .collect::<Vec<_>>();
    pressed.sort_unstable();
    let text = if pressed.is_empty() {
        "Pressed: none".to_owned()
    } else {
        format!("Pressed: {}", pressed.join(", "))
    };
    let observed = device.buttons.len();
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(text),
            Line::from(format!("Observed buttons: {observed}")),
        ]),
        area,
    );
}

fn render_axes(frame: &mut Frame<'_>, device: &DeviceState, area: Rect) {
    if device.axes.is_empty() {
        frame.render_widget(
            Paragraph::new("Axes: move a stick or trigger to populate this table."),
            area,
        );
        return;
    }

    let mut axes = device
        .axes
        .iter()
        .map(|(axis, state)| (format!("{axis:?}"), state))
        .collect::<Vec<_>>();
    axes.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    let rows = axes.into_iter().map(|(name, state)| axis_row(name, state));
    let table = Table::new(
        rows,
        [
            Constraint::Min(13),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(7),
        ],
    )
    .header(
        Row::new(["Axis", "Current", "Min", "Max", "Changes"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn axis_row(name: String, state: &AxisState) -> Row<'static> {
    Row::new([
        name,
        format!("{:+.3}", state.current),
        format!("{:+.3}", state.minimum),
        format!("{:+.3}", state.maximum),
        state.changes.to_string(),
    ])
}

fn render_events(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let title = if app.event_scroll_anchor.is_some() {
        " Recent events — scroll paused "
    } else {
        " Recent events "
    };
    let visible_rows = area.height.saturating_sub(2) as usize;
    let entries = app
        .events
        .iter()
        .filter(|entry| {
            app.event_scroll_anchor
                .is_none_or(|anchor| entry.sequence <= anchor)
        })
        .collect::<Vec<_>>();
    let skip = entries.len().saturating_sub(visible_rows);
    let lines = entries.into_iter().skip(skip).map(|entry| {
        let seconds = entry.elapsed.as_secs();
        let millis = entry.elapsed.subsec_millis();
        let source = entry
            .device_id
            .map_or_else(|| "app".to_owned(), |id| format!("gilrs:{id}"));
        Line::from(format!(
            "{seconds:>5}.{millis:03}  {source}  {}",
            entry.description
        ))
    });
    frame.render_widget(
        Paragraph::new(lines.collect::<Vec<_>>())
            .block(focused_block(title, app.focus == Focus::Events)),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let controls = if app.focus == Focus::Events {
        "q quit | r rumble | tab focus | p pause events | ? help"
    } else {
        "q quit | r rumble | ↑↓/jk select | tab focus | ? help"
    };
    let width = area.width as usize;
    let status_room = width.saturating_sub(controls.len() + 3);
    let status = if status_room > 8 {
        format!(" | {:.status_room$}", app.status)
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(format!("{controls}{status}")).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered_rect(58, 13, area);
    frame.render_widget(Clear, popup);
    let help = Paragraph::new(vec![
        Line::from("Keyboard controls"),
        Line::from(""),
        Line::from("  q / Ctrl-C       Quit"),
        Line::from("  Tab / Shift-Tab  Change focused pane"),
        Line::from("  ↑ ↓ / j k        Select a connected controller"),
        Line::from("  r                 Run a 300 ms rumble test"),
        Line::from("  p                 Pause event auto-scrolling"),
        Line::from("  Esc               Cancel rumble / close help"),
        Line::from("  ?                 Close this help"),
        Line::from(""),
        Line::from("Disconnected selections are retained until you choose another device."),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(ACTIVE_BORDER)),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(help, popup);
}
