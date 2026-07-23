use gamepad_widget::prelude::*;
use ratatui::{Terminal, backend::TestBackend};
use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
    let trace = edge_trace();
    let mut terminal = Terminal::new(TestBackend::new(36, 18))?;
    terminal.draw(|frame| {
        frame.render_widget(
            StickGauge::new("Right stick", 0.62, -0.48)
                .button("R3", true)
                .trace(&trace)
                .metric("p95 rest 2.8% · edge error 2.1%"),
            frame.area(),
        );
    })?;
    print_buffer(terminal.backend().buffer());
    Ok(())
}

fn edge_trace() -> Vec<(f64, f64)> {
    const SAMPLE_COUNT: u16 = 96;

    (0..=SAMPLE_COUNT)
        .map(|sample| {
            let angle = f64::from(sample) / f64::from(SAMPLE_COUNT) * std::f64::consts::TAU;
            let radius = (angle * 4.0).sin().mul_add(0.04, 0.96);
            (angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
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
