use gamepad_widget::prelude::*;
use ratatui::{Terminal, backend::TestBackend};
use std::convert::Infallible;

const TRACE: &[(f64, f64)] = &[(0.0, 0.0), (0.2, 0.3), (0.5, 0.7), (0.8, 0.5), (1.0, 0.0)];

fn main() -> Result<(), Infallible> {
    let mut terminal = Terminal::new(TestBackend::new(24, 14))?;
    terminal.draw(|frame| {
        frame.render_widget(
            StickGauge::new("Left stick", 0.8, 0.5)
                .button("L3", false)
                .trace(TRACE)
                .metric("edge deviation 2.1%"),
            frame.area(),
        );
    })?;
    print_buffer(terminal.backend().buffer());
    Ok(())
}

fn print_buffer(buffer: &ratatui::buffer::Buffer) {
    for row in buffer.content().chunks(usize::from(buffer.area.width)) {
        println!(
            "{}",
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        );
    }
}
