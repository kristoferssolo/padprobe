use std::{
    io::{self, Stdout},
    panic,
};

use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    pub fn start() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                restore();
                Err(error.into())
            }
        }
    }

    pub fn draw(
        &mut self,
        render: impl FnOnce(&mut ratatui::Frame<'_>),
    ) -> Result<ratatui::CompletedFrame<'_>> {
        Ok(self.terminal.draw(render)?)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        restore();
    }
}

pub fn install_panic_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore();
        previous_hook(panic_info);
    }));
}

fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}
