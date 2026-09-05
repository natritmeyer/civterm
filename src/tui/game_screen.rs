use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::theme::{ACCENT, DIM, draw_text};
use crate::game_engine::{Event, GameView};
use crate::model::cities::CityId;
use crate::model::civilizations::Civilization;
use crate::model::geography::Terrain;
use crate::model::units::{UnitClass, UnitId, UnitOrder};

/// Fixed width of the left-hand information column.
pub const LEFT_COLUMN_WIDTH: u16 = 36;

/// Width of the event-log overlay when visible.
const EVENT_LOG_WIDTH: u16 = 44;

pub const TILE_WIDTH: usize = 2;

fn terrain_colors(terrain: Terrain) -> (Color, Color) {
    use Terrain::*;
    match terrain {
        Ocean => (Color::Rgb(40, 90, 160), Color::Rgb(20, 40, 80)),
        Plains => (Color::Rgb(185, 205, 110), Color::Rgb(80, 95, 45)),
        Jungle => (Color::Rgb(140, 210, 90), Color::Rgb(45, 95, 40)),
        Forest => (Color::Rgb(60, 150, 60), Color::Rgb(25, 70, 30)),
        Hills => (Color::Rgb(150, 130, 90), Color::Rgb(70, 60, 40)),
        Mountain => (Color::Rgb(170, 170, 180), Color::Rgb(80, 80, 90)),
        Desert => (Color::Rgb(210, 200, 130), Color::Rgb(110, 100, 50)),
        Tundra => (Color::Rgb(170, 200, 200), Color::Rgb(70, 90, 90)),
        Swamp => (Color::Rgb(90, 150, 120), Color::Rgb(40, 80, 60)),
        Grassland => (Color::Rgb(60, 160, 80), Color::Rgb(25, 75, 40)),
    }
}

/// The civilization flag colour used to flash a selected unit's tile while it
/// awaits instruction.
pub(crate) fn civilization_color(civ: Civilization) -> Color {
    use Civilization::*;
    match civ {
        American => Color::Rgb(220, 130, 40),
        Aztec => Color::Rgb(180, 120, 80),
        Babylonian => Color::Rgb(120, 90, 180),
        Chinese => Color::Rgb(190, 60, 60),
        Egyptian => Color::Rgb(200, 170, 90),
        English => Color::Rgb(200, 60, 80),
        French => Color::Rgb(90, 110, 210),
        German => Color::Rgb(90, 90, 90),
        Greek => Color::Rgb(120, 150, 200),
        Indian => Color::Rgb(170, 120, 60),
        Mongol => Color::Rgb(130, 90, 40),
        Roman => Color::Rgb(160, 50, 50),
        Russian => Color::Rgb(120, 40, 140),
        Zulu => Color::Rgb(90, 60, 50),
    }
}

pub(crate) fn tile_style(explored: bool, terrain: Terrain) -> Style {
    if !explored {
        return Style::default()
            .fg(Color::Rgb(20, 20, 20))
            .bg(Color::Rgb(10, 10, 36));
    }
    let (fg, bg) = terrain_colors(terrain);
    Style::default().fg(fg).bg(bg)
}

/// Paint one world tile at `(x, y)` spanning `TILE_WIDTH` columns, exactly as
/// the main map does: the terrain/unit/city symbol in the first cell and a
/// blank style-carrying cell to its right. World columns wrap horizontally.
/// Returns the city's name when the tile holds an explored city so callers can
/// draw the label beneath it.
///
/// `selected_city` outlines the matching city tile; `flashing` gates the
/// selected-unit flash, which itself must be the selected unit with moves left.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tile(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    view: &dyn GameView,
    world_x: usize,
    world_y: usize,
    map_w: usize,
    map_h: usize,
    selected_city: Option<CityId>,
    selected_unit: Option<UnitId>,
    flashing: bool,
) -> Option<String> {
    let map_x = world_x % map_w;
    let (symbol, style, city_name) = if world_y >= map_h {
        (' ', Style::default().bg(Color::Rgb(6, 6, 22)), None)
    } else {
        let tile = view.tile(map_x, world_y);
        let explored = view.explored(map_x, world_y);
        let terrain = tile.terrain;
        let mut style = tile_style(explored, terrain);

        let unit = view.units_at(map_x, world_y);
        let city = view.city_at(map_x, world_y);

        let (symbol, city_name) = if explored && let Some(city) = city {
            // A city occupies the whole tile: show its population on a
            // background of the owning civilization's colour.
            (population_digit(city.population()), Some(city.name.clone()))
        } else if let Some(u) = unit.first() {
            (first_letter(u.unit_class), None)
        } else {
            (terrain.as_char(), None)
        };

        if city_name.is_some()
            && let Some(city) = city
        {
            style = style.bg(civilization_color(view.civilization_of(city.owner())));
            // The selected city's tile is outlined so it stands out.
            if selected_city == Some(city.id()) {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
        }
        if city_name.is_none() && !unit.is_empty() {
            style = style
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED);
        }

        // The selected unit awaiting instruction flashes once per second: its
        // tile turns the civilization flag colour then dims back to terrain.
        if flashing
            && let Some(id) = selected_unit
            && unit
                .iter()
                .any(|u| u.id() == id && u.order() == UnitOrder::Idle && u.moves_remaining() > 0)
        {
            style = style.bg(civilization_color(view.current_player()));
        }

        (symbol, style, city_name)
    };

    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(&symbol.to_string());
        cell.set_style(style);
    }
    if TILE_WIDTH > 1
        && let Some(cell) = buf.cell_mut((x + 1, y))
    {
        cell.set_symbol(" ");
        cell.set_style(style);
    }
    city_name
}

pub struct GameScreen<'a> {
    view: &'a dyn GameView,
    focus: Option<(usize, usize)>,
    camera: (usize, usize),
    selected_unit: Option<UnitId>,
    selected_city: Option<CityId>,
    /// An instant pushed forward every frame; drives the idle-unit flash.
    now: Duration,
    /// Whether the event log overlays the map pane's top-right corner.
    show_events: bool,
    /// The most recent event messages, oldest first.
    events: &'a [Event],
}

impl<'a> GameScreen<'a> {
    /// `camera` is the world-tile coordinate at the top-left of the map pane.
    /// `selected_unit` (if any) is the unit whose tile flashes while it awaits
    /// instruction; `now` is a monotonic clock used to time that flash.
    /// `show_events` toggles the event log overlay, which shows `events`
    /// (most recent messages, oldest first) in the top-right of the map pane.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view: &'a dyn GameView,
        focus: Option<(usize, usize)>,
        camera: (usize, usize),
        selected_unit: Option<UnitId>,
        selected_city: Option<CityId>,
        now: Duration,
        show_events: bool,
        events: &'a [Event],
    ) -> Self {
        GameScreen {
            view,
            focus,
            camera,
            selected_unit,
            selected_city,
            now,
            show_events,
            events,
        }
    }

    /// Whether the selected idle unit's tile is currently showing the
    /// civilization flash colour: on for half a second, off for half a second,
    /// repeating once per second.
    fn flash_phase(&self) -> bool {
        self.now.as_millis() % 1000 < 500
    }

    fn draw_main_map(&self, area: Rect, buf: &mut Buffer) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let cell_cols = area.width as usize / TILE_WIDTH;
        let cell_rows = area.height as usize;
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }

        let flashing = self.flash_phase();
        // City labels are drawn in a second pass, after every tile row has been
        // painted, so a label on one row's tiles is not overwritten by the next
        // map row below it.
        let mut city_labels: Vec<(u16, u16, String)> = Vec::new();

        // Tiles are shown at 1:1; `camera` is the top-left world tile. The map
        // wraps horizontally (east/west) but not vertically — tiles beyond the
        // north/south edge render as void.
        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let src_x = self.camera.0 + col;
                let src_y = self.camera.1 + row;
                let cx = area.x + (col * TILE_WIDTH) as u16;
                let cy = area.y + row as u16;

                if let Some(name) = paint_tile(
                    buf,
                    cx,
                    cy,
                    self.view,
                    src_x,
                    src_y,
                    map_w,
                    map_h,
                    self.selected_city,
                    self.selected_unit,
                    flashing,
                ) {
                    city_labels.push((cx, cy + 1, name));
                }
            }
        }

        // Second pass: city name labels, so they sit on top of the map.
        for (tile_cx, row_y, name) in city_labels {
            draw_city_label(buf, tile_cx, row_y, &name);
        }
    }

    /// Overlays the event log on the top-right corner of the map pane. The box
    /// shows up to five most-recent messages, newest at the bottom, or a
    /// placeholder when none have been recorded yet.
    fn draw_event_log(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_events {
            return;
        }
        let rows = self.events.len().clamp(1, 5) as u16 + 2;
        let width = area.width.min(EVENT_LOG_WIDTH);
        if width < 4 || rows > area.height {
            return;
        }
        let left = area.right() - width;
        let top = area.y;
        let right = area.right() - 1;
        let bottom = top + rows - 1;

        let bg = Color::Rgb(12, 12, 40);
        let border = Style::default().fg(DIM).bg(bg);
        let message_style = Style::default().fg(Color::Rgb(220, 220, 235)).bg(bg);
        let title_style = Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD)
            .bg(bg);
        let inner_width = (width - 2) as usize;

        // Fill the box so the map underneath does not show through.
        for y in top..=bottom {
            for x in left..=right {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_symbol(" ");
                    cell.set_bg(bg);
                }
            }
        }

        // Top border with the title centred.
        fill_row(buf, left, right, top, "─", border);
        set_cell(buf, left, top, "┌", border);
        set_cell(buf, right, top, "┐", border);
        let title = " EVENTS ";
        let title_x = left + (width - title.chars().count() as u16) / 2;
        draw_text(buf, right, title_x, top, title, title_style);

        // Message rows, newest at the bottom. `events` holds at most five in
        // oldest-first order, so the last one lands on the final message row.
        // With no events yet, show a placeholder so the box is still visible.
        let start = self.events.len().saturating_sub(5);
        for offset in 0..rows.saturating_sub(2) as usize {
            let y = top + 1 + offset as u16;
            let visible = match self.events.get(start + offset) {
                Some(event) => event
                    .message()
                    .chars()
                    .take(inner_width)
                    .collect::<String>(),
                None => "(no events yet)"
                    .chars()
                    .take(inner_width)
                    .collect::<String>(),
            };
            draw_text(buf, right, left + 1, y, &visible, message_style);
        }

        // Bottom border.
        fill_row(buf, left, right, bottom, "─", border);
        set_cell(buf, left, bottom, "└", border);
        set_cell(buf, right, bottom, "┘", border);
    }

    fn draw_minimap(&self, area: Rect, buf: &mut Buffer) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let cell_cols = area.width as usize;
        let cell_rows = area.height as usize;
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }
        let x_stride = (map_w as f64 / cell_cols as f64).ceil().max(1.0) as usize;
        let y_stride = (map_h as f64 / cell_rows as f64).ceil().max(1.0) as usize;

        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let src_x = (col * x_stride).min(map_w - 1);
                let src_y = (row * y_stride).min(map_h - 1);
                if !self.view.explored(src_x, src_y) {
                    continue;
                }
                let terrain = self.view.tile(src_x, src_y).terrain.as_char();
                let color = if terrain == '~' {
                    Color::Rgb(40, 90, 150)
                } else {
                    Color::Rgb(90, 160, 70)
                };
                if let Some(cell) = buf.cell_mut((area.x + col as u16, area.y + row as u16)) {
                    cell.set_symbol(" ");
                    cell.set_bg(color);
                }
            }
        }
    }

    fn draw_player_stats(&self, area: Rect, buf: &mut Buffer) {
        let year = self.view.year();
        let gold = self.view.gold();
        let civ = self.view.current_player();
        let x = area.x + 1;

        draw_text(
            buf,
            area.right(),
            x,
            area.y,
            civ.display_name(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        );
        draw_text(
            buf,
            area.right(),
            x,
            area.y + 1,
            &format!("Year  {}", format_year(year)),
            Style::default().fg(DIM),
        );
        draw_text(
            buf,
            area.right(),
            x,
            area.y + 2,
            &format!("Gold  {}", gold),
            Style::default().fg(Color::Rgb(230, 190, 80)),
        );

        let target = self.view.advancement_in_progress();
        let progress = self.view.research_progress();
        let cost = self.view.research_cost();
        let income = self.view.research_income();
        let header_y = area.y + 4;
        let label = if let Some(t) = target {
            format!("Researching {:?}", t)
        } else {
            "Researching  --".to_string()
        };
        draw_text(
            buf,
            area.right(),
            x,
            header_y,
            &label,
            Style::default().fg(Color::Rgb(180, 140, 255)),
        );
        if let Some(cost) = cost {
            let pct = if cost == 0 {
                0
            } else {
                (progress as f64 / cost as f64 * 10.0) as usize
            };
            let bar: String = "█".repeat(pct) + &"░".repeat(10 - pct);
            draw_text(
                buf,
                area.right(),
                x,
                header_y + 1,
                &bar,
                Style::default().fg(Color::Rgb(180, 140, 255)),
            );
            draw_text(
                buf,
                area.right(),
                x + 22,
                header_y + 1,
                &format!("{progress}/{cost} +{income}"),
                Style::default().fg(DIM),
            );
        }
    }

    fn draw_focus(&self, area: Rect, buf: &mut Buffer) {
        let civ = self.view.current_player();
        let x = area.x + 1;
        draw_text(
            buf,
            area.right(),
            x,
            area.y,
            &format!("In turn: {}", civ.display_name()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        );

        let mut row = area.y + 2;
        match self.focus {
            Some((fx, fy)) => {
                let unit = self.view.units_at(fx, fy).first().copied();
                if let Some(unit) = unit {
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        &format!("Unit: {:?} mv {}", unit.unit_class, unit.moves_remaining()),
                        Style::default().fg(Color::Rgb(230, 200, 120)),
                    );
                    row += 1;
                } else {
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        "(no unit here)",
                        Style::default().fg(DIM),
                    );
                    row += 1;
                }
                let tile = self.view.tile(fx, fy);
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!("Terrain: {:?}", tile.terrain),
                    Style::default().fg(DIM),
                );
                row += 1;
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!(
                        "Food {}  Prod {}  Trade {}",
                        tile.yields_food(),
                        tile.yields_resources(),
                        tile.yields_trade()
                    ),
                    Style::default().fg(DIM),
                );
                if tile.has_road() || tile.is_mined() || tile.is_irrigated() {
                    row += 1;
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        &[
                            tile.has_road().then_some("road"),
                            tile.is_mined().then_some("mine"),
                            tile.is_irrigated().then_some("irrigation"),
                        ]
                        .iter()
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_string(),
                        Style::default().fg(DIM),
                    );
                }
            }
            None => {
                draw_text(
                    buf,
                    area.right(),
                    x,
                    area.y + 1,
                    "(move a unit to inspect it)",
                    Style::default().fg(DIM),
                );
            }
        }
    }
}

fn first_letter(unit_class: UnitClass) -> char {
    format!("{unit_class:?}").chars().next().unwrap_or('?')
}

/// The digit shown on a city tile: its population, clipped to a single char.
pub(crate) fn population_digit(population: u32) -> char {
    if population >= 10 {
        '9'
    } else {
        char::from_digit(population, 10).unwrap_or('0')
    }
}

/// The yellow used for city labels.
const CITY_LABEL_FG: Color = Color::Rgb(255, 215, 0);

/// Set a single cell's symbol and style.
fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

/// Fill a horizontal run of cells between `x0` and `x1` (inclusive) on row `y`.
fn fill_row(buf: &mut Buffer, x0: u16, x1: u16, y: u16, symbol: &str, style: Style) {
    for x in x0..=x1 {
        set_cell(buf, x, y, symbol, style);
    }
}

/// Draw `name` centred beneath a city tile spanning the two cells that begin
/// at `tile_cx`, on the row `row_y`, in yellow.
fn draw_city_label(buf: &mut Buffer, tile_cx: u16, row_y: u16, name: &str) {
    // The two-cell tile is centred on its second cell; centre the name there.
    let center_col = tile_cx as isize + 1;
    let len = name.len();
    let start = center_col - (len / 2) as isize;

    for (i, ch) in name.chars().enumerate() {
        let col = start + i as isize;
        if col < 0 {
            continue;
        }
        if let Some(cell) = buf.cell_mut((col as u16, row_y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(Style::default().fg(CITY_LABEL_FG).bold());
        }
    }
}

fn format_year(year: i32) -> String {
    if year < 0 {
        format!("{} BC", -year)
    } else {
        format!("{year} AD")
    }
}

impl<'a> Widget for GameScreen<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut left = area;
        left.width = LEFT_COLUMN_WIDTH.min(area.width);
        let mut right = area;
        right.x += LEFT_COLUMN_WIDTH;
        right.width = area.width.saturating_sub(LEFT_COLUMN_WIDTH);

        // Fill the backgrounds: the left column is grey, the map pane is dark.
        let left_bg = Color::DarkGray;
        let right_bg = Color::Rgb(12, 12, 40);
        for y in area.y..area.bottom() {
            for x in area.x..left.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(left_bg);
                }
            }
            for x in left.right().max(area.x)..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(right_bg);
                }
            }
        }

        let left_mid = left.height / 3;
        let left_remaining = left.height - left_mid;
        let stats_height = left_remaining / 2;
        let mini_height = left_mid;
        let focus_height = left_remaining - stats_height;

        let minimap_area = Rect::new(left.x, left.y, left.width, mini_height);
        let stats_area = Rect::new(left.x, left.y + mini_height, left.width, stats_height);
        let focus_area = Rect::new(
            left.x,
            left.y + mini_height + stats_height,
            left.width,
            focus_height,
        );

        // The middle (player stats) panel gets a lighter grey background.
        for y in stats_area.y..stats_area.bottom() {
            for x in stats_area.x..stats_area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(Color::Gray);
                }
            }
        }

        self.draw_minimap(minimap_area, buf);
        self.draw_player_stats(stats_area, buf);
        self.draw_focus(focus_area, buf);
        self.draw_main_map(right, buf);
        self.draw_event_log(right, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cartography::Direction;
    use crate::model::civilizations::Civilization;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        w: usize,
        h: usize,
        tile: crate::model::cartography::Tile,
        city: Option<crate::model::cities::City>,
        explored: bool,
    }

    impl GameView for FakeView {
        fn width(&self) -> usize {
            self.w
        }
        fn height(&self) -> usize {
            self.h
        }
        fn tile(&self, _x: usize, _y: usize) -> &crate::model::cartography::Tile {
            &self.tile
        }
        fn units_at(&self, _x: usize, _y: usize) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn city_at(&self, _x: usize, _y: usize) -> Option<&crate::model::cities::City> {
            self.city.as_ref()
        }
        fn player_units(&self) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn player_cities(&self) -> Vec<&crate::model::cities::City> {
            self.city
                .as_ref()
                .map(|city| vec![city])
                .unwrap_or_default()
        }
        fn city(&self, id: crate::model::cities::CityId) -> Option<&crate::model::cities::City> {
            self.city.as_ref().filter(|city| city.id() == id)
        }
        fn current_player_id(&self) -> crate::model::civilizations::PlayerId {
            crate::model::civilizations::PlayerId::new(0)
        }
        fn city_income(
            &self,
            _id: crate::model::cities::CityId,
        ) -> crate::game_engine::game_view::CityIncome {
            crate::game_engine::game_view::CityIncome {
                food: 2,
                resources: 0,
                trade: 1,
                gold: 0,
                research: 0,
                special_resources: Vec::new(),
            }
        }
        fn home_units(
            &self,
            _city: crate::model::cities::CityId,
        ) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn explored(&self, _x: usize, _y: usize) -> bool {
            self.explored
        }
        fn current_player(&self) -> Civilization {
            Civilization::English
        }
        fn civilization_of(&self, _player: crate::model::civilizations::PlayerId) -> Civilization {
            Civilization::English
        }
        fn turn(&self) -> u32 {
            1
        }
        fn year(&self) -> i32 {
            4000
        }
        fn gold(&self) -> u32 {
            50
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
            w: 80,
            h: 50,
            tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Ocean),
            city: None,
            explored: true,
        }
    }

    fn render() -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(&view, None, (0, 0), None, None, Duration::ZERO, false, &[]),
                    frame.area(),
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn year_4000_bc_is_formatted() {
        assert_eq!(format_year(-4000), "4000 BC");
        assert_eq!(format_year(50), "50 AD");
        assert_eq!(format_year(0), "0 AD");
    }

    #[test]
    fn the_left_column_is_fixed_width() {
        assert_eq!(LEFT_COLUMN_WIDTH, 36);
        let buf = render();
        // The right pane shows map tiles; the left column is background-filled.
        assert_ne!(
            buf.cell((LEFT_COLUMN_WIDTH, 1)).unwrap().style().bg,
            buf.cell((1, 1)).unwrap().style().bg
        );
    }

    #[test]
    fn the_middle_stats_panel_background_is_gray() {
        let buf = render();
        let height: u16 = 40;
        let mini_height = height / 3;
        let stats_height = (height - mini_height) / 2;
        let y = mini_height + stats_height / 2;
        assert_eq!(buf.cell((5, y)).unwrap().style().bg, Some(Color::Gray));
    }

    #[test]
    fn renders_a_populated_engine_without_panicking() {
        let mut engine = crate::game_engine::Engine::new(
            crate::game_engine::DEFAULT_MAP_WIDTH,
            crate::game_engine::DEFAULT_MAP_HEIGHT,
            crate::game_engine::Player::new(Civilization::English),
            vec![],
        );
        engine.populate_starting_world();

        let mut focus = None;
        let mut found_settler = false;
        for y in 0..engine.height() {
            for x in 0..engine.width() {
                if engine
                    .units_at(x, y)
                    .iter()
                    .any(|unit| unit.unit_class == crate::model::units::UnitClass::Settler)
                {
                    found_settler = true;
                    focus = Some((x, y));
                }
            }
        }
        assert!(found_settler, "the populated world starts with a settler");

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        focus,
                        (0, 0),
                        None,
                        None,
                        Duration::ZERO,
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
    }

    fn events_log(events: &[&str]) -> Vec<Event> {
        events.iter().map(|m| Event::new(*m)).collect()
    }

    fn row_text(buf: &Buffer, x0: u16, x1: u16, y: u16) -> String {
        (x0..=x1)
            .map(|x| buf.cell((x, y)).unwrap().symbol().chars().next().unwrap())
            .collect()
    }

    #[test]
    fn event_log_renders_a_box_in_the_top_right_when_enabled() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        let events = events_log(&["Unit 0 moves E", "London grows"]);
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &view,
                        None,
                        (0, 0),
                        None,
                        None,
                        Duration::ZERO,
                        true,
                        &events,
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();

        // The overlay sits in the map pane's top-right corner: x 76..120, and
        // 4 rows tall (top border, 2 messages, bottom border).
        assert_eq!(buffer.cell((76, 0)).unwrap().symbol(), "┌");
        assert_eq!(buffer.cell((119, 0)).unwrap().symbol(), "┐");
        assert_eq!(buffer.cell((76, 3)).unwrap().symbol(), "└");
        assert_eq!(buffer.cell((119, 3)).unwrap().symbol(), "┘");
        let top_row = row_text(buffer, 76, 119, 0);
        assert!(
            top_row.contains("EVENTS"),
            "title should be on the top border"
        );
        assert!(row_text(buffer, 77, 118, 1).contains("Unit 0 moves E"));
        assert!(row_text(buffer, 77, 118, 2).contains("London grows"));
    }

    #[test]
    fn event_log_newest_message_sits_at_the_bottom() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        let events = events_log(&["first", "second"]);
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &view,
                        None,
                        (0, 0),
                        None,
                        None,
                        Duration::ZERO,
                        true,
                        &events,
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert!(row_text(buffer, 77, 118, 1).contains("first"));
        assert!(row_text(buffer, 77, 118, 2).contains("second"));
    }

    #[test]
    fn event_log_with_no_events_still_renders_the_box() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(&view, None, (0, 0), None, None, Duration::ZERO, true, &[]),
                    frame.area(),
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer.cell((76, 0)).unwrap().symbol(), "┌");
        assert!(row_text(buffer, 77, 118, 1).contains("no events yet"));
    }

    #[test]
    fn event_log_hides_completely_when_disabled() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        let events = events_log(&["Unit 0 moves E"]);
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &view,
                        None,
                        (0, 0),
                        None,
                        None,
                        Duration::ZERO,
                        false,
                        &events,
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_ne!(buffer.cell((76, 0)).unwrap().symbol(), "┌");
        let top_row = row_text(buffer, 76, 119, 0);
        assert!(
            !top_row.contains("EVENTS"),
            "no event log when disabled (got {top_row:?})"
        );
    }

    #[test]
    fn a_unit_is_rendered_at_its_world_position_with_a_centred_camera() {
        let mut engine = crate::game_engine::Engine::new(
            crate::game_engine::DEFAULT_MAP_WIDTH,
            crate::game_engine::DEFAULT_MAP_HEIGHT,
            crate::game_engine::Player::new(Civilization::English),
            vec![],
        );
        engine.populate_starting_world();

        let (fx, fy) = find_settler(&engine);
        // Centre a camera on the settler (a 42x40 tile pane).
        let pane_cols = (120 - LEFT_COLUMN_WIDTH as usize) / 2;
        let pane_rows = 40;
        let camera = camera_center(
            (fx, fy),
            (engine.width(), engine.height()),
            (pane_cols, pane_rows),
        );

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((fx, fy)),
                        camera,
                        None,
                        None,
                        Duration::ZERO,
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();

        // The settler is drawn with its class letter at the 1:1 tile position,
        // in bold to distinguish it from terrain.
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(fx, camera.0, map_w) * 2;
        let py = fy - camera.1;
        let buffer = terminal.backend().buffer();
        let cell = buffer.cell((px as u16, py as u16)).unwrap();
        assert_eq!(cell.symbol(), "S");
        assert!(
            cell.style().add_modifier.contains(Modifier::BOLD),
            "the unit tile should render bold"
        );
        assert!(
            cell.style().add_modifier.contains(Modifier::UNDERLINED),
            "the unit tile should render underlined"
        );
    }

    #[test]
    fn terrain_tiles_are_not_bold() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(&view, None, (0, 0), None, None, Duration::ZERO, false, &[]),
                    frame.area(),
                )
            })
            .unwrap();
        // A tile with no unit/city on it is bare terrain and must not be bold.
        let buffer = terminal.backend().buffer();
        let px = LEFT_COLUMN_WIDTH as usize;
        let py = 1;
        let cell = buffer.cell((px as u16, py as u16)).unwrap();
        assert!(
            !cell.style().add_modifier.contains(Modifier::BOLD),
            "bare terrain tiles should not be bold"
        );
    }

    fn settler_engine() -> (
        crate::game_engine::Engine,
        usize,
        usize,
        UnitId,
        (usize, usize),
    ) {
        let mut engine = crate::game_engine::Engine::new(
            crate::game_engine::DEFAULT_MAP_WIDTH,
            crate::game_engine::DEFAULT_MAP_HEIGHT,
            crate::game_engine::Player::new(Civilization::English),
            vec![],
        );
        engine.populate_starting_world();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|unit| unit.unit_class == crate::model::units::UnitClass::Settler)
            .unwrap();
        let id = unit.id();
        let (fx, fy) = (unit.location.x as usize, unit.location.y as usize);
        let pane_cols = (120 - LEFT_COLUMN_WIDTH as usize) / 2;
        let camera = camera_center((fx, fy), (engine.width(), engine.height()), (pane_cols, 40));
        (engine, fx, fy, id, camera)
    }

    fn render_settler(now: Duration) -> (ratatui::buffer::Cell, crate::game_engine::Engine) {
        let (engine, fx, fy, id, camera) = settler_engine();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((fx, fy)),
                        camera,
                        Some(id),
                        None,
                        now,
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(fx, camera.0, map_w) * 2;
        let py = fy - camera.1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        (cell.clone(), engine)
    }

    #[test]
    fn a_selected_idle_unit_flashes_its_civilizations_colour() {
        let (cell, _engine) = render_settler(Duration::from_millis(200));
        assert_eq!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "an idle selected unit tile should flash the civ colour during the on phase"
        );
    }

    #[test]
    fn a_selected_idle_unit_is_dim_during_the_off_half_second() {
        let (cell, _engine) = render_settler(Duration::from_millis(700));
        assert_ne!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "an idle selected unit tile should not flash the civ colour off-phase"
        );
    }

    #[test]
    fn a_working_unit_does_not_flash() {
        let (mut engine, fx, fy, id, camera) = settler_engine();
        // Busy the settler so it is no longer awaiting instruction.
        engine.submit(crate::game_engine::Command::Work {
            unit: id,
            improvement: crate::model::geography::TerrainImprovement::Road,
        });
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((fx, fy)),
                        camera,
                        Some(id),
                        None,
                        Duration::from_millis(200),
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(fx, camera.0, map_w) * 2;
        let py = fy - camera.1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        assert_ne!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "a working unit should not flash even on-phase"
        );
    }

    #[test]
    fn an_idle_unit_with_no_moves_left_does_not_flash() {
        let (mut engine, _fx, _fy, id, _camera) = settler_engine();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == id)
            .unwrap();
        let before = unit.location;

        // Spend the settler's only move on the first affordable step so it is
        // still idle but has no movement budget left.
        let mut moved = None;
        for direction in [Direction::E, Direction::W, Direction::N, Direction::S] {
            let (dx, dy) = direction.delta();
            let nx = (before.x as isize + dx).clamp(0, engine.width() as isize - 1) as usize;
            let ny = (before.y as isize + dy).clamp(0, engine.height() as isize - 1) as usize;
            let terrain = engine.tile(nx, ny).terrain;
            if terrain.is_land() && terrain.movement_cost() <= 1 {
                moved = Some((nx, ny, direction));
                break;
            }
        }
        let Some((nx, ny, direction)) = moved else {
            return; // no affordable step; nothing legal to assert
        };
        engine.submit(crate::game_engine::Command::Move {
            unit: id,
            direction,
        });

        let camera = camera_center((nx, ny), (engine.width(), engine.height()), (42, 40));
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((nx, ny)),
                        camera,
                        Some(id),
                        None,
                        Duration::from_millis(200),
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(nx, camera.0, map_w) * 2;
        let py = ny - camera.1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        assert_ne!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "an idle unit with zero moves left should not flash even on-phase"
        );
    }

    #[test]
    fn a_city_is_rendered_with_its_population_on_a_civ_coloured_tile() {
        let (mut engine, fx, fy, id, camera) = settler_engine();
        engine.submit(crate::game_engine::Command::FoundCity {
            unit: id,
            name: "London".to_string(),
        });

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((fx, fy)),
                        camera,
                        None,
                        None,
                        Duration::from_millis(200),
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(fx, camera.0, map_w) * 2;
        let py = fy - camera.1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        assert_eq!(
            cell.symbol(),
            "1",
            "a population-1 city should display its population digit"
        );
        assert_eq!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "a city tile should take its civilization's colour"
        );

        // The city's name is centred beneath the tile.
        let name = "London";
        let center_col = (px + 1) as isize;
        let start = center_col - (name.len() / 2) as isize;
        for (i, ch) in name.chars().enumerate() {
            let col = (start + i as isize) as u16;
            let cell = terminal
                .backend()
                .buffer()
                .cell((col, py as u16 + 1))
                .unwrap();
            assert_eq!(
                cell.symbol(),
                ch.to_string(),
                "city name char {i} at column {col}"
            );
            assert_eq!(
                cell.style().fg,
                Some(CITY_LABEL_FG),
                "city name char {i} should be yellow"
            );
        }
    }

    #[test]
    fn the_selected_city_tile_is_outlined() {
        let (mut engine, fx, fy, id, camera) = settler_engine();
        engine.submit(crate::game_engine::Command::FoundCity {
            unit: id,
            name: "London".to_string(),
        });
        let city_id = engine.player_cities().first().unwrap().id();

        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(
                        &engine,
                        Some((fx, fy)),
                        camera,
                        None,
                        Some(city_id),
                        Duration::from_millis(200),
                        false,
                        &[],
                    ),
                    frame.area(),
                )
            })
            .unwrap();
        let map_w = engine.width();
        let px = LEFT_COLUMN_WIDTH as usize + wrapped_col(fx, camera.0, map_w) * 2;
        let py = fy - camera.1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        assert!(
            cell.style().add_modifier.contains(Modifier::UNDERLINED),
            "the selected city's tile should be underlined"
        );
    }

    #[test]
    fn a_city_on_an_unexplored_tile_stays_hidden() {
        let city = crate::model::cities::City::new(
            "Hidden",
            crate::model::cartography::Location::new(0, 0),
            crate::model::civilizations::PlayerId::new(0),
            crate::model::cities::CityId::new(0),
        );
        let view = FakeView {
            w: 80,
            h: 50,
            tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Grassland),
            city: Some(city),
            explored: false,
        };
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GameScreen::new(&view, None, (0, 0), None, None, Duration::ZERO, false, &[]),
                    frame.area(),
                )
            })
            .unwrap();
        let px = LEFT_COLUMN_WIDTH as usize;
        let py = 1;
        let cell = terminal
            .backend()
            .buffer()
            .cell((px as u16, py as u16))
            .unwrap();
        assert_ne!(
            cell.symbol(),
            "1",
            "an undiscovered city must not reveal its population"
        );
        assert_ne!(
            cell.style().bg,
            Some(civilization_color(Civilization::English)),
            "an undiscovered city must not show its civilization colour"
        );
        // No name is drawn beneath the hidden city.
        let mut label = String::new();
        for x in px..(px + 8) {
            let c = terminal
                .backend()
                .buffer()
                .cell((x as u16, py as u16 + 1))
                .unwrap()
                .symbol()
                .to_string();
            label.push_str(&c);
        }
        assert!(
            !label.contains("Hidden"),
            "an undiscovered city must not reveal its name (got {label:?})"
        );
    }

    fn find_settler(engine: &crate::game_engine::Engine) -> (usize, usize) {
        for y in 0..engine.height() {
            for x in 0..engine.width() {
                if engine
                    .units_at(x, y)
                    .iter()
                    .any(|unit| unit.unit_class == crate::model::units::UnitClass::Settler)
                {
                    return (x, y);
                }
            }
        }
        panic!("no settler found in populated world");
    }

    fn camera_center(
        focus: (usize, usize),
        map: (usize, usize),
        pane: (usize, usize),
    ) -> (usize, usize) {
        let left = (focus.0 as isize - (pane.0 / 2) as isize).rem_euclid(map.0 as isize) as usize;
        let top = (focus.1 as isize - (pane.1 / 2) as isize).max(0) as usize;
        let top = map.1.saturating_sub(pane.1).min(top);
        (left, top)
    }

    /// Viewport column of a tile, accounting for horizontal map wrapping.
    fn wrapped_col(fx: usize, camera_x: usize, map_w: usize) -> usize {
        (fx + map_w - camera_x) % map_w
    }
}
