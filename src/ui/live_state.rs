use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

use crate::app::{App, AxisState, DeviceState, Focus};

use super::gamepad::render_gamepad;
use super::layout::{WARNING, focused_block};

pub(super) fn render_live_state(frame: &mut Frame<'_>, app: &App, area: Rect) {
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
        .constraints([Constraint::Length(4), Constraint::Min(2)])
        .split(inner);
    render_metadata(frame, id, device, sections[0]);
    if sections[1].width >= 72 && sections[1].height >= 13 {
        if sections[1].height >= 22 {
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(13), Constraint::Min(8)])
                .split(sections[1]);
            render_gamepad(frame, device, body[0]);
            let measurements = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(2)])
                .split(body[1]);
            render_buttons(frame, device, measurements[0]);
            render_axes(frame, device, measurements[1]);
        } else {
            render_gamepad(frame, device, sections[1]);
        }
    } else {
        let measurements = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(2)])
            .split(sections[1]);
        render_buttons(frame, device, measurements[0]);
        render_axes(frame, device, measurements[1]);
    }
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
        Line::from(
            "cluster view: ○ idle, ● active; values are normalized mapped input reported by gilrs",
        )
        .style(Style::default().fg(Color::DarkGray)),
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
    let mut fallback = Vec::new();
    if device.buttons.contains_key(&Button::Unknown) {
        fallback.push("unknown button");
    }
    if device.axes.contains_key(&Axis::Unknown) {
        fallback.push("unknown axis");
    }
    let fallback = if fallback.is_empty() {
        String::new()
    } else {
        format!(" | observed: {}", fallback.join(", "))
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(text),
            Line::from(format!("Observed buttons: {observed}{fallback}")),
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
