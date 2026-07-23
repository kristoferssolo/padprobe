use ratatui::{buffer::Buffer, layout::Rect, style::Style};

pub(super) fn render_shell(area: Rect, buffer: &mut Buffer, style: Style) {
    let left = area.x;
    let right = area.right().saturating_sub(1);
    let top = area.y + 6;
    draw_horizontal(buffer, left + 11, right - 9, top, style);
    buffer[(left + 10, top)].set_char('╭').set_style(style);
    buffer[(right - 10, top)].set_char('╮').set_style(style);

    draw_horizontal(buffer, left + 7, left + 10, top + 1, style);
    draw_horizontal(buffer, right - 9, right - 6, top + 1, style);
    buffer[(left + 6, top + 1)].set_char('╭').set_style(style);
    buffer[(left + 10, top + 1)].set_char('╯').set_style(style);
    buffer[(right - 10, top + 1)].set_char('╰').set_style(style);
    buffer[(right - 6, top + 1)].set_char('╮').set_style(style);

    draw_horizontal(buffer, left + 4, left + 6, top + 2, style);
    draw_horizontal(buffer, right - 5, right - 3, top + 2, style);
    buffer[(left + 3, top + 2)].set_char('╭').set_style(style);
    buffer[(left + 6, top + 2)].set_char('╯').set_style(style);
    buffer[(right - 6, top + 2)].set_char('╰').set_style(style);
    buffer[(right - 3, top + 2)].set_char('╮').set_style(style);

    buffer[(left + 2, top + 3)].set_char('╱').set_style(style);
    buffer[(right - 2, top + 3)].set_char('╲').set_style(style);
    buffer[(left + 1, top + 4)].set_char('╱').set_style(style);
    buffer[(right - 1, top + 4)].set_char('╲').set_style(style);
    for y in top + 5..top + 14 {
        buffer[(left + 1, y)].set_char('│').set_style(style);
        buffer[(right - 1, y)].set_char('│').set_style(style);
    }

    buffer[(left + 1, top + 14)].set_char('╲').set_style(style);
    buffer[(right - 1, top + 14)].set_char('╱').set_style(style);
    buffer[(left + 2, top + 15)].set_char('╲').set_style(style);
    buffer[(right - 2, top + 15)].set_char('╱').set_style(style);
    draw_horizontal(buffer, left + 19, right - 18, top + 15, style);
    buffer[(left + 18, top + 15)].set_char('╭').set_style(style);
    buffer[(right - 18, top + 15)]
        .set_char('╮')
        .set_style(style);

    buffer[(left + 3, top + 16)].set_char('╲').set_style(style);
    buffer[(left + 17, top + 16)].set_char('╱').set_style(style);
    buffer[(right - 17, top + 16)]
        .set_char('╲')
        .set_style(style);
    buffer[(right - 3, top + 16)].set_char('╱').set_style(style);
    buffer[(left + 4, top + 17)].set_char('╲').set_style(style);
    buffer[(left + 16, top + 17)].set_char('╱').set_style(style);
    buffer[(right - 16, top + 17)]
        .set_char('╲')
        .set_style(style);
    buffer[(right - 4, top + 17)].set_char('╱').set_style(style);

    buffer[(left + 5, top + 18)].set_char('╰').set_style(style);
    buffer[(left + 15, top + 18)].set_char('╯').set_style(style);
    buffer[(right - 15, top + 18)]
        .set_char('╰')
        .set_style(style);
    buffer[(right - 5, top + 18)].set_char('╯').set_style(style);
    draw_horizontal(buffer, left + 6, left + 15, top + 18, style);
    draw_horizontal(buffer, right - 14, right - 5, top + 18, style);
}

fn draw_horizontal(buffer: &mut Buffer, start: u16, end: u16, y: u16, style: Style) {
    for x in start..end {
        buffer[(x, y)].set_char('─').set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_has_shoulders_body_notch_and_grips() {
        let area = Rect::new(0, 0, 70, 25);
        let mut buffer = Buffer::empty(area);

        render_shell(area, &mut buffer, Style::default());

        assert_eq!(buffer[(10, 6)].symbol(), "╭");
        assert_eq!(buffer[(59, 6)].symbol(), "╮");
        assert_eq!(buffer[(1, 11)].symbol(), "│");
        assert_eq!(buffer[(18, 21)].symbol(), "╭");
        assert_eq!(buffer[(51, 21)].symbol(), "╮");
        assert_eq!(buffer[(5, 24)].symbol(), "╰");
        assert_eq!(buffer[(15, 24)].symbol(), "╯");
        assert_eq!(buffer[(54, 24)].symbol(), "╰");
        assert_eq!(buffer[(64, 24)].symbol(), "╯");
    }
}
