use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::StarvationNotice;
use crate::tui::window_geometry;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 44;
const IDEAL_HEIGHT: u16 = 9;

const OK_TEXT: &str = "OK";
const OK_WIDTH: u16 = OK_TEXT.len() as u16 + 2;

/// A modal window announcing that a city has starved. It carries a single OK
/// button: the player acknowledges the lost citizen and the window closes.
pub struct StarvationDialog {
    notice: StarvationNotice,
}

impl StarvationDialog {
    pub fn new(notice: StarvationNotice) -> Self {
        StarvationDialog { notice }
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

/// Draw `text` starting at `(x, y)`, stopping at the panel's right edge rather
/// than spilling over the window's border.
fn draw_text(buf: &mut Buffer, x: u16, y: u16, limit: u16, text: &str, style: Style) {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= limit {
            return;
        }
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
        }
        cx = cx.saturating_add(1);
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect) {
    window_geometry::draw_border(buf, rect, TEXT);
}

/// The rectangle the dialog occupies over `area`, centred across it.
pub fn dialog_rect(area: Rect) -> Rect {
    window_geometry::centered(area, IDEAL_WIDTH, IDEAL_HEIGHT)
}

/// The interior of the dialog, inside its border.
fn inner(panel: Rect) -> Rect {
    window_geometry::inner(panel)
}

/// The y row the button sits on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    window_geometry::button_row_y(panel)
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

impl Widget for StarvationDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(
            buf,
            inner.x + 1,
            inner.y,
            inner.right(),
            "Starvation!",
            BOLD,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 1,
            inner.right(),
            &format!("{} is starving.", self.notice.city_name),
            TEXT,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            inner.right(),
            "It has lost 10,000 population.",
            TEXT,
        );

        let ok = ok_button_rect(area);
        fill_row(buf, ok, Style::new().fg(BLACK).bg(Color::White));
        draw_text(buf, ok.x + 1, ok.y, ok.right(), OK_TEXT, LINK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(city: &str) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    StarvationDialog::new(StarvationNotice {
                        city_name: city.to_string(),
                    }),
                    panel(),
                )
            })
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
    fn the_dialog_names_the_starving_city_and_the_loss() {
        let buf = render("London");
        let panel = panel();
        assert_eq!(
            row_text(&buf, panel.x + 2, panel.y + 1, "Starvation!".len() as u16),
            "Starvation!"
        );
        let city = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(city.contains("London"), "city: {city:?}");
        let loss = row_text(&buf, panel.x + 2, panel.y + 3, 40);
        assert!(loss.contains("10,000 population"), "loss: {loss:?}");
    }

    #[test]
    fn the_ok_button_is_drawn_and_highlighted() {
        let buf = render("London");
        let panel = panel();
        let ok = ok_button_rect(panel);
        assert_eq!(
            row_text(&buf, ok.x + 1, ok.y, OK_TEXT.len() as u16),
            OK_TEXT
        );
        assert_eq!(buf.cell((ok.x, ok.y)).unwrap().bg, Color::White);
    }
}
