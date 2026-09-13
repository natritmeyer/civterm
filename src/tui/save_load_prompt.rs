use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const ERROR: Style = Style::new().fg(Color::Rgb(220, 90, 90)).bg(VANILLA_BG);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 60;
const IDEAL_HEIGHT: u16 = 9;

/// What the prompt is asking for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveLoadKind {
    Save,
    Load,
}

impl SaveLoadKind {
    fn title(self) -> &'static str {
        match self {
            SaveLoadKind::Save => "Save game",
            SaveLoadKind::Load => "Load game",
        }
    }

    fn prompt(self) -> &'static str {
        match self {
            SaveLoadKind::Save => "Save to",
            SaveLoadKind::Load => "Load from",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            SaveLoadKind::Save => "Enter saves  •  Esc cancels",
            SaveLoadKind::Load => "Enter loads  •  Esc cancels",
        }
    }
}

/// A modal text-entry prompt asking for a save path when the player presses
/// S in play, or a load path when they choose "Load Saved Game" from the menu.
pub struct SaveLoadPrompt {
    kind: SaveLoadKind,
    input: String,
    error: Option<String>,
}

impl SaveLoadPrompt {
    pub fn new(kind: SaveLoadKind, input: String, error: Option<String>) -> Self {
        SaveLoadPrompt { kind, input, error }
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

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
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

impl Widget for SaveLoadPrompt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 24 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        let x = inner.x + 1;
        draw_text(buf, x, inner.y, self.kind.title(), BOLD);

        let prompt = self.kind.prompt();
        let input_x = x + self.kind.prompt().len() as u16 + 2;
        draw_text(buf, x, inner.y + 1, &format!("{prompt}:"), TEXT);
        // The typed path, drawn right after the label and clipped to the
        // window, with a block cursor so the entry point stays visible.
        let mut cx = input_x;
        for ch in self.input.chars() {
            let width = cell_width(ch);
            if cx.saturating_add(width) > inner.right().saturating_sub(1) {
                break;
            }
            if let Some(cell) = buf.cell_mut((cx, inner.y + 1)) {
                cell.set_symbol(&ch.to_string());
                cell.set_style(BOLD);
                if width == 2
                    && let Some(next) = buf.cell_mut((cx.saturating_add(1), inner.y + 1))
                {
                    next.set_diff_option(ratatui::buffer::CellDiffOption::Skip);
                }
            }
            cx = cx.saturating_add(width);
        }
        if cx < inner.right().saturating_sub(1) {
            let style = BOLD.add_modifier(Modifier::REVERSED);
            set_cell(buf, cx, inner.y + 1, " ", style);
        }

        draw_text(buf, x, inner.y + 2, self.kind.hint(), TEXT);
        if let Some(error) = &self.error {
            fill_row(
                buf,
                Rect {
                    x: inner.x,
                    y: inner.y + 3,
                    width: inner.width,
                    height: 1,
                },
                ERROR,
            );
            draw_text(buf, x, inner.y + 3, error, ERROR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(kind: SaveLoadKind, input: &str, error: Option<&str>) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    SaveLoadPrompt::new(kind, input.to_string(), error.map(String::from)),
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
        assert_eq!(panel.width, 60);
        assert_eq!(panel.height, 8);
        assert_eq!(panel.x, (100 - 60) / 2);
        assert_eq!(panel.y, (40 - 8) / 2);
    }

    #[test]
    fn save_kind_names_the_action_and_shows_the_path() {
        let buf = render(SaveLoadKind::Save, "civterm.civ", None);
        let panel = panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 12);
        assert!(title.contains("Save game"), "title: {title:?}");
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 44);
        assert!(line.contains("Save to:"), "line: {line:?}");
        assert!(line.contains("civterm.civ"), "line: {line:?}");
        let hint = row_text(&buf, panel.x + 2, panel.y + 3, 40);
        assert!(hint.contains("Enter saves"), "hint: {hint:?}");
    }

    #[test]
    fn load_kind_names_the_action_and_asks_for_the_source() {
        let buf = render(SaveLoadKind::Load, "/tmp/game.civ", None);
        let panel = panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 12);
        assert!(title.contains("Load game"), "title: {title:?}");
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 44);
        assert!(line.contains("Load from:"), "line: {line:?}");
        assert!(line.contains("/tmp/game.civ"), "line: {line:?}");
        let hint = row_text(&buf, panel.x + 2, panel.y + 3, 40);
        assert!(hint.contains("Enter loads"), "hint: {hint:?}");
    }

    #[test]
    fn an_error_is_shown_on_its_own_row() {
        let buf = render(SaveLoadKind::Load, "x.civ", Some("boom"));
        let panel = panel();
        let error = row_text(&buf, panel.x + 2, panel.y + 4, 20);
        assert!(error.contains("boom"), "error: {error:?}");
        assert_eq!(buf.cell((panel.x + 2, panel.y + 4)).unwrap().bg, VANILLA_BG);
        assert_eq!(
            buf.cell((panel.x + 2, panel.y + 4)).unwrap().fg,
            Color::Rgb(220, 90, 90)
        );
    }
}
