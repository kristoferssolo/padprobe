# gamepad-widget

A backend-neutral [Ratatui](https://ratatui.rs/) widget for displaying gamepad
state as controller-positioned control clusters.

The widget provides:

- semantic placement for shoulders, menu buttons, sticks, D-pad, face buttons,
  and extra controls;
- circular stick gauges with direction, magnitude, coordinates, and click state;
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

The semantic controller layout is used when the available area is at least
88×17 cells. Smaller areas use an ordered responsive grid, so arbitrary control
clusters remain usable.
