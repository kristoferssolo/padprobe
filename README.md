# PadProbe

PadProbe is a Linux-first terminal instrument for inspecting game-controller
input. The current MVP uses `gilrs` to show connected devices, persistent
button and axis state, observed axis ranges, and a bounded recent-event log.

PadProbe is local-first: it has no account, telemetry, network service, or cloud
dependency.

## Current capabilities

- Discovery of controllers present at startup
- Controller connect and disconnect handling
- Explicit multi-controller selection
- Persistent pressed-button and normalized axis state
- Controller-shaped cluster view with circular stick gauges, analog trigger
  bars, diamond-arranged face/D-pad buttons, and observed extra controls
- Session minimum, maximum, and change count for observed axes
- A 256-entry event history with pausable auto-scrolling
- Device name, backend ID, VID/PID, UUID, mapping source, power information,
  and reported rumble support
- A bounded 300 ms rumble test
- Safe terminal restoration on normal exit, initialization failure, and panic
- A compact fallback for terminals smaller than 54 columns by 16 rows
- Optional structured debug logging to a file

## Build and run

PadProbe requires a Rust toolchain and the Linux development dependencies
required by `gilrs` (notably `libudev`).

```console
cargo build --release
./target/release/padprobe
```

Run the complete local check suite with:

```console
just check
```

## Keyboard controls

| Key | Action |
| --- | --- |
| `q`, `Ctrl-C` | Quit |
| `d` | Open the controller selector |
| `↑`/`↓`, `j`/`k` | Select a connected controller inside the selector |
| `Enter`, `Esc` | Close the controller selector |
| `p` | Pause or resume event auto-scrolling |
| `r` | Run a short rumble test when supported |
| `Esc` | Cancel an active rumble test |
| `?` | Open or close help |

A disconnected selected controller remains selected so the disconnection is
visible. Open the selector with `d` to choose another connected controller.

The gamepad view groups related controls into separate boxes. Hollow circles
indicate idle buttons, filled circles indicate pressed buttons, and active
values are highlighted. On smaller terminals PadProbe falls back to the
numerical state table.

## Reusable gamepad widget

The cluster renderer is the backend-neutral
[`gamepad-widget`](gamepad-widget) workspace crate. It
does not depend on `gilrs` or PadProbe application state and can be published
independently:

```rust
use gamepad_widget::{
    ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState,
    GamepadWidget,
};

let state = GamepadState::new([
    ControlCluster::new("Face buttons")
        .with_placement(ClusterPlacement::Face)
        .with_control(Control::new(
            "South",
            ControlValue::Button { pressed: true },
        )),
]);

frame.render_widget(GamepadWidget::new(&state), area);
```

The `gilrs` conversion remains in PadProbe's UI adapter, so consumers can feed
the widget from any controller backend.

## Debug logging

Logging is disabled by default. Set `PADPROBE_LOG` to a file path to enable it;
use `RUST_LOG` to choose the filter:

```console
PADPROBE_LOG=padprobe.log RUST_LOG=padprobe=debug ./target/release/padprobe
```

Logs go to the specified file and do not corrupt the TUI. PadProbe logs
lifecycle, hotplug, and recoverable operation failures, but does not stream
high-rate axis events to the log.

## Measurement limits

Axis values are normalized values reported through `gilrs`. They can be
influenced by driver mappings, Steam Input, configured deadzones, transport,
and OS scheduling. The MVP does not measure physical wear or exact hardware
polling rate, and it does not expose raw Linux input events.

Drift analysis, report export, guided tests, timing statistics, and an optional
raw `evdev` comparison backend remain post-MVP work.

See [the hardware test matrix](docs/hardware-test-matrix.md) for manual
validation status.
