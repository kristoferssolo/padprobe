mod keymap;

use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event};
use gilrs::{EventType, Gilrs};
use keymap::handle_key;
use padprobe::{
    app::{App, DeviceMetadata},
    logging,
    rumble::RumbleTest,
    terminal::{self, TerminalSession},
    ui,
};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const FRAME_INTERVAL: Duration = Duration::from_millis(33);

fn main() -> Result<()> {
    color_eyre::install()?;
    let log_path = logging::init()?;
    terminal::install_panic_hook();

    let mut gilrs = Gilrs::new().map_err(|error| eyre!("failed to initialize gilrs: {error}"))?;
    let mut app = App::new();
    debug!(?log_path, "gilrs backend initialized");

    for (id, gamepad) in gilrs.gamepads() {
        if gamepad.is_connected() {
            info!(
                gamepad_id = usize::from(id),
                name = gamepad.name(),
                "controller detected"
            );
            app.connect(usize::from(id), DeviceMetadata::from_gamepad(&gamepad));
        }
    }

    let mut terminal = TerminalSession::start()?;
    let now = Instant::now();
    let mut last_frame = now.checked_sub(FRAME_INTERVAL).unwrap_or(now);
    let mut rumble_test = None;

    while !app.should_quit {
        while let Some(event) = gilrs.next_event() {
            let id = usize::from(event.id);
            match event.event {
                EventType::Connected => {
                    info!(
                        gamepad_id = id,
                        name = gilrs.gamepad(event.id).name(),
                        "controller connected"
                    );
                    app.connect(id, DeviceMetadata::from_gamepad(&gilrs.gamepad(event.id)));
                }
                EventType::Disconnected => {
                    info!(gamepad_id = id, "controller disconnected");
                    if rumble_test
                        .as_ref()
                        .is_some_and(|test: &RumbleTest| test.device_id() == id)
                        && let Some(test) = rumble_test.take()
                        && let Err(error) = test.cancel()
                    {
                        warn!(%error, gamepad_id = id, "could not stop disconnected controller rumble");
                    }
                    app.disconnect(id);
                }
                ref event_type => app.apply_controller_event(id, event_type),
            }
        }

        if let Some(test) = rumble_test.take_if(|test| test.is_finished()) {
            app.record_notice_for(Some(test.device_id()), "Rumble test completed");
        }
        app.update_diagnostics(Instant::now());

        if last_frame.elapsed() >= FRAME_INTERVAL {
            terminal.draw(|frame| ui::render(frame, &app))?;
            last_frame = Instant::now();
        }

        if event::poll(FRAME_INTERVAL.saturating_sub(last_frame.elapsed()))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(&mut app, &mut gilrs, &mut rumble_test, key);
        }
    }

    Ok(())
}
