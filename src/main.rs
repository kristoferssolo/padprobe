use color_eyre::eyre::eyre;
use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    io::{self},
    thread,
    time::Duration,
};

const REDRAW_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> color_eyre::Result<()> {
    use io::Write;
    color_eyre::install()?;

    let mut gilrs = Gilrs::new().map_err(|error| eyre!("failed to initialize gilrs: {error}"))?;
    let mut state = ControllerState::default();

    if let Some((id, gamepad)) = gilrs.gamepads().find(|(_, gamepad)| gamepad.is_connected()) {
        state.select(id, gamepad.name());
    }

    loop {
        while let Some(event) = gilrs.next_event() {
            let gamepad = gilrs.gamepad(event.id);

            match event.event {
                EventType::Connected => {
                    if state.id.is_none() {
                        state.select(event.id, gamepad.name())
                    }
                }
                EventType::Disconnected => {
                    if state.is_selected(event.id) {
                        state.deselect()
                    }
                }
                event_type if state.is_selected(event.id) => state.apply_event(&event_type),
                _ => {}
            }
        }

        print!("\x1B[2J\x1B[H{}", state.render_snapshot());
        io::stdout().flush()?;

        thread::sleep(REDRAW_INTERVAL);
    }
}

#[derive(Debug, Default)]
struct ControllerState {
    id: Option<GamepadId>,
    name: Option<String>,
    axes: HashMap<Axis, f32>,
    pressed_buttons: HashSet<Button>,
    last_event: Option<String>,
}

impl ControllerState {
    fn select(&mut self, id: GamepadId, name: impl Into<String>) {
        self.id = Some(id);
        self.name = Some(name.into());
        self.axes.clear();
        self.pressed_buttons.clear();
        self.last_event = Some("controller selected".into());
    }

    fn deselect(&mut self) {
        self.id = None;
        self.name = None;
        self.axes.clear();
        self.pressed_buttons.clear();
        self.last_event = Some("controller disconnected".into());
    }

    fn is_selected(&self, id: GamepadId) -> bool {
        self.id == Some(id)
    }

    fn apply_event(&mut self, event: &EventType) {
        self.last_event = Some(format!("{event:?}"));

        match event {
            EventType::ButtonPressed(button, _) => {
                self.pressed_buttons.insert(*button);
            }
            EventType::ButtonReleased(button, _) => {
                self.pressed_buttons.remove(button);
            }
            EventType::AxisChanged(axis, value, _) => {
                self.axes.insert(*axis, *value);
            }
            _ => {}
        };
    }

    fn render_snapshot(&self) -> String {
        let mut output = String::new();

        writeln!(output, "PadProbe").unwrap();

        if let (Some(id), Some(name)) = (self.id, &self.name) {
            writeln!(output, "\nController: {name} ({id:?})").unwrap();
            writeln!(output, "\nPressed buttons:").unwrap();

            if self.pressed_buttons.is_empty() {
                writeln!(output, "    None").unwrap();
            } else {
                let mut buttons = self
                    .pressed_buttons
                    .iter()
                    .map(|button| format!("{button:?}"))
                    .collect::<Vec<_>>();

                buttons.sort();

                for button in buttons {
                    writeln!(output, "    {button}").unwrap();
                }
            }

            writeln!(output, "\nObserved axes:").unwrap();

            if self.axes.is_empty() {
                writeln!(output, "    Move a stick or trigger to populate this list.").unwrap();
            } else {
                let mut axes = self
                    .axes
                    .iter()
                    .map(|(axis, value)| (format!("{axis:?}"), *value))
                    .collect::<Vec<_>>();

                axes.sort_by(|left, right| left.0.cmp(&right.0));

                for (axis, value) in axes {
                    writeln!(output, "    {axis:<16} {value:+.3}").unwrap();
                }
            }
        } else {
            writeln!(output, "\nNo controller selected.").unwrap();
            writeln!(output, "Connect a controller to begin.").unwrap();
        }

        writeln!(output, "\nLast event:").unwrap();

        match &self.last_event {
            Some(event) => writeln!(output, "    {event}").unwrap(),
            None => writeln!(output, "    None").unwrap(),
        }

        writeln!(output, "\nPress Ctrl-C to quit.").unwrap();

        output
    }
}
