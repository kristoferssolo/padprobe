use super::*;
use crate::{Control, ControlValue};
use ratatui::style::Color;

#[test]
fn shoulder_art_separates_trigger_and_bumper() {
    let cluster = ControlCluster::new("Left shoulder")
        .with_placement(ClusterPlacement::LeftShoulder)
        .with_controls([
            Control::new("LB", ControlValue::button(false)),
            Control::new("LT", ControlValue::trigger(0.5)),
        ]);
    let state = GamepadState::default();
    let widget = GamepadWidget::new(&state);
    let area = Rect::new(0, 0, 9, 6);
    let mut buffer = Buffer::empty(area);

    render_shoulder(Some(&cluster), 4, 0, &mut buffer, widget);
    let symbols = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>();

    assert!(symbols.contains("LT"));
    assert!(symbols.contains("LB"));
    assert!(buffer.content().iter().any(|cell| cell.fg == Color::Cyan));
    assert!(
        buffer
            .content()
            .iter()
            .any(|cell| cell.fg == Color::DarkGray)
    );
}
