use super::*;
use crate::app::DeviceMetadata;
use ratatui::{Terminal, backend::TestBackend};

fn app() -> App {
    let mut app = App::new();
    app.connect(
        0,
        DeviceMetadata {
            name: "Fixture Controller".to_owned(),
            vendor_id: Some(0x1234),
            product_id: Some(0x5678),
            uuid: "fixture".to_owned(),
            mapping: "standard".to_owned(),
            power: "unknown".to_owned(),
            rumble_supported: false,
        },
    );
    app
}

fn render_text(width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| render(frame, &app()))
        .expect("test frame should render");
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn wide_terminal_renders_dashboard_cards() {
    let rendered = render_text(120, 30);

    assert!(rendered.contains("Controller · Fixture Controller"));
    assert!(rendered.contains("Analog sticks"));
    assert!(rendered.contains("Triggers · 0–1"));
    assert!(rendered.contains("Raw mapped data"));
    assert!(rendered.contains("Recent events"));
    assert!(rendered.contains("L3 ○"));
    assert!(!rendered.contains("VID:PID"));
    assert!(
        rendered
            .chars()
            .filter(|symbol| ('\u{2800}'..='\u{28ff}').contains(symbol))
            .count()
            > 40
    );
}

#[test]
fn medium_terminal_uses_stacked_view() {
    let rendered = render_text(80, 24);

    assert!(rendered.contains("Controller · Fixture Controller"));
    assert!(rendered.contains("Recent events"));
    assert!(!rendered.contains("Analog sticks"));
}

#[test]
fn small_terminal_uses_compact_view() {
    let rendered = render_text(50, 15);

    assert!(rendered.contains("Terminal is too small"));
}

#[test]
fn large_dashboard_is_constrained_to_content_dimensions() {
    assert_eq!(
        dashboard_area(Rect::new(0, 0, 240, 70)),
        Rect::new(30, 0, 180, 29)
    );
    assert_eq!(
        dashboard_area(Rect::new(0, 0, 120, 30)),
        Rect::new(0, 0, 120, 29)
    );
}

#[test]
fn large_dashboard_keeps_events_short_and_wider() {
    let [controller, primary, raw, events] = dashboard_sections(Rect::new(0, 0, 240, 70));

    assert_eq!(controller.height, 28);
    assert_eq!(primary.height, 15);
    assert_eq!(raw.height, 13);
    assert_eq!(events.height, 13);
    assert!(raw.width < events.width);
    assert!(events.width >= 60);
}

#[test]
fn dashboard_has_a_complete_outer_contour() {
    let backend = TestBackend::new(240, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

    terminal
        .draw(|frame| render(frame, &app()))
        .expect("dashboard should render");

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 1)].symbol(), "┌");
    assert_eq!(buffer[(239, 1)].symbol(), "┐");
    assert_eq!(buffer[(0, 38)].symbol(), "└");
    assert_eq!(buffer[(239, 38)].symbol(), "┘");
    assert_eq!(buffer[(0, 1)].fg, ratatui::style::Color::Reset);
}
