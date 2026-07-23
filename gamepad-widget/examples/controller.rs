use gamepad_widget::prelude::*;
use ratatui::{Terminal, backend::TestBackend};
use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
    let state = GamepadState::new([
        ControlCluster::new("Left shoulder")
            .with_placement(ClusterPlacement::LeftShoulder)
            .with_control(Control::new("LB", ControlValue::Button { pressed: false }))
            .with_control(Control::new(
                "LT",
                ControlValue::Trigger { value: Some(0.4) },
            )),
        ControlCluster::new("Menu")
            .with_placement(ClusterPlacement::Menu)
            .with_control(Control::new(
                "Select",
                ControlValue::Button { pressed: false },
            ))
            .with_control(Control::new(
                "Start",
                ControlValue::Button { pressed: true },
            )),
        ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(Control::new(
                "L3",
                ControlValue::Stick {
                    x: 0.35,
                    y: 0.65,
                    pressed: false,
                },
            )),
        ControlCluster::new("Face")
            .with_placement(ClusterPlacement::Face)
            .with_control(Control::new(
                "North",
                ControlValue::Button { pressed: false },
            ))
            .with_control(Control::new(
                "West",
                ControlValue::Button { pressed: false },
            ))
            .with_control(Control::new("East", ControlValue::Button { pressed: true }))
            .with_control(Control::new(
                "South",
                ControlValue::Button { pressed: false },
            )),
    ]);
    let mut terminal = Terminal::new(TestBackend::new(70, 25))?;
    terminal.draw(|frame| {
        frame.render_widget(
            GamepadWidget::new(&state).layout(GamepadLayout::Controller),
            frame.area(),
        );
    })?;
    print_buffer(terminal.backend().buffer());
    Ok(())
}

fn print_buffer(buffer: &ratatui::buffer::Buffer) {
    for row in buffer.content().chunks(usize::from(buffer.area.width)) {
        println!(
            "{}",
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        );
    }
}
