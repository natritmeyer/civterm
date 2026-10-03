use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::model::civilizations::Civilization;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 42;
const IDEAL_HEIGHT: u16 = 9;

const OK_TEXT: &str = "OK";
const OK_WIDTH: u16 = OK_TEXT.len() as u16 + 2;

/// A modal window announcing that a rival civilization declared war on the
/// player. It carries a single OK button: the player acknowledges the news
/// and play continues under whatever state the war has already put the world
/// in (the rival attacks on the turn it draws the sword).
pub struct WarDialog {
    rival: Civilization,
}

impl WarDialog {
    pub fn new(rival: Civilization) -> Self {
        WarDialog { rival }
    }
}

fn fill_row(buf: &mut Buffer, rect: Rect, style: Style) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            cell.reset();
            cell.set_style(style);
        }
    }
}

/// The display width of a glyph in terminal cells: emoji occupy two cells.
fn cell_width(ch: char) -> u16 {
    if ch as u32 >= 0x1_0000 { 2 } else { 1 }
}

/// Draw `text` starting at `(x, y)`, advancing past the continuation cells of
/// any wide glyphs.
fn draw_text(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    let mut cx = x;
    for ch in text.chars() {
        let width = cell_width(ch);
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
            if width == 2
                && let Some(next) = buf.cell_mut((cx.saturating_add(1), y))
            {
                next.set_diff_option(ratatui::buffer::CellDiffOption::Skip);
            }
        }
        cx = cx.saturating_add(width);
    }
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

/// Fill a rectangle with the vanilla background (and clear any prior paint).
fn fill_rect(buf: &mut Buffer, rect: Rect) {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(TEXT);
            }
        }
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect) {
    let x0 = rect.x;
    let x1 = rect.right() - 1;
    let y0 = rect.y;
    let y1 = rect.bottom() - 1;
    for x in x0..=x1 {
        set_cell(buf, x, y0, "─", TEXT);
        set_cell(buf, x, y1, "─", TEXT);
    }
    for y in y0..=y1 {
        set_cell(buf, x0, y, "│", TEXT);
        set_cell(buf, x1, y, "│", TEXT);
    }
    set_cell(buf, x0, y0, "┌", TEXT);
    set_cell(buf, x1, y0, "┐", TEXT);
    set_cell(buf, x0, y1, "└", TEXT);
    set_cell(buf, x1, y1, "┘", TEXT);
}

/// The rectangle the dialog occupies over `area`, centred across it.
pub fn dialog_rect(area: Rect) -> Rect {
    let width = (IDEAL_WIDTH.min(area.width.saturating_sub(2)).max(2)) & !1;
    let height = (IDEAL_HEIGHT.min(area.height.saturating_sub(2)).max(2)) & !1;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// The interior of the dialog, inside its border.
fn inner(panel: Rect) -> Rect {
    Rect {
        x: panel.x + 1,
        y: panel.y + 1,
        width: panel.width - 2,
        height: panel.height - 2,
    }
}

/// The y row the button sits on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    panel.y + panel.height - 2
}

/// The rectangle of the OK button, right-aligned on the button row.
pub fn ok_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right().saturating_sub(OK_WIDTH + 1),
        y: buttons_y(panel),
        width: OK_WIDTH,
        height: 1,
    }
}

impl Widget for WarDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(buf, inner.x + 1, inner.y, "War!", BOLD);
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 1,
            &format!("The {:?} declare war on you!", self.rival),
            TEXT,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            "Their armies are on the march.",
            TEXT,
        );

        let ok = ok_button_rect(area);
        fill_row(buf, ok, Style::new().fg(BLACK).bg(Color::White));
        draw_text(buf, ok.x + 1, ok.y, OK_TEXT, LINK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(rival: Civilization) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(WarDialog::new(rival), panel()))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn panel() -> Rect {
        dialog_rect(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        })
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn dialog_centres_the_window_in_the_area() {
        let panel = dialog_rect(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        });
        assert_eq!(panel.width, 42);
        assert_eq!(panel.height, 8);
        assert_eq!(panel.x, (100 - 42) / 2);
        assert_eq!(panel.y, (40 - 8) / 2);
    }

    #[test]
    fn the_dialog_names_the_warring_civilization() {
        let buf = render(Civilization::Zulu);
        let panel = panel();
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("Zulu"), "line: {line:?}");
        assert!(line.contains("declare war on you"), "line: {line:?}");
    }

    #[test]
    fn the_ok_button_is_drawn_and_highlighted() {
        let buf = render(Civilization::English);
        let panel = panel();
        let ok = ok_button_rect(panel);
        assert_eq!(
            row_text(&buf, ok.x + 1, ok.y, OK_TEXT.len() as u16),
            OK_TEXT
        );
        assert_eq!(buf.cell((ok.x, ok.y)).unwrap().bg, Color::White);
    }
}
