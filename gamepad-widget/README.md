# gamepad-widget

A backend-neutral [Ratatui](https://ratatui.rs/) widget for displaying gamepad
state as a unified controller overview.

The widget provides:

- a qualitative controller silhouette with semantic placement for shoulders,
  menu buttons, sticks, D-pad, and face buttons;
- scalable Braille-resolution stick gauges with direction, magnitude,
  coordinates, click state, observed traces, and optional metrics;
- diamond layouts for D-pad and face buttons;
- analog trigger bars;
- reusable color and monochrome themes;
- explicit automatic, controller, and grid layout policies;
- a responsive grid fallback for narrow or short terminal areas.

It has no controller-backend dependency. Applications translate input from
`gilrs`, `evdev`, SDL, or another source into the crate's `GamepadState`.

```rust
use gamepad_widget::prelude::*;

let state = GamepadState::new([
    ControlCluster::new("Face buttons")
        .with_placement(ClusterPlacement::Face)
        .with_controls([
            Control::new("North", ControlValue::button(false)),
            Control::new("West", ControlValue::button(false)),
            Control::new("East", ControlValue::button(true)),
            Control::new("South", ControlValue::button(false)),
        ]),
]);

frame.render_widget(GamepadWidget::new(&state), area);
```

Use `GamepadLayout::Grid` to always show boxed clusters, or
`GamepadLayout::Controller` to prefer the controller silhouette while retaining
the safe grid fallback for unsupported states:

```rust
frame.render_widget(
    GamepadWidget::new(&state)
        .layout(GamepadLayout::Grid)
        .theme(GamepadTheme::monochrome()),
    area,
);
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

The gauge renders Ratatui canvas `Circle` and `Line` shapes using the 2×4 dot
grid in each Unicode Braille cell. Its viewport preserves a two-columns-per-row
terminal aspect ratio so the gate remains circular as the available area
changes.

Runnable examples render into Ratatui's test backend and print the result:

```console
cargo run -p gamepad-widget --example controller
cargo run -p gamepad-widget --example stick
```

The unified controller layout is used when the available area is at least
48×25 cells. Smaller areas, or states containing extra unplaced controls, use
an ordered responsive grid so arbitrary clusters remain visible.
