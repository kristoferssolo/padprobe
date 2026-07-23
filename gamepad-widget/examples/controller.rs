use gamepad_widget::prelude::*;
use ratatui::{Terminal, backend::TestBackend};
use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
    let state = GamepadState::new([
        shoulder(
            "Left shoulder",
            ClusterPlacement::LeftShoulder,
            "LB",
            false,
            "LT",
            0.42,
        ),
        menu(),
        shoulder(
            "Right shoulder",
            ClusterPlacement::RightShoulder,
            "RB",
            true,
            "RT",
            0.78,
        ),
        stick(
            "Left stick",
            ClusterPlacement::LeftStick,
            "L3",
            0.35,
            0.65,
            false,
        ),
        diamond(
            "Face buttons",
            ClusterPlacement::Face,
            [
                ("North", false),
                ("West", false),
                ("East", true),
                ("South", false),
            ],
        ),
        diamond(
            "D-pad",
            ClusterPlacement::DPad,
            [
                ("Up", true),
                ("Left", false),
                ("Right", true),
                ("Down", false),
            ],
        ),
        stick(
            "Right stick",
            ClusterPlacement::RightStick,
            "R3",
            -0.55,
            -0.25,
            true,
        ),
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

fn shoulder(
    title: &str,
    placement: ClusterPlacement,
    bumper: &str,
    bumper_pressed: bool,
    trigger: &str,
    trigger_value: f32,
) -> ControlCluster {
    ControlCluster::new(title)
        .with_placement(placement)
        .with_control(button(bumper, bumper_pressed))
        .with_control(Control::new(
            trigger,
            ControlValue::Trigger {
                value: Some(trigger_value),
            },
        ))
}

fn menu() -> ControlCluster {
    [("Select", false), ("Mode", true), ("Start", false)]
        .into_iter()
        .fold(
            ControlCluster::new("Menu").with_placement(ClusterPlacement::Menu),
            |cluster, (label, pressed)| cluster.with_control(button(label, pressed)),
        )
}

fn stick(
    title: &str,
    placement: ClusterPlacement,
    label: &str,
    x: f32,
    y: f32,
    pressed: bool,
) -> ControlCluster {
    ControlCluster::new(title)
        .with_placement(placement)
        .with_control(Control::new(label, ControlValue::Stick { x, y, pressed }))
}

fn diamond(
    title: &str,
    placement: ClusterPlacement,
    controls: [(&str, bool); 4],
) -> ControlCluster {
    controls.into_iter().fold(
        ControlCluster::new(title).with_placement(placement),
        |cluster, (label, pressed)| cluster.with_control(button(label, pressed)),
    )
}

fn button(label: &str, pressed: bool) -> Control {
    Control::new(label, ControlValue::Button { pressed })
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
