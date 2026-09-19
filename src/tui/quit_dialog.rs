use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 42;
const IDEAL_HEIGHT: u16 = 9;

const CONTINUE_TEXT: &str = "CONTINUE";
const CONTINUE_WIDTH: u16 = CONTINUE_TEXT.len() as u16 + 2;
const SAVE_TEXT: &str = "SAVE";
const SAVE_WIDTH: u16 = SAVE_TEXT.len() as u16 + 2;
const QUIT_TEXT: &str = "QUIT";
const QUIT_WIDTH: u16 = QUIT_TEXT.len() as u16 + 2;

/// The three answers the quit window may give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuitChoice {
    Continue,
    Save,
    Quit,
}

impl QuitChoice {
    pub(crate) fn next(self) -> Self {
        match self {
            QuitChoice::Continue => QuitChoice::Save,
            QuitChoice::Save => QuitChoice::Quit,
            QuitChoice::Quit => QuitChoice::Continue,
        }
    }

    pub(crate) fn prev(self) -> Self {
        match self {
            QuitChoice::Continue => QuitChoice::Quit,
            QuitChoice::Save => QuitChoice::Continue,
            QuitChoice::Quit => QuitChoice::Save,
        }
    }
}

/// A modal window asking what to do with the game when the player presses q:
/// continue playing, save first, or quit the process without saving.
pub struct QuitDialog {
    choice: QuitChoice,
}

impl QuitDialog {
    pub fn new(choice: QuitChoice) -> Self {
        QuitDialog { choice }
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

/// The y row the buttons sit on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    panel.y + panel.height - 2
}

fn button_rect(panel: Rect, width: u16, trailing_gap: u16) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right().saturating_sub(width + trailing_gap),
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

/// The rectangle of the QUIT button, right-aligned on the button row.
pub fn quit_button_rect(panel: Rect) -> Rect {
    button_rect(panel, QUIT_WIDTH, 1)
}

/// The rectangle of the SAVE button, just left of QUIT.
pub fn save_button_rect(panel: Rect) -> Rect {
    button_rect(panel, SAVE_WIDTH, QUIT_WIDTH + 1)
}

/// The rectangle of the CONTINUE button, just left of SAVE.
pub fn continue_button_rect(panel: Rect) -> Rect {
    button_rect(panel, CONTINUE_WIDTH, QUIT_WIDTH + SAVE_WIDTH + 2)
}

impl Widget for QuitDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(buf, inner.x + 1, inner.y, "Quit game", BOLD);
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 1,
            "Continue playing, save the game,",
            TEXT,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            "or quit without saving?",
            TEXT,
        );

        let continue_ = continue_button_rect(area);
        let save = save_button_rect(area);
        let quit = quit_button_rect(area);
        match self.choice {
            QuitChoice::Continue => fill_row(buf, continue_, SELECTED),
            QuitChoice::Save => fill_row(buf, save, SELECTED),
            QuitChoice::Quit => fill_row(buf, quit, SELECTED),
        }
        draw_text(buf, continue_.x + 1, continue_.y, CONTINUE_TEXT, LINK);
        draw_text(buf, save.x + 1, save.y, SAVE_TEXT, LINK);
        draw_text(buf, quit.x + 1, quit.y, QUIT_TEXT, LINK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(choice: QuitChoice) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(QuitDialog::new(choice), panel()))
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
    fn choice_cycles_forwards_and_backwards() {
        let mut choice = QuitChoice::Continue;
        assert_eq!(choice.next(), QuitChoice::Save);
        choice = choice.next();
        assert_eq!(choice, QuitChoice::Save);
        choice = choice.next();
        assert_eq!(choice, QuitChoice::Quit);
        choice = choice.next();
        assert_eq!(choice, QuitChoice::Continue);
        choice = choice.prev();
        assert_eq!(choice, QuitChoice::Quit);
        choice = choice.prev();
        assert_eq!(choice, QuitChoice::Save);
        choice = choice.prev();
        assert_eq!(choice, QuitChoice::Continue);
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
    fn all_three_buttons_are_drawn() {
        let buf = render(QuitChoice::Continue);
        let panel = panel();
        let continue_ = continue_button_rect(panel);
        let save = save_button_rect(panel);
        let quit = quit_button_rect(panel);
        assert_eq!(
            row_text(
                &buf,
                continue_.x + 1,
                continue_.y,
                CONTINUE_TEXT.len() as u16
            ),
            CONTINUE_TEXT
        );
        assert_eq!(
            row_text(&buf, save.x + 1, save.y, SAVE_TEXT.len() as u16),
            SAVE_TEXT
        );
        assert_eq!(
            row_text(&buf, quit.x + 1, quit.y, QUIT_TEXT.len() as u16),
            QUIT_TEXT
        );
        // Buttons sit on the dialog's bottom row, left to right.
        assert!(continue_.x < save.x);
        assert!(save.x < quit.x);
        assert_eq!(continue_.y, quit.y);
    }

    #[test]
    fn the_selected_button_is_highlighted_white() {
        let buf = render(QuitChoice::Continue);
        let panel = panel();
        assert_eq!(
            buf.cell((continue_button_rect(panel).x, continue_button_rect(panel).y))
                .unwrap()
                .bg,
            Color::White
        );
        assert_eq!(
            buf.cell((save_button_rect(panel).x, save_button_rect(panel).y))
                .unwrap()
                .bg,
            VANILLA_BG
        );
        assert_eq!(
            buf.cell((quit_button_rect(panel).x, quit_button_rect(panel).y))
                .unwrap()
                .bg,
            VANILLA_BG
        );
        // A different choice highlights its own button.
        let buf = render(QuitChoice::Quit);
        assert_eq!(
            buf.cell((quit_button_rect(panel).x, quit_button_rect(panel).y))
                .unwrap()
                .bg,
            Color::White
        );
    }
}
