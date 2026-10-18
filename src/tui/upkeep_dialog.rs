use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::UpkeepNotice;
use crate::tui::window_geometry;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 46;
const IDEAL_HEIGHT: u16 = 9;

const OK_TEXT: &str = "OK";
const OK_WIDTH: u16 = OK_TEXT.len() as u16 + 2;

/// A modal window announcing that a civilization could not pay its
/// improvements' upkeep. It carries a single OK button: the player
/// acknowledges the empty treasury and the disbanded unit, and it closes.
pub struct UpkeepDialog {
    notice: UpkeepNotice,
}

impl UpkeepDialog {
    pub fn new(notice: UpkeepNotice) -> Self {
        UpkeepDialog { notice }
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
        x: inner.right().saturating_sub(OK_WIDTH),
        y: buttons_y(panel),
        width: OK_WIDTH,
        height: 1,
    }
}

impl Widget for UpkeepDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 16 || area.height < 8 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        draw_text(
            buf,
            inner(area).x + 1,
            inner(area).y,
            inner(area).right(),
            "Treasury Empty!",
            TEXT.add_modifier(Modifier::BOLD),
        );
        draw_text(
            buf,
            inner(area).x + 1,
            inner(area).y + 1,
            inner(area).right(),
            &format!("Upkeep of {} gold could not be paid.", self.notice.upkeep),
            TEXT,
        );
        let second = match &self.notice.disbanded {
            Some(name) => format!("A {name} has been disbanded."),
            None => "There was nothing left to disband.".to_string(),
        };
        draw_text(
            buf,
            inner(area).x + 1,
            inner(area).y + 2,
            inner(area).right(),
            &second,
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

    fn render(notice: UpkeepNotice) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(UpkeepDialog::new(notice), panel()))
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
    fn the_dialog_reports_the_upkeep_and_the_disbanded_unit() {
        let buf = render(UpkeepNotice {
            upkeep: 4,
            disbanded: Some("Settler".to_string()),
        });
        let panel = panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 40);
        assert!(title.contains("Treasury Empty!"), "title: {title:?}");
        let owed = row_text(&buf, panel.x + 2, panel.y + 2, 44);
        assert!(owed.contains("4 gold"), "owed: {owed:?}");
        let gone = row_text(&buf, panel.x + 2, panel.y + 3, 44);
        assert!(gone.contains("Settler"), "disbanded: {gone:?}");
        assert!(gone.contains("disbanded"), "disbanded: {gone:?}");
    }

    #[test]
    fn the_dialog_reports_an_empty_treasury_when_nothing_was_left_to_disband() {
        let buf = render(UpkeepNotice {
            upkeep: 2,
            disbanded: None,
        });
        let panel = panel();
        let gone = row_text(&buf, panel.x + 2, panel.y + 3, 44);
        assert!(gone.contains("nothing left"), "disbanded: {gone:?}");
    }

    #[test]
    fn the_ok_button_is_drawn_and_highlighted() {
        let buf = render(UpkeepNotice {
            upkeep: 1,
            disbanded: None,
        });
        let panel = panel();
        let ok = ok_button_rect(panel);
        assert_eq!(
            row_text(&buf, ok.x + 1, ok.y, OK_TEXT.len() as u16),
            OK_TEXT
        );
        assert_eq!(buf.cell((ok.x, ok.y)).unwrap().bg, Color::White);
    }
}
