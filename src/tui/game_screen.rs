use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::theme::{ACCENT, DIM, draw_text};
use crate::game_engine::GameView;
use crate::model::units::UnitClass;

/// Fixed width of the left-hand information column.
pub const LEFT_COLUMN_WIDTH: u16 = 36;

const TILE_WIDTH: usize = 2;

fn terrain_colors(terrain: char) -> (Color, Color) {
    match terrain {
        '~' => (Color::Rgb(40, 90, 160), Color::Rgb(20, 40, 80)),
        '.' => (Color::Rgb(185, 205, 110), Color::Rgb(80, 95, 45)),
        'v' => (Color::Rgb(140, 210, 90), Color::Rgb(45, 95, 40)),
        '#' => (Color::Rgb(60, 150, 60), Color::Rgb(25, 70, 30)),
        '^' => (Color::Rgb(150, 130, 90), Color::Rgb(70, 60, 40)),
        'M' => (Color::Rgb(170, 170, 180), Color::Rgb(80, 80, 90)),
        'd' => (Color::Rgb(210, 200, 130), Color::Rgb(110, 100, 50)),
        't' => (Color::Rgb(170, 200, 200), Color::Rgb(70, 90, 90)),
        's' => (Color::Rgb(90, 150, 120), Color::Rgb(40, 80, 60)),
        'T' => (Color::Rgb(60, 160, 80), Color::Rgb(25, 75, 40)),
        _ => (Color::Rgb(120, 200, 90), Color::Rgb(40, 90, 40)),
    }
}

fn tile_style(explored: bool, terrain: char) -> Style {
    if !explored {
        return Style::default()
            .fg(Color::Rgb(20, 20, 20))
            .bg(Color::Rgb(10, 10, 36));
    }
    let (fg, bg) = terrain_colors(terrain);
    Style::default().fg(fg).bg(bg)
}

pub struct GameScreen<'a> {
    view: &'a dyn GameView,
    focus: Option<(usize, usize)>,
    camera: (usize, usize),
}

impl<'a> GameScreen<'a> {
    /// `camera` is the world-tile coordinate at the top-left of the map pane.
    pub fn new(
        view: &'a dyn GameView,
        focus: Option<(usize, usize)>,
        camera: (usize, usize),
    ) -> Self {
        GameScreen {
            view,
            focus,
            camera,
        }
    }

    fn draw_main_map(&self, area: Rect, buf: &mut Buffer) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let cell_cols = area.width as usize / TILE_WIDTH;
        let cell_rows = area.height as usize;
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }

        // Tiles are shown at 1:1; `camera` is the top-left world tile. World
        // tiles outside the map (when scrolled up against an edge) render as
        // void.
        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let src_x = self.camera.0 + col;
                let src_y = self.camera.1 + row;
                let cx = area.x + (col * TILE_WIDTH) as u16;
                let cy = area.y + row as u16;

                let (symbol, style) = if src_x >= map_w || src_y >= map_h {
                    (' ', Style::default().bg(Color::Rgb(6, 6, 22)))
                } else {
                    let tile = self.view.tile(src_x, src_y);
                    let explored = self.view.explored(src_x, src_y);
                    let terrain = tile.geography.as_char();
                    let mut style = tile_style(explored, terrain);

                    let unit = self.view.units_at(src_x, src_y);
                    let city = self.view.city_at(src_x, src_y);

                    let (symbol, uses_city, is_unit) = if let Some(city) = city {
                        (city.name.chars().next().unwrap_or('*'), true, false)
                    } else if let Some(u) = unit.first() {
                        (first_letter(u.unit_class), false, true)
                    } else {
                        (terrain, false, false)
                    };

                    if uses_city {
                        style = style.fg(ACCENT);
                    }
                    if is_unit {
                        style = style
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED);
                    }
                    (symbol, style)
                };

                if let Some(cell) = buf.cell_mut((cx, cy)) {
                    cell.set_symbol(&symbol.to_string());
                    cell.set_style(style);
                }
                if TILE_WIDTH > 1
                    && let Some(cell) = buf.cell_mut((cx + 1, cy))
                {
                    cell.set_symbol(" ");
                    cell.set_style(style);
                }
            }
        }
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
                let terrain = self.view.tile(src_x, src_y).geography.as_char();
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
                    &format!("Terrain: {:?}", tile.geography),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::civilizations::Civilization;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct FakeView {
        w: usize,
        h: usize,
        tile: crate::model::cartography::Tile,
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
            None
        }
        fn player_units(&self) -> Vec<&crate::model::units::Unit> {
            Vec::new()
        }
        fn explored(&self, _x: usize, _y: usize) -> bool {
            true
        }
        fn current_player(&self) -> Civilization {
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
            tile: crate::model::cartography::Tile::new(crate::model::geography::Geography::Ocean),
        }
    }

    fn render() -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let view = fake_view();
        terminal
            .draw(|frame| frame.render_widget(GameScreen::new(&view, None, (0, 0)), frame.area()))
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
                frame.render_widget(GameScreen::new(&engine, focus, (0, 0)), frame.area())
            })
            .unwrap();
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
                    GameScreen::new(&engine, Some((fx, fy)), camera),
                    frame.area(),
                )
            })
            .unwrap();

        // The settler is drawn with its class letter at the 1:1 tile position,
        // in bold to distinguish it from terrain.
        let px = LEFT_COLUMN_WIDTH as usize + (fx - camera.0) * 2;
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
            .draw(|frame| frame.render_widget(GameScreen::new(&view, None, (0, 0)), frame.area()))
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
        let left = (focus.0 as isize - (pane.0 / 2) as isize).max(0) as usize;
        let top = (focus.1 as isize - (pane.1 / 2) as isize).max(0) as usize;
        (
            map.0.saturating_sub(pane.0).min(left),
            map.1.saturating_sub(pane.1).min(top),
        )
    }
}
