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
- Dashboard view with a unified controller overview plus dedicated analog-stick,
  trigger-pressure, raw-data, and recent-event cards
- Dedicated dashboard, drift, range, control-checklist, and timing tabs
- Bounded stick traces with observed resting offset and outer-edge error
- Guided fixed-interval resting-input tests with mean, median, percentile,
  variance, directional-bias, and suggested-deadzone results
- Guided stick range tests with an observed figure, angular coverage,
  circularity deviation, and under/over-range measurements
- Guided control checklists with skipping, unexpected controls, and repeat
  counts
- Signed −1…+1 raw-axis bars and numbered mapped-button indicators
- Session minimum, maximum, and change count for observed axes
- A reset action for selected-device ranges, counters, and traces
- A 256-entry event history with pausable scrolling, kind/device/control
  filtering, display coalescing, clearing, and eviction counts
- Observed event-rate and interval statistics with percentiles, long gaps,
  duplicate frequency, and a histogram
- Versioned JSON and human-readable text report export
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
| `Tab` / `Shift-Tab`, `←` / `→` | Change diagnostic tab |
| `1`–`5` | Select a diagnostic tab |
| `d` | Open the controller selector |
| `↑`/`↓`, `j`/`k` | Select a connected controller inside the selector |
| `Enter`, `Esc` | Close the controller selector |
| `x` | Reset selected-controller observations |
| `e` | Export JSON and text reports to the current directory |
| `p` | Pause or resume event auto-scrolling |
| `↑` / `↓` | Scroll through paused event history |
| `f` | Cycle event-kind filters |
| `v` | Toggle all/selected-device event filtering |
| `/` | Enter a control-name event filter |
| `c` | Clear event history on the dashboard or timing tab |
| `r` | Run a short rumble test when supported |
| `Esc` | Cancel an active guided or rumble test |
| `?` | Open or close help |

A disconnected selected controller remains selected so the disconnection is
visible. Open the selector with `d` to choose another connected controller.

On the Drift and Range tabs, use `l` or `r` to select a stick and `s` to start
the guided test. On the Range tab, press `s` again after tracing the outer edge.
On the Controls tab, press `s` to start, use `j`/`k` to select an item, Space to
skip it, and Enter to finish.

The gamepad overview is a qualitative controller silhouette that places
controls according to their role without assuming Xbox-specific button letters.
Hollow circles indicate idle controls and filled circles indicate active
controls. Exact stick coordinates, traces, offset/error measurements, and
trigger pressure remain in their dedicated diagnostic cards. Smaller terminals
fall back to boxed clusters or the numerical state table.

## Reusable gamepad widget

The cluster renderer is the backend-neutral
[`gamepad-widget`](gamepad-widget) workspace crate. It
does not depend on `gilrs` or PadProbe application state and can be published
independently:

```rust
use gamepad_widget::prelude::*;

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
and OS scheduling. Resting-input and range tests describe reported values; they
do not identify the physical source or declare a controller faulty. Observed
event timing includes driver, operating-system, transport, and API effects and
is not an exact hardware polling-rate measurement.

PadProbe does not currently expose raw Linux input events. An optional `evdev`
comparison backend remains future work.

See [the hardware test matrix](docs/hardware-test-matrix.md) for manual
validation status.
