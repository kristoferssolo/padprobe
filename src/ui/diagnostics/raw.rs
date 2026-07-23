use super::{axis_value, diagnostic_block};
use crate::app::{App, DeviceState};
use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

const NUMBERED_BUTTONS: [Button; 19] = [
    Button::South,
    Button::East,
    Button::North,
    Button::West,
    Button::C,
    Button::Z,
    Button::LeftTrigger,
    Button::LeftTrigger2,
    Button::RightTrigger,
    Button::RightTrigger2,
    Button::Select,
    Button::Start,
    Button::Mode,
    Button::LeftThumb,
    Button::RightThumb,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

pub(in crate::ui) fn render_raw_data(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = diagnostic_block(" Raw mapped data ", Color::Blue);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some((_, device)) = app.selected_device() else {
        frame.render_widget(Paragraph::new("No controller selected."), inner);
        return;
    };

    let mut pressed = device
        .buttons
        .iter()
        .filter(|(_, pressed)| **pressed)
        .map(|(button, _)| format!("{button:?}"))
        .collect::<Vec<_>>();
    pressed.sort_unstable();
    let pressed = if pressed.is_empty() {
        "none".to_owned()
    } else {
        pressed.join(", ")
    };
    let mut lines = vec![
        Line::styled(
            format!("gilrs · {}", device.metadata.mapping),
            Style::default().fg(Color::DarkGray),
        ),
        Line::styled("AXES −1…+1", Style::default().add_modifier(Modifier::BOLD)),
    ];
    lines.extend(
        [
            ("LX", Axis::LeftStickX),
            ("LY", Axis::LeftStickY),
            ("RX", Axis::RightStickX),
            ("RY", Axis::RightStickY),
        ]
        .map(|(label, axis)| signed_axis_bar(label, axis_value(device, axis), inner.width)),
    );
    lines.push(Line::styled(
        "BUTTONS",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    lines.extend(button_grid(device, inner.width));
    lines.push(Line::from(format!("Pressed: {pressed}")));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn signed_axis_bar(label: &str, value: f32, width: u16) -> Line<'static> {
    let bar_width = usize::from(width.saturating_sub(18).clamp(7, 17));
    let value = value.clamp(-1.0, 1.0);
    let position = normalized_bar_position(value, bar_width);
    let mut bar = vec!['─'; bar_width];
    bar[bar_width / 2] = '┼';
    bar[position] = '●';
    Line::from(format!(
        "{label:<2} −1 [{}] +1 {value:+.2}",
        bar.into_iter().collect::<String>()
    ))
}

#[inline]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the normalized, clamped value maps to a bar no wider than 17 cells"
)]
fn normalized_bar_position(value: f32, bar_width: usize) -> usize {
    let last_index = u16::try_from(bar_width.saturating_sub(1)).unwrap_or(u16::MAX);
    (value.midpoint(1.0) * f32::from(last_index)).round() as usize
}

fn button_grid(device: &DeviceState, width: u16) -> Vec<Line<'static>> {
    const CELL_WIDTH: u16 = 5;
    let columns = usize::from((width / CELL_WIDTH).max(1));
    NUMBERED_BUTTONS
        .chunks(columns)
        .enumerate()
        .map(|(row, buttons)| {
            let spans = buttons
                .iter()
                .enumerate()
                .flat_map(|(column, button)| {
                    let index = row * columns + column;
                    let pressed = device.buttons.get(button).copied().unwrap_or(false);
                    (column > 0)
                        .then(|| Span::raw(" "))
                        .into_iter()
                        .chain(std::iter::once(Span::styled(
                            format!("[{index:>2}]"),
                            if pressed {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        )))
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::DeviceMetadata;

    #[test]
    fn signed_axis_bar_marks_center_and_extremes() {
        assert!(signed_axis_bar("LX", -1.0, 40).to_string().contains("[●"));
        assert!(signed_axis_bar("LX", 0.0, 40).to_string().contains('●'));
        assert!(signed_axis_bar("LX", 1.0, 40).to_string().contains("●]"));
    }

    #[test]
    fn button_grid_wraps_numbered_cells() {
        let mut app = App::new();
        app.connect(
            0,
            DeviceMetadata {
                name: "fixture".to_owned(),
                vendor_id: None,
                product_id: None,
                uuid: String::new(),
                mapping: "test".to_owned(),
                power: "unknown".to_owned(),
                rumble_supported: false,
            },
        );
        let device = app.devices.get_mut(&0).expect("device should exist");
        device.buttons.insert(Button::South, true);

        let lines = button_grid(device, 20);

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].spans[0].content, "[ 0]");
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Cyan));
    }
}
