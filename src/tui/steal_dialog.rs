use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::StealOutcome;
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

/// A modal window announcing the outcome of a diplomat's Steal Technology
/// action: which advance came away and from which rival, or the refusal — the
/// rival knew nothing the player could take, and the diplomat walked back out.
/// Either way it carries a single OK button: the player acknowledges and the
/// window closes.
pub struct StealDialog {
    outcome: StealOutcome,
}

impl StealDialog {
    pub fn new(outcome: StealOutcome) -> Self {
        StealDialog { outcome }
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

impl Widget for StealDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        let (title, lines): (&str, [String; 2]) = match &self.outcome {
            StealOutcome::Stolen { advancement, rival } => (
                "Technology Stolen",
                [
                    format!("The {rival} knew {advancement:?}."),
                    "The secret is ours to keep.".to_string(),
                ],
            ),
            StealOutcome::NothingToSteal { rival } => (
                "Nothing to Steal",
                [
                    format!("The {rival} have nothing worth taking."),
                    "The diplomat walks back out.".to_string(),
                ],
            ),
        };
        draw_text(buf, inner.x + 1, inner.y, inner.right(), title, BOLD);
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 1,
            inner.right(),
            &lines[0],
            TEXT,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            inner.right(),
            &lines[1],
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

    use crate::game_engine::StealOutcome;
    use crate::model::advancements::Advancement;

    fn render(outcome: StealOutcome) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(StealDialog::new(outcome), panel()))
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
    fn the_dialog_names_the_stolen_advance_and_its_rival() {
        let buf = render(StealOutcome::Stolen {
            advancement: Advancement::BronzeWorking,
            rival: "Zulu".to_string(),
        });
        let panel = panel();
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("BronzeWorking"), "line: {line:?}");
        assert!(line.contains("Zulu"), "line: {line:?}");
        assert_eq!(
            row_text(
                &buf,
                panel.x + 2,
                panel.y + 1,
                "Technology Stolen".len() as u16
            ),
            "Technology Stolen"
        );
    }

    #[test]
    fn the_refusal_window_reports_that_and_the_walk_back_out() {
        let buf = render(StealOutcome::NothingToSteal {
            rival: "Babylonian".to_string(),
        });
        let panel = panel();
        assert_eq!(
            row_text(
                &buf,
                panel.x + 2,
                panel.y + 1,
                "Nothing to Steal".len() as u16
            ),
            "Nothing to Steal"
        );
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("Babylonian"), "line: {line:?}");
        assert!(
            row_text(&buf, panel.x + 2, panel.y + 3, 30).contains("walks back out"),
            "third line: {:?}",
            row_text(&buf, panel.x + 2, panel.y + 3, 30)
        );
    }

    #[test]
    fn the_ok_button_is_drawn_and_highlighted() {
        let buf = render(StealOutcome::Stolen {
            advancement: Advancement::Mathematics,
            rival: "Babylonian".to_string(),
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
