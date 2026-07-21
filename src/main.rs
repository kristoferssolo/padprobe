use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use gilrs::{EventType, Gilrs};
use padprobe::{
    app::{App, DeviceMetadata, Focus},
    rumble::RumbleTest,
    terminal::{self, TerminalSession},
    ui,
};
use std::time::{Duration, Instant};

const FRAME_INTERVAL: Duration = Duration::from_millis(33);

fn main() -> Result<()> {
    color_eyre::install()?;
    terminal::install_panic_hook();

    let mut gilrs = Gilrs::new().map_err(|error| eyre!("failed to initialize gilrs: {error}"))?;
    let mut app = App::new();

    for (id, gamepad) in gilrs.gamepads() {
        if gamepad.is_connected() {
            app.connect(usize::from(id), DeviceMetadata::from_gamepad(&gamepad));
        }
    }

    let mut terminal = TerminalSession::start()?;
    let mut last_frame = Instant::now() - FRAME_INTERVAL;
    let mut rumble_test = None;

    while !app.should_quit {
        while let Some(event) = gilrs.next_event() {
            let id = usize::from(event.id);
            match event.event {
                EventType::Connected => {
                    app.connect(id, DeviceMetadata::from_gamepad(&gilrs.gamepad(event.id)));
                }
                EventType::Disconnected => {
                    if rumble_test
                        .as_ref()
                        .is_some_and(|test: &RumbleTest| test.device_id() == id)
                    {
                        let _ = rumble_test.take().and_then(|test| test.cancel().err());
                    }
                    app.disconnect(id);
                }
                ref event_type => app.apply_controller_event(id, event_type),
            }
        }

        if rumble_test.as_ref().is_some_and(RumbleTest::is_finished) {
            let device_id = rumble_test.as_ref().map(RumbleTest::device_id);
            rumble_test = None;
            app.record_notice_for(device_id, "Rumble test completed");
        }

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

fn handle_key(
    app: &mut App,
    gilrs: &mut Gilrs,
    rumble_test: &mut Option<RumbleTest>,
    key: KeyEvent,
) {
    if key.kind.is_release() {
        return;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if app.help_visible {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
        ) {
            app.help_visible = false;
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.help_visible = true,
        KeyCode::Esc => {
            if let Some(test) = rumble_test.take() {
                let device_id = test.device_id();
                let message = match test.cancel() {
                    Ok(()) => "Rumble test cancelled".to_owned(),
                    Err(error) => format!("Could not cancel rumble test: {error}"),
                };
                app.record_notice_for(Some(device_id), message);
            }
        }
        KeyCode::Tab => app.focus = app.focus.next(),
        KeyCode::BackTab => app.focus = app.focus.previous(),
        KeyCode::Char('r') => {
            if let Some(test) = rumble_test.take() {
                let _ = test.cancel();
            }
            match RumbleTest::start(gilrs, app.selected_id) {
                Ok(test) => {
                    app.record_notice("Running short rumble test — Esc cancels");
                    *rumble_test = Some(test);
                }
                Err(error) => app.record_notice(error.to_string()),
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.focus == Focus::Devices => {
            app.select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') if app.focus == Focus::Devices => {
            app.select_next();
        }
        KeyCode::Char('p') if app.focus == Focus::Events => {
            app.toggle_event_scroll();
        }
        _ => {}
    }
}
