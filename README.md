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
- Abstract mapped-controller view with stick direction, trigger levels, mapped
  buttons, and D-pad state
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
| `Tab`, `Shift-Tab` | Change focused pane |
| `p` | Pause or resume event auto-scrolling while the event pane is focused |
| `r` | Run a short rumble test when supported |
| `Esc` | Cancel an active rumble test |
| `?` | Open or close help |

A disconnected selected controller remains selected so the disconnection is
visible. Open the selector with `d` to choose another connected controller.

The abstract gamepad uses lowercase labels and hollow D-pad symbols for idle
controls, then uppercase labels and filled symbols for active controls. It is a
mapped-input overview; the adjacent numerical table remains authoritative and
continues to expose unknown controls.

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
