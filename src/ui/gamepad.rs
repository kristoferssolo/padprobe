use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

const SHELL: [&str; 9] = [
    "       ╭── LT ──╮          ╭── RT ──╮",
    "       ╰── LB ──╯          ╰── RB ──╯",
    "      ╭──────────────────────────────╮",
    "   ╭──╯   ( LS )    ◦    ◦      N    ╰──╮",
    "  ╱             ╭─┬─╮         W   E     ╲",
    " │              ├─┼─┤           S         │",
    "  ╲             ╰─┴─╯     ( RS )         ╱",
    "   ╰──╮         ╭────────────╮       ╭──╯",
    "      ╰─────────╯            ╰───────╯",
];

pub(super) fn render_gamepad(frame: &mut Frame<'_>, area: Rect) {
    let lines = SHELL
        .into_iter()
        .map(|line| Line::styled(line, Style::default().fg(Color::DarkGray)))
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}
