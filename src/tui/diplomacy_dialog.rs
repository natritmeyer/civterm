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
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 42;
const IDEAL_HEIGHT: u16 = 9;

const WAR_TEXT: &str = "WAR";
const WAR_WIDTH: u16 = WAR_TEXT.len() as u16 + 2;
const PEACE_TEXT: &str = "PEACE";
const PEACE_WIDTH: u16 = PEACE_TEXT.len() as u16 + 2;

/// Why the war-or-peace window opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiplomacyOrigin {
    /// Two civilizations met for the first time.
    Contact,
    /// A unit tried to step onto a foreign tile while at peace.
    Movement,
}

/// The two answers the window may give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiplomacyChoice {
    War,
    Peace,
}

/// A modal window asking whether to declare war or remain at peace, shown
/// when a civilization is first encountered or when a unit tries to enter
/// foreign land while the two sides are at peace.
pub struct DiplomacyDialog {
    opponent: Civilization,
    origin: DiplomacyOrigin,
    choice: DiplomacyChoice,
}

impl DiplomacyDialog {
    pub fn new(opponent: Civilization, origin: DiplomacyOrigin, choice: DiplomacyChoice) -> Self {
        DiplomacyDialog {
            opponent,
            origin,
            choice,
        }
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

/// The rectangle of the PEACE button, right-aligned on the button row.
pub fn peace_button_rect(panel: Rect) -> Rect {
    button_rect(panel, PEACE_WIDTH, 1)
}

/// The rectangle of the WAR button, just left of PEACE.
pub fn war_button_rect(panel: Rect) -> Rect {
    button_rect(panel, WAR_WIDTH, PEACE_WIDTH + 1)
}

impl Widget for DiplomacyDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(buf, inner.x + 1, inner.y, "Diplomacy", BOLD);
        match self.origin {
            DiplomacyOrigin::Contact => {
                draw_text(
                    buf,
                    inner.x + 1,
                    inner.y + 1,
                    &format!("The {:?} have been discovered!", self.opponent),
                    BOLD,
                );
                draw_text(
                    buf,
                    inner.x + 1,
                    inner.y + 2,
                    "We may declare war or make peace with them.",
                    TEXT,
                );
            }
            DiplomacyOrigin::Movement => {
                draw_text(
                    buf,
                    inner.x + 1,
                    inner.y + 1,
                    &format!("Your unit may not enter {:?} land.", self.opponent),
                    BOLD,
                );
                draw_text(
                    buf,
                    inner.x + 1,
                    inner.y + 2,
                    "Declare war to attack, or remain at peace.",
                    TEXT,
                );
            }
        }

        let war = war_button_rect(area);
        let peace = peace_button_rect(area);
        let (war_selected, peace_selected) = match self.choice {
            DiplomacyChoice::War => (true, false),
            DiplomacyChoice::Peace => (false, true),
        };
        if war_selected {
            fill_row(buf, war, SELECTED);
        }
        if peace_selected {
            fill_row(buf, peace, SELECTED);
        }
        draw_text(buf, war.x + 1, war.y, WAR_TEXT, LINK);
        draw_text(buf, peace.x + 1, peace.y, PEACE_TEXT, LINK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(origin: DiplomacyOrigin, choice: DiplomacyChoice) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    DiplomacyDialog::new(Civilization::Zulu, origin, choice),
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
    fn contact_origin_names_the_new_civilization() {
        let buf = render(DiplomacyOrigin::Contact, DiplomacyChoice::Peace);
        let panel = panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 12);
        assert!(title.contains("Diplomacy"), "title: {title:?}");
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("Zulu"), "line: {line:?}");
        let hint = row_text(&buf, panel.x + 2, panel.y + 3, 44);
        assert!(hint.contains("declare war or make peace"), "hint: {hint:?}");
    }

    #[test]
    fn movement_origin_explains_the_blocked_step() {
        let buf = render(DiplomacyOrigin::Movement, DiplomacyChoice::War);
        let panel = panel();
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("Zulu"), "line: {line:?}");
        let hint = row_text(&buf, panel.x + 2, panel.y + 3, 44);
        assert!(hint.contains("Declare war to attack"), "hint: {hint:?}");
    }

    #[test]
    fn both_buttons_are_drawn() {
        let buf = render(DiplomacyOrigin::Contact, DiplomacyChoice::Peace);
        let panel = panel();
        let war = war_button_rect(panel);
        let peace = peace_button_rect(panel);
        assert_eq!(
            row_text(&buf, war.x + 1, war.y, WAR_TEXT.len() as u16),
            WAR_TEXT
        );
        assert_eq!(
            row_text(&buf, peace.x + 1, peace.y, PEACE_TEXT.len() as u16),
            PEACE_TEXT
        );
    }

    #[test]
    fn the_selected_button_is_highlighted_white() {
        let buf = render(DiplomacyOrigin::Contact, DiplomacyChoice::War);
        let panel = panel();
        let war = war_button_rect(panel);
        let peace = peace_button_rect(panel);
        assert_eq!(buf.cell((war.x, war.y)).unwrap().bg, Color::White);
        assert_eq!(buf.cell((peace.x, peace.y)).unwrap().bg, VANILLA_BG);
    }
}
