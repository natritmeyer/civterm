use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::game_screen::{TILE_WIDTH, paint_tile};
use crate::game_engine::GameView;
use crate::game_engine::game_view::CityIncome;
use crate::model::cities::{City, CityId, ProductionTarget};
use crate::model::geography::SpecialResource;

/// The vanilla-yellow backdrop of the city window.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const RULE: Style = Style::new().fg(BLACK).bg(VANILLA_BG);

/// Fixed-width emoji (single codepoint, two terminal cells each).
const FA_FOOD: char = '🌾';
const FA_SHIELD: char = '🪖';
const FA_TRADE: char = '🛒';
const FA_RESEARCH: char = '💡';
const FA_GOLD: char = '🪎';
const FA_POP: char = '🧍';

const CLOSE_TEXT: &str = "✕ Close";
const CLOSE_WIDTH: u16 = 7;

/// The dims of a standard city window before clamping to fit the screen.
const IDEAL_WIDTH: u16 = 64;
const IDEAL_HEIGHT: u16 = 30;

/// The bordered 5x5 mini-map in the middle band. Each world tile spans two
/// terminal cells (matching the main map), so the box is 12 wide and 7 tall.
const MINIMAP_WIDTH: u16 = 12;
const MINIMAP_HEIGHT: u16 = 7;

fn resource_icon(resource: SpecialResource) -> char {
    match resource {
        SpecialResource::Coal => '🪨',
        SpecialResource::Fish => '🐟',
        SpecialResource::Game => '🦌',
        SpecialResource::Gems => '💎',
        SpecialResource::Gold => '🪙',
        SpecialResource::Horses => '🐎',
        SpecialResource::Oasis => '🌴',
        SpecialResource::Oil => '⛽',
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
                // Mark the trailing half of a wide glyph so the terminal
                // doesn't reprint it as a separate column.
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

/// Draw a single-cell black box border around `rect` in the same character
/// style as the event log's border.
fn draw_border(buf: &mut Buffer, rect: Rect) {
    let x0 = rect.x;
    let x1 = rect.right() - 1;
    let y0 = rect.y;
    let y1 = rect.bottom() - 1;
    for x in x0..=x1 {
        set_cell(buf, x, y0, "─", RULE);
        set_cell(buf, x, y1, "─", RULE);
    }
    for y in y0..=y1 {
        set_cell(buf, x0, y, "│", RULE);
        set_cell(buf, x1, y, "│", RULE);
    }
    set_cell(buf, x0, y0, "┌", RULE);
    set_cell(buf, x1, y0, "┐", RULE);
    set_cell(buf, x0, y1, "└", RULE);
    set_cell(buf, x1, y1, "┘", RULE);
}

fn hrule(buf: &mut Buffer, y: u16, x0: u16, x1: u16) {
    for x in x0..=x1 {
        set_cell(buf, x, y, "─", RULE);
    }
}

fn vrule(buf: &mut Buffer, x: u16, y0: u16, y1: u16) {
    for y in y0..=y1 {
        set_cell(buf, x, y, "│", RULE);
    }
}

/// The rectangle the city window occupies over `area`, centred across it.
pub(crate) fn window_rect(area: Rect) -> Rect {
    let width = (IDEAL_WIDTH.min(area.width.saturating_sub(2)).max(2)) & !1;
    let height = (IDEAL_HEIGHT.min(area.height.saturating_sub(2)).max(2)) & !1;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// The rectangle of the close button, right-aligned on the population panel's
/// top row (immediately inside the top border).
pub(crate) fn close_button_rect(window: Rect) -> Rect {
    Rect {
        x: window.x + window.width - 1 - CLOSE_WIDTH,
        y: window.y + 1,
        width: CLOSE_WIDTH,
        height: 1,
    }
}

/// Thousands-separated number, e.g. `30000` -> `"30,000"`.
fn with_commas(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Turns needed to accumulate `needed` more of something at `income` per turn.
fn turns_to(needed: u32, income: u32) -> Option<u32> {
    if needed == 0 {
        Some(0)
    } else if income == 0 {
        None
    } else {
        Some(needed.div_ceil(income))
    }
}

pub struct CityWindow<'a> {
    view: &'a dyn GameView,
    city_id: CityId,
    scroll: usize,
}

impl<'a> CityWindow<'a> {
    /// A window over the city `city_id`. `scroll` is the offset of the
    /// improvements list.
    pub fn new(view: &'a dyn GameView, city_id: CityId, scroll: usize) -> Self {
        CityWindow {
            view,
            city_id,
            scroll,
        }
    }

    fn draw_population_panel(&self, buf: &mut Buffer, city: &City, rect: Rect) {
        let pop = city.population();
        let header = format!("{} (Pop: {})", city.name, with_commas(pop * 10_000));
        draw_text(buf, rect.x, rect.y, &header, BOLD);
        draw_text(
            buf,
            rect.right() - CLOSE_WIDTH,
            rect.y,
            CLOSE_TEXT,
            TEXT.add_modifier(Modifier::UNDERLINED),
        );
        let mut cx = rect.x;
        let count = pop as u16;
        for _ in 0..count.min(rect.width / 2) {
            cx = draw_text(buf, cx, rect.y + 1, &FA_POP.to_string(), TEXT);
        }
    }

    fn draw_resources_panel(&self, buf: &mut Buffer, income: &CityIncome, rect: Rect) {
        draw_text(buf, rect.x, rect.y, "City Resources", BOLD);
        let mut y = rect.y + 1;
        let rows: [(&str, char, u32); 5] = [
            ("Food", FA_FOOD, income.food),
            ("Shields", FA_SHIELD, income.resources),
            ("Trade", FA_TRADE, income.trade),
            ("Research", FA_RESEARCH, income.research),
            ("Gold", FA_GOLD, income.gold),
        ];
        for (label, icon, value) in rows {
            if y >= rect.bottom() {
                break;
            }
            let mut x = rect.x;
            x = draw_text(buf, x, y, &icon.to_string(), BOLD);
            let text = format!(" {label:<10}{value}");
            draw_text(buf, x, y, &text, TEXT);
            y += 1;
        }
        for resource in &income.special_resources {
            if y >= rect.bottom() {
                break;
            }
            let mut x = rect.x;
            x = draw_text(buf, x, y, &resource_icon(*resource).to_string(), BOLD);
            draw_text(buf, x, y, &format!(" {resource:?}"), TEXT);
            y += 1;
        }
    }

    fn draw_minimap(&self, buf: &mut Buffer, city: &City, rect: Rect) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let inner = Rect {
            x: rect.x + 1,
            y: rect.y + 1,
            width: rect.width - 2,
            height: rect.height - 2,
        };
        // The city sits at the centre of the 5x5 grid; draw every tile with
        // the same two-cell-per-tile paint as the ordinary map.
        for dy in 0..inner.height {
            for dx in (0..inner.width).step_by(TILE_WIDTH) {
                let wx = (city.location.x as isize + (dx / TILE_WIDTH as u16) as isize - 2)
                    .rem_euclid(map_w as isize) as usize;
                let raw_y = city.location.y as isize + dy as isize - 2;
                // Rows beyond the map's north/south edge paint as void.
                let wy = if (0..map_h as isize).contains(&raw_y) {
                    raw_y as usize
                } else {
                    map_h
                };
                paint_tile(
                    buf,
                    inner.x + dx,
                    inner.y + dy,
                    self.view,
                    wx,
                    wy,
                    map_w,
                    map_h,
                    Some(city.id()),
                    None,
                    false,
                );
            }
        }
    }

    fn draw_improvements(&self, buf: &mut Buffer, city: &City, rect: Rect) {
        draw_text(buf, rect.x, rect.y, "City Improvements", BOLD);
        let mut names: Vec<&'static str> = city.improvements().iter().map(|i| i.name()).collect();
        names.sort_unstable();
        if names.is_empty() {
            draw_text(buf, rect.x, rect.y + 1, "(none)", TEXT);
            return;
        }
        let visible = (rect.height - 1) as usize;
        let offset = self.scroll.min(names.len().saturating_sub(visible));
        for (i, name) in names.iter().enumerate().skip(offset).take(visible) {
            let row = rect.y + 1 + i as u16 - offset as u16;
            draw_text(buf, rect.x, row, name, TEXT);
        }
        if names.len() > visible {
            draw_scrollbar(buf, rect, names.len(), offset);
        }
    }

    fn draw_food_panel(&self, buf: &mut Buffer, city: &City, income: &CityIncome, rect: Rect) {
        draw_text(buf, rect.x, rect.y, "Food Storage", BOLD);
        let consumed = city.food_consumption();
        let surplus = income.food as i32 - consumed as i32;
        let remaining = city.growth_target().saturating_sub(city.food());
        let turns = turns_to(remaining, surplus.max(0) as u32);
        let mut lines = vec![
            format!("{} Stored   {}", FA_FOOD, city.food()),
            format!("{} Eaten    {consumed}", FA_FOOD),
            format!("{} Surplus  {surplus:+}", FA_FOOD),
        ];
        if let Some(t) = turns {
            lines.push(format!("{} Turns    {t}", FA_FOOD));
        }
        for (i, line) in lines.iter().enumerate() {
            let row = rect.y + 1 + i as u16;
            if row >= rect.bottom() {
                break;
            }
            draw_text(buf, rect.x, row, line, TEXT);
        }
    }

    fn draw_units_panel(&self, buf: &mut Buffer, city: &City, rect: Rect) {
        draw_text(buf, rect.x, rect.y, "Units", BOLD);
        let units = self.view.home_units(city.id());
        if units.is_empty() {
            draw_text(buf, rect.x, rect.y + 1, "(none)", TEXT);
            return;
        }
        for (i, unit) in units.iter().enumerate() {
            let row = rect.y + 1 + i as u16;
            if row >= rect.bottom() {
                break;
            }
            draw_text(buf, rect.x, row, &format!("{:?}", unit.unit_class), TEXT);
        }
    }

    fn draw_production_panel(
        &self,
        buf: &mut Buffer,
        city: &City,
        income: &CityIncome,
        rect: Rect,
    ) {
        let target = city.production_target();
        let label = match target {
            Some(ProductionTarget::Improvement(improvement)) => improvement.name().to_string(),
            Some(ProductionTarget::Unit(unit_class)) => format!("{unit_class:?}"),
            None => "Idle".to_string(),
        };
        draw_text(buf, rect.x, rect.y, &label, BOLD);
        if let Some(target) = target {
            let cost = target.resource_cost();
            let stored = city.resource_stored();
            let mut lines = vec![
                format!("{} Stored   {stored}", FA_SHIELD),
                format!("{} Cost     {cost}", FA_SHIELD),
                format!("{} Income   {}", FA_SHIELD, income.resources),
            ];
            if let Some(t) = turns_to(cost.saturating_sub(stored), income.resources) {
                lines.push(format!("{} Turns    {t}", FA_SHIELD));
            }
            for (i, line) in lines.iter().enumerate() {
                let row = rect.y + 1 + i as u16;
                if row >= rect.bottom() {
                    break;
                }
                draw_text(buf, rect.x, row, line, TEXT);
            }
        }
    }
}

/// A single-column scrollbar flush to the right edge of the improvements panel.
fn draw_scrollbar(buf: &mut Buffer, rect: Rect, total: usize, offset: usize) {
    let visible = (rect.height - 1) as usize;
    if total <= visible {
        return;
    }
    let track = rect.height - 1;
    let x = rect.right() - 1;
    let thumb_h = (((track as u32 * visible as u32) / total as u32).max(1)) as u16;
    let max_offset = total - visible;
    let thumb_y = ((track - thumb_h) as u32 * offset as u32 / max_offset as u32) as u16;
    for i in 0..track {
        let in_thumb = i >= thumb_y && i < thumb_y + thumb_h;
        set_cell(
            buf,
            x,
            rect.y + 1 + i,
            if in_thumb { "█" } else { "░" },
            RULE,
        );
    }
}

impl<'a> Widget for CityWindow<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(city) = self.view.city(self.city_id) else {
            return;
        };
        if city.owner() != self.view.current_player_id() {
            return;
        }
        let window = window_rect(area);
        if window.width < 4 || window.height < 4 {
            return;
        }
        let inner = Rect {
            x: window.x + 1,
            y: window.y + 1,
            width: window.width - 2,
            height: window.height - 2,
        };
        fill_rect(buf, inner);
        draw_border(buf, window);

        let iw = inner.width as usize;
        let ih = inner.height as usize;
        if iw < 20 || ih < 12 {
            return;
        }
        let income = self.view.city_income(self.city_id);

        // Population panel (2 rows), a rule, the middle band, a rule, bottom.
        let pop = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 2,
        };
        let mid_h = (ih - 2 - 2) / 2;
        let mid = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: mid_h as u16,
        };
        let bottom_top = mid.bottom() + 1;
        let bottom = Rect {
            x: inner.x,
            y: bottom_top,
            width: inner.width,
            height: ih as u16 - (bottom_top - inner.y),
        };

        self.draw_population_panel(buf, city, pop);
        hrule(buf, pop.bottom(), inner.x, inner.right() - 1);
        hrule(buf, mid.bottom(), inner.x, inner.right() - 1);

        // Middle band: grouped resources (2/3) and the improvements list.
        let grouped_w = iw * 2 / 3;
        let grouped = Rect {
            x: mid.x,
            y: mid.y,
            width: grouped_w as u16,
            height: mid.height,
        };
        let improvements = Rect {
            x: mid.x + grouped_w as u16,
            y: mid.y,
            width: (iw - grouped_w) as u16,
            height: mid.height,
        };
        vrule(buf, grouped.right() - 1, mid.y, mid.bottom() - 1);

        // Within the grouped panel, the 5x5 mini-map sits flush to its right edge
        // and centred vertically; tiles span two cells, like the main map.
        let minimap = Rect {
            x: grouped.right() - MINIMAP_WIDTH,
            y: mid.y + (mid.height - MINIMAP_HEIGHT) / 2,
            width: MINIMAP_WIDTH,
            height: MINIMAP_HEIGHT,
        };
        draw_border(buf, minimap);
        self.draw_minimap(buf, city, minimap);
        let resources = Rect {
            x: grouped.x,
            y: mid.y,
            width: grouped.width - MINIMAP_WIDTH - 1,
            height: mid.height,
        };
        vrule(buf, resources.right(), mid.y, mid.bottom() - 1);
        self.draw_resources_panel(buf, &income, resources);

        self.draw_improvements(buf, city, improvements);

        // Bottom band: three equal panels with black rules between.
        let panel_w = (iw - 2) / 3;
        let food = Rect {
            x: bottom.x,
            y: bottom.y,
            width: panel_w as u16,
            height: bottom.height,
        };
        let units_x = food.right() + 1;
        let units = Rect {
            x: units_x,
            y: bottom.y,
            width: panel_w as u16,
            height: bottom.height,
        };
        let production = Rect {
            x: units.right() + 1,
            y: bottom.y,
            width: bottom.right() - (units.right() + 1),
            height: bottom.height,
        };
        vrule(buf, food.right(), bottom.y, bottom.bottom() - 1);
        vrule(buf, units.right(), bottom.y, bottom.bottom() - 1);

        self.draw_food_panel(buf, city, &income, food);
        self.draw_units_panel(buf, city, units);
        self.draw_production_panel(buf, city, &income, production);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::{Location, Tile};
    use crate::model::cities::{City, CityId, CityImprovement, ProductionTarget};
    use crate::model::civilizations::{Civilization, PlayerId};
    use crate::model::geography::Terrain;
    use crate::model::units::UnitClass;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        w: usize,
        h: usize,
        tile: Tile,
        city: Option<City>,
        current_player: PlayerId,
    }

    impl FakeView {
        fn with(mut self, city: City) -> Self {
            self.city = Some(city);
            self
        }
    }

    impl GameView for FakeView {
        fn width(&self) -> usize {
            self.w
        }
        fn height(&self) -> usize {
            self.h
        }
        fn tile(&self, _x: usize, _y: usize) -> &Tile {
            &self.tile
        }
        fn units_at(&self, _x: usize, _y: usize) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn city_at(&self, x: usize, y: usize) -> Option<&City> {
            self.city
                .as_ref()
                .filter(|c| c.location.x as usize == x && c.location.y as usize == y)
        }
        fn player_units(&self) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn player_cities(&self) -> Vec<&City> {
            self.city.as_ref().map(|c| vec![c]).unwrap_or_default()
        }
        fn city(&self, id: CityId) -> Option<&City> {
            self.city.as_ref().filter(|c| c.id() == id)
        }
        fn current_player_id(&self) -> PlayerId {
            self.current_player
        }
        fn city_income(&self, _id: CityId) -> CityIncome {
            CityIncome {
                food: 7,
                resources: 3,
                trade: 1,
                gold: 2,
                research: 4,
                special_resources: vec![SpecialResource::Gold],
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
    }

    fn fake_view() -> FakeView {
        FakeView {
            w: 20,
            h: 20,
            tile: Tile::new(Terrain::Grassland),
            city: None,
            current_player: PlayerId::new(0),
        }
    }

    fn london() -> City {
        let mut city = City::new(
            "London",
            Location::new(3, 3),
            PlayerId::new(0),
            CityId::new(0),
        );
        city.grow();
        city.grow();
        city
    }

    fn render(city: City, scroll: usize) -> Buffer {
        let view = fake_view().with(city);
        let id = view.city.as_ref().unwrap().id();
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(CityWindow::new(&view, id, scroll), frame.area()))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn pos_of(x: u16, y: u16) -> (u16, u16) {
        let win = window_rect(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        });
        (win.x + x, win.y + y)
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn window_is_centred_and_bordered() {
        let buf = render(london(), 0);
        let (x, y) = pos_of(0, 0);
        assert_eq!(buf.cell((x, y)).unwrap().symbol(), "┌");
        assert_eq!(buf.cell((x + 63, y)).unwrap().symbol(), "┐");
        assert_eq!(buf.cell((x, y + 29)).unwrap().symbol(), "└");
    }

    #[test]
    fn population_panel_shows_name_population_and_close_button() {
        let buf = render(london(), 0);
        let (x, y) = pos_of(1, 1);
        let row = row_text(&buf, x, y, 62);
        assert!(row.contains("London (Pop: 30,000)"), "row: {row:?}");
        assert!(row.contains("✕ Close"), "row: {row:?}");
    }

    #[test]
    fn population_row_shows_one_person_per_citizen() {
        let buf = render(london(), 0);
        let (x, y) = pos_of(1, 2);
        // Each citizen is a wide glyph followed by its (skipped) half-cell.
        assert!(
            row_text(&buf, x, y, 18).contains(&format!("{FA_POP} {FA_POP} {FA_POP}")),
            "row was {:?}",
            row_text(&buf, x, y, 18)
        );
    }

    #[test]
    fn city_resources_lines_include_the_big_five() {
        let buf = render(london(), 0);
        let (x, y) = pos_of(1, 4);
        assert!(row_text(&buf, x, y, 20).contains("City Resources"));
        for needle in ["Food", "Shields", "Trade", "Research", "Gold"] {
            let found = (1..=11).any(|row| row_text(&buf, x, y + row, 20).contains(needle));
            assert!(found, "missing {needle} row");
        }
    }

    #[test]
    fn minimap_shows_the_city_block_digit_at_its_centre() {
        let buf = render(london(), 0);
        let win = window_rect(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        });
        // Re-derive the layout so this doesn't drift from the widget.
        let inner_x = win.x + 1;
        let inner_y = win.y + 1;
        let grouped_w = (win.width - 2) * 2 / 3;
        let mid_top = inner_y + 3;
        let minimap_x = inner_x + grouped_w - MINIMAP_WIDTH;
        let minimap_y = mid_top + (12 - MINIMAP_HEIGHT) / 2;
        let centre_x = minimap_x + 1 + 2 * 2;
        let centre_y = minimap_y + 1 + 2;
        assert_eq!(buf.cell((centre_x, centre_y)).unwrap().symbol(), "3");
    }

    #[test]
    fn minimap_tiles_span_two_cells_each() {
        let buf = render(london(), 0);
        let win = window_rect(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        });
        let inner_x = win.x + 1;
        let grouped_w = (win.width - 2) * 2 / 3;
        let mid_top = win.y + 4;
        let minimap_x = inner_x + grouped_w - MINIMAP_WIDTH;
        let minimap_y = mid_top + (12 - MINIMAP_HEIGHT) / 2;
        let row_y = minimap_y + 1 + 2; // middle mini-map row (the city's row)
        let glyph = |x: u16| buf.cell((x, row_y)).unwrap().symbol();
        let g0 = minimap_x + 1;
        // West of the city tile the terrain glyph occupies cell one, with a
        // blank style-carrying cell to its right — exactly like the main map.
        assert_ne!(glyph(g0), " ");
        assert_eq!(glyph(g0 + 1), " ");
        assert_ne!(glyph(g0 + 2), " ");
        assert_eq!(glyph(g0 + 3), " ");
        // The city digit also spans its own two cells.
        assert_eq!(glyph(g0 + 4), "3");
        assert_eq!(glyph(g0 + 5), " ");
    }

    #[test]
    fn improvements_are_sorted_alphabetically() {
        let mut city = london();
        for improvement in [
            CityImprovement::University,
            CityImprovement::Library,
            CityImprovement::CityWalls,
        ] {
            city.add_improvement(improvement);
        }
        let buf = render(city, 0);
        let (x, y) = pos_of(42, 5);
        let names: Vec<String> = (0..12)
            .map(|row| row_text(&buf, x, y + row, 20).trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert!(names.iter().any(|n| n.contains("City Walls")), "{names:?}");
    }

    #[test]
    fn production_panel_names_the_target_and_shows_progress() {
        let mut city = london();
        city.set_production(ProductionTarget::Unit(UnitClass::Militia));
        for _ in 0..3 {
            city.tick(0, 3);
        }
        let buf = render(city, 0);
        let (x, y) = pos_of(43, 17);
        let text = row_text(&buf, x, y, 20);
        assert!(text.contains("Militia"), "production label row: {text:?}");
    }

    #[test]
    fn foreign_cities_are_hidden() {
        let mut city = london();
        city.change_owner(PlayerId::new(1));
        let buf = render(city, 0);
        let (x, y) = pos_of(0, 0);
        let central: String = (0..10)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect();
        assert_eq!(central, "          ", "foreign city must not paint");
    }
}
