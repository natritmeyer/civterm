use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};

pub(crate) const ACCENT: Color = Color::Rgb(212, 175, 55);
pub(crate) const DIM: Color = Color::Rgb(120, 120, 160);
pub(crate) const HIGHLIGHT_BG: Color = Color::Rgb(45, 45, 95);

pub(crate) fn draw_text(
    buf: &mut Buffer,
    area_right: u16,
    x: u16,
    y: u16,
    text: &str,
    style: Style,
) {
    for (offset, ch) in (x..).zip(text.chars()) {
        if offset >= area_right {
            return;
        }
        if let Some(cell) = buf.cell_mut((offset, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
        }
    }
}
