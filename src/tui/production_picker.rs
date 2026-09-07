use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameView;
use crate::model::cities::{CityId, ProductionTarget};

/// The vanilla-yellow backdrop shared with the city window.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

/// The floated panel's ideal dimensions before clamping to the window.
const PICKER_WIDTH: u16 = 52;
const PICKER_HEIGHT: u16 = 20;

/// The number of picker rows the panel shows at its ideal size.
pub const MAX_VISIBLE_ROWS: u16 = 14;

pub struct ProductionPicker<'a> {
    view: &'a dyn GameView,
    city_id: CityId,
    cursor_col: usize,
    cursor_row: usize,
    scroll: usize,
}

impl<'a> ProductionPicker<'a> {
    /// A floating panel over the city window listing everything the current
    /// player can build. `cursor_col`/`cursor_row` name the currently selected
    /// item (units column 0, improvements column 1); `scroll` is the shared
    /// vertical offset of the two lists.
    pub fn new(
        view: &'a dyn GameView,
        city_id: CityId,
        cursor_col: usize,
        cursor_row: usize,
        scroll: usize,
    ) -> Self {
        ProductionPicker {
            view,
            city_id,
            cursor_col,
            cursor_row,
            scroll,
        }
    }

    fn draw_rows(
        buf: &mut Buffer,
        rows: &[PickRow],
        column: Rect,
        scroll: usize,
        cursor_row: Option<usize>,
    ) {
        if rows.is_empty() {
            if column.y < column.bottom() {
                draw_text(buf, column.x + 1, column.y, "(none)", BOLD);
            }
            return;
        }
        let visible = column.height as usize;
        let offset = scroll.min(rows.len().saturating_sub(visible));
        for i in 0..visible {
            let row = column.y + i as u16;
            if row >= column.bottom() {
                break;
            }
            let Some(entry) = rows.get(i + offset) else {
                break;
            };
            let is_selected = cursor_row == Some(i + offset);
            let style = if is_selected { SELECTED } else { TEXT };
            if is_selected {
                fill_row(
                    buf,
                    Rect {
                        x: column.x,
                        y: row,
                        width: column.width,
                        height: 1,
                    },
                    SELECTED,
                );
            }
            let text = format!("{} ({})", entry.label, entry.target.resource_cost());
            draw_text(buf, column.x + 2, row, &text, style);
        }
    }

    fn draw_buttons(buf: &mut Buffer, panel: Rect) {
        let cancel = Rect {
            x: cancel_button_rect(panel).x,
            y: buttons_y(panel),
            width: 6,
            height: 1,
        };
        let save = save_button_rect(panel);
        draw_text(buf, cancel.x, cancel.y, "Cancel", LINK);
        draw_text(buf, save.x, save.y, "Save", LINK);
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
/// any wide glyphs. Returns the x the next glyph would land at.
fn draw_text(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) -> u16 {
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
    cx
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

/// The rectangle the picker panel occupies over `window`, centred across it.
pub fn picker_rect(window: Rect) -> Rect {
    let width = (PICKER_WIDTH.min(window.width.saturating_sub(2)).max(8)) & !1;
    let height = (PICKER_HEIGHT.min(window.height.saturating_sub(2)).max(8)) & !1;
    Rect {
        x: window.x + (window.width - width) / 2,
        y: window.y + (window.height - height) / 2,
        width,
        height,
    }
}

/// The interior of the panel, inside its border.
fn inner(panel: Rect) -> Rect {
    Rect {
        x: panel.x + 1,
        y: panel.y + 1,
        width: panel.width - 2,
        height: panel.height - 2,
    }
}

/// The shared rows rectangle: header and column titles sit above it, the
/// Cancel/Save buttons below it.
pub fn rows_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let height = inner.height.saturating_sub(4);
    Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height,
    }
}

/// The rectangles of the two lists: units on the left, improvements right.
pub fn column_rects(panel: Rect) -> (Rect, Rect) {
    let rows = rows_rect(panel);
    let half = rows.width / 2;
    let units = Rect {
        x: rows.x,
        y: rows.y,
        width: half.saturating_sub(1),
        height: rows.height,
    };
    let improvements = Rect {
        x: rows.x + half + 1,
        y: rows.y,
        width: rows.width - half - 1,
        height: rows.height,
    };
    (units, improvements)
}

/// The y row the Cancel/Save buttons sit on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    panel.y + panel.height - 2
}

pub fn cancel_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right() - 17,
        y: buttons_y(panel),
        width: 6,
        height: 1,
    }
}

pub fn save_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right() - 6,
        y: buttons_y(panel),
        width: 6,
        height: 1,
    }
}

impl<'a> Widget for ProductionPicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(city) = self.view.city(self.city_id) else {
            return;
        };
        if area.width < 16 || area.height < 8 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let rows = rows_rect(area);
        let (units_col, improvements_col) = column_rects(area);
        let header = format!("{} — production", city.name);
        draw_text(buf, inner(area).x + 1, inner(area).y, &header, BOLD);
        draw_text(buf, units_col.x + 1, inner(area).y + 1, "Units", BOLD);
        draw_text(
            buf,
            improvements_col.x + 1,
            inner(area).y + 1,
            "Improvements",
            BOLD,
        );

        let (units, improvements) = pick_rows(self.view, self.city_id);
        Self::draw_rows(
            buf,
            &units,
            units_col,
            self.scroll,
            (self.cursor_col == 0).then_some(self.cursor_row),
        );
        Self::draw_rows(
            buf,
            &improvements,
            improvements_col,
            self.scroll,
            (self.cursor_col == 1).then_some(self.cursor_row),
        );
        // Column separator, a single-cell black vertical rule.
        let sep_x = units_col.right();
        for y in rows.y..rows.bottom() {
            set_cell(buf, sep_x, y, "│", TEXT);
        }

        Self::draw_buttons(buf, area);
    }
}

/// One row of a picker list: the target and its plain display name (the
/// resource cost is appended when drawn).
pub struct PickRow {
    pub target: ProductionTarget,
    pub label: String,
}

/// Split the city's buildable targets into units and improvements, each
/// sorted by resource cost and then alphabetically by name.
pub fn pick_rows(view: &dyn GameView, city: CityId) -> (Vec<PickRow>, Vec<PickRow>) {
    let mut units: Vec<PickRow> = Vec::new();
    let mut improvements: Vec<PickRow> = Vec::new();
    for target in view.production_choices(city) {
        match target {
            ProductionTarget::Unit(unit_class) => units.push(PickRow {
                target,
                label: format!("{unit_class:?}"),
            }),
            ProductionTarget::Improvement(improvement) => improvements.push(PickRow {
                target,
                label: improvement.name().to_string(),
            }),
        }
    }
    units.sort_by(|a, b| {
        a.target
            .resource_cost()
            .cmp(&b.target.resource_cost())
            .then_with(|| a.label.cmp(&b.label))
    });
    improvements.sort_by(|a, b| {
        a.target
            .resource_cost()
            .cmp(&b.target.resource_cost())
            .then_with(|| a.label.cmp(&b.label))
    });
    (units, improvements)
}

/// The larger of the two list lengths, for clamping scroll offsets.
pub fn list_len(rows: &(Vec<PickRow>, Vec<PickRow>)) -> usize {
    rows.0.len().max(rows.1.len())
}

/// The target at `(column, row)` in the picker lists, if any.
pub fn target_at(
    rows: &(Vec<PickRow>, Vec<PickRow>),
    column: usize,
    row: usize,
) -> Option<ProductionTarget> {
    let list = if column == 0 { &rows.0 } else { &rows.1 };
    list.get(row).map(|entry| entry.target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::{Location, Tile};
    use crate::model::cities::{City, CityId, CityImprovement};
    use crate::model::civilizations::{Civilization, PlayerId};
    use crate::model::units::UnitClass;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        choices: Vec<ProductionTarget>,
        city: City,
        tile: Tile,
    }

    impl FakeView {
        fn english() -> Self {
            FakeView {
                choices: vec![
                    ProductionTarget::Unit(UnitClass::Settler),
                    ProductionTarget::Unit(UnitClass::Militia),
                    ProductionTarget::Unit(UnitClass::Phalanx),
                    ProductionTarget::Improvement(CityImprovement::Marketplace),
                    ProductionTarget::Improvement(CityImprovement::Barracks),
                ],
                city: City::new(
                    "London",
                    Location::new(3, 3),
                    PlayerId::new(0),
                    CityId::new(0),
                ),
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
            Some(&self.city)
        }
        fn player_units(&self) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn player_cities(&self) -> Vec<&City> {
            vec![&self.city]
        }
        fn city(&self, id: CityId) -> Option<&City> {
            (self.city.id() == id).then_some(&self.city)
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
            self.choices.clone()
        }
    }

    fn render(panel: Rect, col: usize, row: usize, scroll: usize) -> Buffer {
        let view = FakeView::english();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    ProductionPicker::new(&view, CityId::new(0), col, row, scroll),
                    panel,
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn picker_panel() -> Rect {
        picker_rect(Rect {
            x: 8,
            y: 5,
            width: 64,
            height: 30,
        })
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn units_sorted_by_cost_then_name() {
        let view = FakeView::english();
        let (units, _) = pick_rows(&view, CityId::new(0));
        let names: Vec<&str> = units.iter().map(|row| row.label.as_str()).collect();
        // Settler 60 is priciest; Militia 10 and Phalanx 15 follow cost order.
        assert_eq!(names, vec!["Militia", "Phalanx", "Settler"]);
    }

    #[test]
    fn improvements_sorted_by_cost_then_name() {
        let view = FakeView::english();
        let (_, improvements) = pick_rows(&view, CityId::new(0));
        let names: Vec<&str> = improvements.iter().map(|row| row.label.as_str()).collect();
        assert_eq!(names, vec!["Barracks", "Marketplace"]);
    }

    #[test]
    fn cost_order_ties_break_alphabetically() {
        let view = FakeView::english();
        let (units, _) = pick_rows(&view, CityId::new(0));
        // Militia(10) < Phalanx(15) < Settler(60).
        let costs: Vec<u32> = units.iter().map(|r| r.target.resource_cost()).collect();
        assert_eq!(costs, vec![10, 15, 60]);
    }

    #[test]
    fn panel_shows_headers_and_sorted_rows() {
        let buf = render(picker_panel(), 0, 0, 0);
        let panel = picker_panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 30);
        assert!(title.contains("London — production"), "title: {title:?}");
        let rows = rows_rect(panel);
        let units_col = column_rects(panel).0;
        let first_row: String = (0..16)
            .map(|i| buf.cell((units_col.x + i, rows.y)).unwrap().symbol())
            .collect();
        assert_eq!(first_row.trim(), "Militia (10)", "row was {first_row:?}");
    }

    #[test]
    fn selected_row_is_highlighted_white() {
        let buf = render(picker_panel(), 1, 0, 0);
        let panel = picker_panel();
        let rows = rows_rect(panel);
        let improvements_col = column_rects(panel).1;
        let selected = buf.cell((improvements_col.x + 1, rows.y)).unwrap();
        assert_eq!(selected.bg, Color::White);
        // The units column's first row is not selected, so it stays vanilla.
        let units_col = column_rects(panel).0;
        let cell = buf.cell((units_col.x + 1, rows.y)).unwrap();
        assert_eq!(cell.bg, VANILLA_BG);
    }

    #[test]
    fn cancel_and_save_sit_on_the_buttons_row() {
        let panel = picker_panel();
        let y = buttons_y(panel);
        assert_eq!(y, panel.y + panel.height - 2);
        assert_eq!(cancel_button_rect(panel).y, y);
        assert_eq!(save_button_rect(panel).y, y);
        assert!(save_button_rect(panel).x > cancel_button_rect(panel).right());
    }

    #[test]
    fn target_at_resolves_cursor_rows_in_both_lists() {
        let view = FakeView::english();
        let rows = pick_rows(&view, CityId::new(0));
        assert_eq!(
            target_at(&rows, 0, 1),
            Some(ProductionTarget::Unit(UnitClass::Phalanx))
        );
        assert_eq!(
            target_at(&rows, 1, 0),
            Some(ProductionTarget::Improvement(CityImprovement::Barracks))
        );
        assert_eq!(target_at(&rows, 0, 20), None);
        assert_eq!(target_at(&rows, 1, 20), None);
        assert_eq!(list_len(&rows), 3);
    }
}
