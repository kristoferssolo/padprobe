use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use gilrs::{EventType, Gilrs};
use padprobe::{
    analysis::StickSide,
    app::{App, AppTab, DeviceMetadata, EventSearchState},
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
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?' | 'q')) {
            app.help_visible = false;
        }
        return;
    }

    if app.device_selector_visible {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('d') => app.close_device_selector(),
            _ => {}
        }
        return;
    }

    if app.event_search_state == EventSearchState::Open {
        match key.code {
            KeyCode::Enter => app.event_search_state = EventSearchState::Closed,
            KeyCode::Esc => {
                app.event_search.clear();
                app.event_search_state = EventSearchState::Closed;
            }
            KeyCode::Backspace => {
                app.event_search.pop();
            }
            KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.event_search.push(character);
            }
            _ => {}
        }
        return;
    }

    if handle_diagnostic_key(app, key) {
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.help_visible = true,
        KeyCode::Char('d') => app.open_device_selector(),
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
        KeyCode::Char('r') => {
            if let Some(test) = rumble_test.take()
                && let Err(error) = test.cancel()
            {
                warn!(%error, "could not stop previous rumble test");
            }
            match RumbleTest::start(gilrs, app.selected_id) {
                Ok(test) => {
                    app.record_notice("Running short rumble test — Esc cancels");
                    *rumble_test = Some(test);
                }
                Err(error) => {
                    warn!(%error, "rumble test unavailable");
                    app.record_notice(error.to_string());
                }
            }
        }
        KeyCode::Char('p') => {
            app.toggle_event_scroll();
        }
        KeyCode::Char('/') => app.event_search_state = EventSearchState::Open,
        KeyCode::Char('f') => app.cycle_event_kind_filter(),
        KeyCode::Char('v') => app.toggle_event_device_filter(),
        KeyCode::Char('c') if app.active_tab == AppTab::Dashboard => app.clear_events(),
        KeyCode::Up if app.active_tab == AppTab::Dashboard => app.scroll_events_older(),
        KeyCode::Down if app.active_tab == AppTab::Dashboard => app.scroll_events_newer(),
        KeyCode::Char('x') => app.reset_selected_observations(),
        KeyCode::Tab | KeyCode::Right => app.select_next_tab(),
        KeyCode::BackTab | KeyCode::Left => app.select_previous_tab(),
        KeyCode::Char('1') => app.active_tab = AppTab::Dashboard,
        KeyCode::Char('2') => app.active_tab = AppTab::Drift,
        KeyCode::Char('3') => app.active_tab = AppTab::Range,
        KeyCode::Char('4') => app.active_tab = AppTab::Controls,
        KeyCode::Char('5') => app.active_tab = AppTab::Timing,
        _ => {}
    }
}

fn handle_diagnostic_key(app: &mut App, key: KeyEvent) -> bool {
    match (app.active_tab, key.code) {
        (AppTab::Drift, KeyCode::Char('l')) => app.select_drift_stick(StickSide::Left),
        (AppTab::Drift, KeyCode::Char('r')) => app.select_drift_stick(StickSide::Right),
        (AppTab::Drift, KeyCode::Char('s')) => app.start_drift_test(Instant::now()),
        (AppTab::Drift, KeyCode::Esc) => app.cancel_drift_test(),
        (AppTab::Range, KeyCode::Char('l')) => app.select_range_stick(StickSide::Left),
        (AppTab::Range, KeyCode::Char('r')) => app.select_range_stick(StickSide::Right),
        (AppTab::Range, KeyCode::Char('s')) => app.toggle_range_test(),
        (AppTab::Range, KeyCode::Esc) => app.cancel_range_test(),
        (AppTab::Controls, KeyCode::Char('s')) => app.start_control_checklist(),
        (AppTab::Controls, KeyCode::Enter) => app.finish_control_checklist(),
        (AppTab::Controls, KeyCode::Down | KeyCode::Char('j')) => {
            app.control_checklist.select_next();
        }
        (AppTab::Controls, KeyCode::Up | KeyCode::Char('k')) => {
            app.control_checklist.select_previous();
        }
        (AppTab::Controls, KeyCode::Char(' ')) => app.control_checklist.skip_selected(),
        _ => return false,
    }
    true
}
