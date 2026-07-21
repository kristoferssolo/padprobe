use std::time::{Duration, Instant};

use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use gilrs::{EventType, Gilrs};
use padprobe::{
    app::{App, DeviceMetadata, Focus},
    terminal::{self, TerminalSession},
    ui,
};

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

    while !app.should_quit {
        while let Some(event) = gilrs.next_event() {
            let id = usize::from(event.id);
            match event.event {
                EventType::Connected => {
                    app.connect(id, DeviceMetadata::from_gamepad(&gilrs.gamepad(event.id)));
                }
                EventType::Disconnected => app.disconnect(id),
                ref event_type => app.apply_controller_event(id, event_type),
            }
        }

        if last_frame.elapsed() >= FRAME_INTERVAL {
            terminal.draw(|frame| ui::render(frame, &app))?;
            last_frame = Instant::now();
        }

        if event::poll(FRAME_INTERVAL.saturating_sub(last_frame.elapsed()))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(&mut app, key);
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
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
        KeyCode::Esc => app.help_visible = false,
        KeyCode::Tab => app.focus = app.focus.next(),
        KeyCode::BackTab => app.focus = app.focus.previous(),
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
