# gamepad-widget

A backend-neutral [Ratatui](https://ratatui.rs/) widget for displaying gamepad
state as a unified controller overview.

The widget provides:

- semantic placement for shoulders, menu buttons, sticks, D-pad, and face
  buttons without nested cluster borders;
- scalable Braille-resolution stick gauges with direction, magnitude,
  coordinates, and click state;
- diamond layouts for D-pad and face buttons;
- analog trigger bars;
- a responsive grid fallback for narrow or short terminal areas.

It has no controller-backend dependency. Applications translate input from
`gilrs`, `evdev`, SDL, or another source into the crate's `GamepadState`.

```rust
use gamepad_widget::prelude::*;

let state = GamepadState::new([
    ControlCluster::new("Face buttons")
        .with_placement(ClusterPlacement::Face)
        .with_control(Control::new(
            "North",
            ControlValue::Button { pressed: false },
        ))
        .with_control(Control::new(
            "West",
            ControlValue::Button { pressed: false },
        ))
        .with_control(Control::new(
            "East",
            ControlValue::Button { pressed: true },
        ))
        .with_control(Control::new(
            "South",
            ControlValue::Button { pressed: false },
        )),
]);

frame.render_widget(GamepadWidget::new(&state), area);
```

`StickGauge` is also available independently when an application needs a
larger dedicated analog-stick view:

```rust
use gamepad_widget::prelude::*;

frame.render_widget(
    StickGauge::new("Left stick", x, y).button("L3", pressed),
    area,
);
```

The gauge uses the 2×4 dot grid in each Unicode Braille cell, allowing its
circle and crosshair to scale smoothly with the available terminal area.

The unified controller layout is used when the available area is at least
44×21 cells. Smaller areas, or states containing extra unplaced controls, use
an ordered responsive grid so arbitrary clusters remain visible.
