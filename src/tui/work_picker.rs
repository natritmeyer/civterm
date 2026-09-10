use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameView;
use crate::model::cartography::Tile;
use crate::model::geography::TerrainImprovement;
use crate::model::units::UnitId;

/// The vanilla-yellow backdrop shared with the city window.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

/// The floated panel's ideal dimensions before clamping to the window.
const PICKER_WIDTH: u16 = 40;
const PICKER_HEIGHT: u16 = 18;

/// The number of picker rows the panel shows at its ideal size.
pub const MAX_VISIBLE_ROWS: u16 = 12;

/// The improvements a settler can build on `tile` right now, in a stable
/// order: only those the terrain supports and which are not yet in place.
pub fn buildable_improvements(tile: &Tile) -> Vec<TerrainImprovement> {
    [
        TerrainImprovement::Irrigation,
        TerrainImprovement::Mine,
        TerrainImprovement::Road,
    ]
    .into_iter()
    .filter(|improvement| {
        let already = match improvement {
            TerrainImprovement::Irrigation => tile.is_irrigated(),
            TerrainImprovement::Mine => tile.is_mined(),
            TerrainImprovement::Road => tile.has_road(),
        };
        !already && tile.terrain.supports(*improvement)
    })
    .collect()
}

pub struct WorkPicker<'a> {
    view: &'a dyn GameView,
    unit: UnitId,
    cursor: usize,
    scroll: usize,
}

impl<'a> WorkPicker<'a> {
    /// A floating panel over the map listing the improvements the selected
    /// settler can build on the tile it stands on.
    pub fn new(view: &'a dyn GameView, unit: UnitId, cursor: usize, scroll: usize) -> Self {
        WorkPicker {
            view,
            unit,
            cursor,
            scroll,
        }
    }

    /// The tile beneath the unit, if the unit still exists.
    fn tile(&self) -> Option<&Tile> {
        self.view
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == self.unit)
            .map(|unit| {
                self.view
                    .tile(unit.location.x as usize, unit.location.y as usize)
            })
    }

    fn draw_buttons(buf: &mut Buffer, panel: Rect) {
        let cancel = cancel_button_rect(panel);
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

/// The rectangle the picker panel occupies over `area`, centred across it.
pub fn work_picker_rect(area: Rect) -> Rect {
    let width = (PICKER_WIDTH.min(area.width.saturating_sub(2)).max(24)) & !1;
    let height = (PICKER_HEIGHT.min(area.height.saturating_sub(2)).max(8)) & !1;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
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

/// The rows rectangle: the title and terrain subtitle sit above it, the
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
        width: 4,
        height: 1,
    }
}

impl<'a> Widget for WorkPicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 16 || area.height < 8 {
            return;
        }
        let Some(tile) = self.tile() else {
            return;
        };
        if tile.terrain.is_water() {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let header = "Settler — work";
        draw_text(buf, inner(area).x + 1, inner(area).y, header, BOLD);
        draw_text(
            buf,
            inner(area).x + 1,
            inner(area).y + 1,
            tile.terrain.name(),
            LINK,
        );

        let rows = rows_rect(area);
        let improvements = buildable_improvements(tile);
        if improvements.is_empty() {
            draw_text(buf, rows.x + 1, rows.y, "(nothing to build here)", BOLD);
            Self::draw_buttons(buf, area);
            return;
        }
        let visible = rows.height as usize;
        let offset = self.scroll.min(improvements.len().saturating_sub(visible));
        for i in 0..visible {
            let row = rows.y + i as u16;
            if row >= rows.bottom() {
                break;
            }
            let Some(improvement) = improvements.get(i + offset) else {
                break;
            };
            let is_selected = self.cursor == i + offset;
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
            let text = format!("{} ({})", improvement.name(), improvement.work_turns());
            draw_text(
                buf,
                rows.x + 2,
                row,
                &text,
                if is_selected { SELECTED } else { TEXT },
            );
        }
        Self::draw_buttons(buf, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::Location;
    use crate::model::cities::{City, CityId, ProductionTarget};
    use crate::model::civilizations::{Civilization, PlayerId};
    use crate::model::geography::Terrain;
    use crate::model::units::{Unit, UnitClass};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        tile: Tile,
        unit: Unit,
    }

    impl FakeView {
        fn new(terrain: Terrain) -> Self {
            Self::with_tile(Tile::new(terrain))
        }

        fn with_tile(tile: Tile) -> Self {
            FakeView {
                tile,
                unit: Unit::new(
                    UnitClass::Settler,
                    Location::new(3, 3),
                    PlayerId::new(0),
                    CityId::new(0),
                    UnitId::new(0),
                ),
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
        fn units_at(&self, _x: usize, _y: usize) -> Vec<&Unit> {
            Vec::new()
        }
        fn city_at(&self, _x: usize, _y: usize) -> Option<&City> {
            None
        }
        fn player_units(&self) -> Vec<&Unit> {
            vec![&self.unit]
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
        fn home_units(&self, _city: CityId) -> Vec<&Unit> {
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

    fn picker_panel() -> Rect {
        work_picker_rect(Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 40,
        })
    }

    fn render(terrain: Terrain, cursor: usize, scroll: usize) -> Buffer {
        let view = FakeView::new(terrain);
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    WorkPicker::new(&view, UnitId::new(0), cursor, scroll),
                    picker_panel(),
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_tile(tile: Tile, cursor: usize, scroll: usize) -> Buffer {
        let view = FakeView::with_tile(tile);
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    WorkPicker::new(&view, UnitId::new(0), cursor, scroll),
                    picker_panel(),
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn buildable_improvements_respect_support_and_existing_work() {
        let mut grassland = Tile::new(Terrain::Grassland);
        assert_eq!(
            buildable_improvements(&grassland),
            vec![TerrainImprovement::Irrigation, TerrainImprovement::Road]
        );
        grassland.irrigate().unwrap();
        grassland.build_road().unwrap();
        assert!(buildable_improvements(&grassland).is_empty());

        let mut desert = Tile::new(Terrain::Desert);
        assert_eq!(
            buildable_improvements(&desert),
            vec![
                TerrainImprovement::Irrigation,
                TerrainImprovement::Mine,
                TerrainImprovement::Road
            ]
        );
        desert.mine().unwrap();
        assert_eq!(
            buildable_improvements(&desert),
            vec![TerrainImprovement::Irrigation, TerrainImprovement::Road]
        );

        let forest = Tile::new(Terrain::Forest);
        assert_eq!(
            buildable_improvements(&forest),
            vec![TerrainImprovement::Road]
        );

        let ocean = Tile::new(Terrain::Ocean);
        assert!(buildable_improvements(&ocean).is_empty());
    }

    #[test]
    fn panel_shows_the_title_subtitle_and_buildable_rows() {
        let buf = render(Terrain::Grassland, 0, 0);
        let panel = picker_panel();
        let title = row_text(&buf, panel.x + 2, panel.y + 1, 30);
        assert!(title.contains("Settler — work"), "title: {title:?}");
        let subtitle = row_text(&buf, panel.x + 2, panel.y + 2, 30);
        assert!(subtitle.contains("Grassland"), "subtitle: {subtitle:?}");
        let rows = rows_rect(panel);
        let first_row: String = (0..20)
            .map(|i| buf.cell((rows.x + i, rows.y)).unwrap().symbol())
            .collect();
        assert_eq!(first_row.trim(), "Irrigation (2)", "row was {first_row:?}");
        let second_row: String = (0..20)
            .map(|i| buf.cell((rows.x + i, rows.y + 1)).unwrap().symbol())
            .collect();
        assert_eq!(second_row.trim(), "Road (2)", "row was {second_row:?}");
    }

    #[test]
    fn selected_row_is_highlighted_white() {
        let buf = render(Terrain::Grassland, 1, 0);
        let panel = picker_panel();
        let rows = rows_rect(panel);
        let selected = buf.cell((rows.x + 1, rows.y + 1)).unwrap();
        assert_eq!(selected.bg, Color::White);
        let unselected = buf.cell((rows.x + 1, rows.y)).unwrap();
        assert_eq!(unselected.bg, VANILLA_BG);
    }

    #[test]
    fn a_fully_improved_tile_offers_no_rows() {
        let mut grassland = Tile::new(Terrain::Grassland);
        grassland.irrigate().unwrap();
        grassland.build_road().unwrap();
        let buf = render_tile(grassland, 0, 0);
        let panel = picker_panel();
        let rows = rows_rect(panel);
        let message: String = (0..28)
            .map(|i| buf.cell((rows.x + i, rows.y)).unwrap().symbol())
            .collect();
        assert!(
            message.trim().contains("nothing to build"),
            "message: {message:?}"
        );
    }

    #[test]
    fn rows_show_the_number_of_turns_each_build_takes() {
        let buf = render(Terrain::Desert, 0, 0);
        let rows = rows_rect(picker_panel());
        let mine: String = (0..20)
            .map(|i| buf.cell((rows.x + i, rows.y + 1)).unwrap().symbol())
            .collect();
        assert_eq!(mine.trim(), "Mine (3)", "row was {mine:?}");
    }

    #[test]
    fn scroll_is_clamped_when_the_list_fits() {
        // A desert offers three improvements; they all fit, so any scroll is
        // clamped and the first offered row stays first.
        for scroll in [0, 1, 100] {
            let buf = render(Terrain::Desert, 0, scroll);
            let panel = picker_panel();
            let rows = rows_rect(panel);
            let first_row: String = (0..20)
                .map(|i| buf.cell((rows.x + i, rows.y)).unwrap().symbol())
                .collect();
            assert_eq!(
                first_row.trim(),
                "Irrigation (2)",
                "scroll={scroll}: {first_row:?}"
            );
        }
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
}
