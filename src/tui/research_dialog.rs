use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameView;
use crate::model::advancements::Advancement;

/// The vanilla-yellow backdrop shared with the city window.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 46;
const IDEAL_HEIGHT: u16 = 16;

const OK_TEXT: &str = "OK";
const OK_WIDTH: u16 = 4;

/// The rows below the header where the researchable advancements are listed.
const HEADER_ROWS: u16 = 3;

/// A modal window over the whole screen shown when an advancement completes at
/// the start of a turn. It congratulates the civilization and lists the
/// advancements it may research next; the player must pick one and hit OK.
pub struct ResearchDialog<'a> {
    view: &'a dyn GameView,
    discovered: Advancement,
    choices: Vec<Advancement>,
    cursor: usize,
    scroll: usize,
}

impl<'a> ResearchDialog<'a> {
    pub fn new(
        view: &'a dyn GameView,
        discovered: Advancement,
        choices: Vec<Advancement>,
        cursor: usize,
        scroll: usize,
    ) -> Self {
        ResearchDialog {
            view,
            discovered,
            choices,
            cursor,
            scroll,
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

/// The y row the OK button sits on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    panel.y + panel.height - 2
}

/// The rectangle of the OK button, right-aligned on the button row.
pub fn ok_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right() - OK_WIDTH,
        y: buttons_y(panel),
        width: OK_WIDTH,
        height: 1,
    }
}

/// The rows region holding the researchable advancement list.
pub fn list_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let top = inner.y + HEADER_ROWS;
    let height = buttons_y(panel).saturating_sub(top);
    Rect {
        x: inner.x + 1,
        y: top,
        width: inner.width.saturating_sub(2),
        height,
    }
}

impl Widget for ResearchDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 8 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(buf, inner.x + 1, inner.y, "Advancement Achieved!", BOLD);
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 1,
            &format!(
                "The {:?} discover {:?}!",
                self.view.current_player(),
                self.discovered
            ),
            BOLD,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            "Choose what to research next:",
            TEXT,
        );

        let rows = list_rect(area);
        if self.choices.is_empty() {
            if rows.y < rows.bottom() {
                draw_text(buf, rows.x, rows.y, "(nothing left to research)", TEXT);
            }
        } else {
            let visible = rows.height as usize;
            let offset = self.scroll.min(self.choices.len().saturating_sub(visible));
            for i in 0..visible {
                let row = rows.y + i as u16;
                if row >= rows.bottom() {
                    break;
                }
                let Some(choice) = self.choices.get(i + offset) else {
                    break;
                };
                let is_selected = self.cursor == i + offset;
                let style = if is_selected { SELECTED } else { TEXT };
                if is_selected {
                    fill_row(
                        buf,
                        Rect {
                            x: rows.x,
                            y: row,
                            width: rows.width,
                            height: 1,
                        },
                        SELECTED,
                    );
                }
                draw_text(
                    buf,
                    rows.x + 1,
                    row,
                    &format!("{:?} ({})", choice, choice.cost()),
                    style,
                );
            }
        }

        let ok = ok_button_rect(area);
        draw_text(buf, ok.x, ok.y, OK_TEXT, LINK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::Tile;
    use crate::model::cities::{City, CityId, ProductionTarget};
    use crate::model::civilizations::{Civilization, PlayerId};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        tile: Tile,
    }

    impl FakeView {
        fn english() -> Self {
            FakeView {
                tile: Tile::new(crate::model::geography::Terrain::Grassland),
            }
        }
    }

    impl GameView for FakeView {
        fn width(&self) -> usize {
            20
        }
        fn height(&self) -> usize {
            20
        }
        fn tile(&self, _x: usize, _y: usize) -> &Tile {
            &self.tile
        }
        fn units_at(&self, _x: usize, _y: usize) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn city_at(&self, _x: usize, _y: usize) -> Option<&City> {
            None
        }
        fn player_units(&self) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn player_cities(&self) -> Vec<&City> {
            Vec::new()
        }
        fn city(&self, _id: CityId) -> Option<&City> {
            None
        }
        fn current_player_id(&self) -> PlayerId {
            PlayerId::new(0)
        }
        fn city_income(&self, _id: CityId) -> crate::game_engine::game_view::CityIncome {
            crate::game_engine::game_view::CityIncome {
                food: 0,
                resources: 0,
                trade: 0,
                gold: 0,
                research: 0,
                special_resources: Vec::new(),
            }
        }
        fn home_units(&self, _city: CityId) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn explored(&self, _x: usize, _y: usize) -> bool {
            true
        }
        fn current_player(&self) -> Civilization {
            Civilization::English
        }
        fn civilization_of(&self, _player: PlayerId) -> Civilization {
            Civilization::English
        }
        fn turn(&self) -> u32 {
            1
        }
        fn year(&self) -> i32 {
            4000
        }
        fn gold(&self) -> u32 {
            0
        }
        fn advancement_in_progress(&self) -> Option<crate::model::advancements::Advancement> {
            None
        }
        fn research_progress(&self) -> u32 {
            0
        }
        fn research_cost(&self) -> Option<u32> {
            None
        }
        fn research_income(&self) -> u32 {
            0
        }
        fn production_choices(&self, _city: CityId) -> Vec<ProductionTarget> {
            Vec::new()
        }
    }

    const CHOICES: &[Advancement] = &[
        Advancement::Alphabet,
        Advancement::BronzeWorking,
        Advancement::Pottery,
    ];

    fn render(cursor: usize, scroll: usize) -> Buffer {
        let view = FakeView::english();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    ResearchDialog::new(
                        &view,
                        Advancement::Masonry,
                        CHOICES.to_vec(),
                        cursor,
                        scroll,
                    ),
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
        assert_eq!(panel.width, 46);
        assert_eq!(panel.height, 16);
        assert_eq!(panel.x, (100 - 46) / 2);
        assert_eq!(panel.y, (40 - 16) / 2);
    }

    #[test]
    fn render_shows_the_congratulations_and_ok_button() {
        let buf = render(0, 0);
        let panel = panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 30);
        assert!(title.contains("Advancement Achieved!"), "title: {title:?}");
        let line = row_text(&buf, panel.x + 2, panel.y + 2, 40);
        assert!(line.contains("English"), "line: {line:?}");
        assert!(line.contains("Masonry"), "line: {line:?}");
        let ok = ok_button_rect(panel);
        let ok_text = row_text(&buf, ok.x, ok.y, OK_TEXT.len() as u16);
        assert_eq!(ok_text, "OK");
    }

    #[test]
    fn list_rows_show_the_researchable_advancements() {
        let buf = render(0, 0);
        let panel = panel();
        let rows = list_rect(panel);
        let first = row_text(&buf, rows.x + 1, rows.y, 24);
        assert!(first.contains("Alphabet"), "row: {first:?}");
        let second = row_text(&buf, rows.x + 1, rows.y + 1, 24);
        assert!(second.contains("BronzeWorking"), "row: {second:?}");
    }

    #[test]
    fn selected_row_is_highlighted_white() {
        let buf = render(1, 0);
        let panel = panel();
        let rows = list_rect(panel);
        let selected = buf.cell((rows.x, rows.y + 1)).unwrap();
        assert_eq!(selected.bg, Color::White);
        let unselected = buf.cell((rows.x, rows.y)).unwrap();
        assert_eq!(unselected.bg, VANILLA_BG);
    }

    #[test]
    fn scrolling_reveals_rows_below_the_view() {
        let buf = render(3, 1);
        let panel = panel();
        let rows = list_rect(panel);
        let first = row_text(&buf, rows.x + 1, rows.y, 24);
        assert!(first.contains("Alphabet"), "row: {first:?}");
    }
}
