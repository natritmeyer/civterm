use super::*;
use crate::model::cartography::Direction;
use crate::model::civilizations::Civilization;
use crate::model::geography::Terrain;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

struct FakeView {
    w: usize,
    h: usize,
    tile: crate::model::cartography::Tile,
    city: Option<crate::model::cities::City>,
    unit: Option<crate::model::units::Unit>,
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
        self.unit.iter().collect()
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
    fn city_income(&self, _id: crate::model::cities::CityId) -> crate::game_engine::CityIncome {
        crate::game_engine::CityIncome {
            food: 2,
            resources: 0,
            trade: 1,
            gold: 0,
            research: 0,
            special_resources: Vec::new(),
        }
    }
    fn home_units(&self, _city: crate::model::cities::CityId) -> Vec<&crate::model::units::Unit> {
        Vec::new()
    }
    fn explored(&self, _x: usize, _y: usize) -> bool {
        self.explored
    }
    fn current_player(&self) -> Civilization {
        Civilization::English
    }
    fn civilization_of(&self, player: crate::model::civilizations::PlayerId) -> Civilization {
        if player == crate::model::civilizations::PlayerId::new(1) {
            Civilization::Zulu
        } else {
            Civilization::English
        }
    }
    fn turn(&self) -> u32 {
        1
    }
    fn year(&self) -> i32 {
        -4000
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
    fn production_choices(
        &self,
        _city: crate::model::cities::CityId,
    ) -> Vec<crate::model::cities::ProductionTarget> {
        Vec::new()
    }
}

fn fake_view() -> FakeView {
    FakeView {
        w: 80,
        h: 50,
        tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Ocean),
        city: None,
        unit: None,
        explored: true,
    }
}

fn render() -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let view = fake_view();
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
                    &[],
                    None,
                ),
                frame.area(),
            )
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn years_are_formatted_with_a_bc_and_ad_boundary() {
    assert_eq!(format_year(-4000), "4000 BC");
    assert_eq!(format_year(-1), "1 BC");
    assert_eq!(format_year(0), "1 AD");
    assert_eq!(format_year(50), "50 AD");
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
                    None,
                ),
                frame.area(),
            )
        })
        .unwrap();
}

#[test]
fn the_hovered_tile_is_shaded_while_its_neighbours_are_not() {
    let unit = crate::model::units::Unit::new(
        crate::model::units::UnitClass::Militia,
        crate::model::cartography::Location::new(0, 0),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
        crate::model::units::UnitId::new(7),
    );
    let view = FakeView {
        w: 80,
        h: 50,
        tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Plains),
        city: None,
        unit: Some(unit),
        explored: true,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                GameScreen::new(
                    &view,
                    Some((0, 0)),
                    (0, 0),
                    Some(crate::model::units::UnitId::new(7)),
                    None,
                    // The mid-phase `now` turns the idle-unit flash off, so
                    // the hovered tile keeps its plain terrain background.
                    Duration::from_millis(600),
                    false,
                    &[],
                    Some((1, 0)),
                ),
                frame.area(),
            )
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let base = LEFT_COLUMN_WIDTH;
    let expected_bg = tile_style(true, crate::model::geography::Terrain::Plains).bg;
    for (cell_x, cell_y) in [(base + 2, 0), (base + 3, 0)] {
        let cell = buffer.cell((cell_x, cell_y)).unwrap();
        assert_eq!(cell.symbol(), "▓", "hovered tile cells should be shaded");
        assert_eq!(
            cell.style().bg,
            expected_bg,
            "the hover shade must keep the tile's terrain background"
        );
    }
    // Neighbouring tiles keep their own symbols.
    for cell_x in [base + 4, base + 5, base + 6, base + 8] {
        let cell = buffer.cell((cell_x, 0)).unwrap();
        assert_ne!(cell.symbol(), "▓", "non-hovered tile must stay untouched");
    }
}

fn events_log(events: &[&str]) -> Vec<Event> {
    events.iter().map(|m| Event::new(*m)).collect()
}

fn row_text(buf: &Buffer, x0: u16, x1: u16, y: u16) -> String {
    (x0..=x1)
        .map(|x| buf.cell((x, y)).unwrap().symbol().chars().next().unwrap())
        .collect()
}

/// Paints one world tile into a scratch buffer via `paint_tile` and returns
/// the drawn cell (symbol + style) and the city-name label it yielded.
fn painted_cell(
    view: &dyn GameView,
    selected_unit: Option<crate::model::units::UnitId>,
    flashing: bool,
) -> (ratatui::buffer::Cell, Option<String>) {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
    let name = paint_tile(
        &mut buf,
        0,
        0,
        view,
        2,
        2,
        80,
        50,
        None,
        selected_unit,
        flashing,
        None,
    );
    (buf.cell((0, 0)).unwrap().clone(), name)
}

/// Paints one world tile into a scratch buffer via `paint_tile`, keeping
/// the whole tile so tests can read the improvement watermark in the spare
/// column.
fn painted_buffer(
    view: &dyn GameView,
    selected_unit: Option<crate::model::units::UnitId>,
    flashing: bool,
) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
    paint_tile(
        &mut buf,
        0,
        0,
        view,
        2,
        2,
        80,
        50,
        None,
        selected_unit,
        flashing,
        None,
    );
    buf
}

fn city_and_unit_view() -> FakeView {
    let mut view = fake_view();
    view.city = Some(crate::model::cities::City::new(
        "London",
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
    ));
    view.unit = Some(crate::model::units::Unit::new(
        crate::model::units::UnitClass::Militia,
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
        crate::model::units::UnitId::new(1),
    ));
    view
}

#[test]
fn a_unit_on_a_city_tile_displays_ahead_of_the_population() {
    let view = city_and_unit_view();
    let (cell, name) = painted_cell(&view, None, false);
    assert_eq!(cell.symbol(), "M");
    // The city still yields its name label even though a unit is shown.
    assert_eq!(name.as_deref(), Some("London"));
}

#[test]
fn a_unit_in_a_city_stands_on_the_citys_owner_colour() {
    // The same militiaman is painted once alone and once standing in the
    // city; the two tiles share symbol and emphasis, but the city tile
    // wears the city owner's colour rather than the terrain's.
    let mut plain = fake_view();
    plain.unit = Some(crate::model::units::Unit::new(
        crate::model::units::UnitClass::Militia,
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
        crate::model::units::UnitId::new(1),
    ));
    let (plain_cell, plain_name) = painted_cell(&plain, None, false);

    let view = city_and_unit_view();
    let (city_cell, city_name) = painted_cell(&view, None, false);

    assert_eq!(city_cell.symbol(), plain_cell.symbol());
    assert!(city_cell.style().add_modifier.contains(Modifier::BOLD));
    assert!(
        city_cell
            .style()
            .add_modifier
            .contains(Modifier::UNDERLINED)
    );
    // The tile shows the city's owner colour, not the unit's terrain: it
    // belongs to English in this view, while a plain unit keeps the terrain.
    assert_eq!(
        city_cell.style().bg,
        Some(civilization_color(Civilization::English))
    );
    assert_ne!(plain_cell.style().bg, city_cell.style().bg);
    // The city is still identified below the unit.
    assert_eq!(plain_name, None);
    assert_eq!(city_name.as_deref(), Some("London"));
}

#[test]
fn a_captured_city_changes_colour_beneath_the_conquering_unit() {
    // A Zulu city (player 1) occupied by the attacking English militia
    // still shows Zulu's colour while it remains foreign.
    let mut foreign = fake_view();
    foreign.city = Some(crate::model::cities::City::new(
        "Glasgow",
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(1),
        crate::model::cities::CityId::new(2),
    ));
    foreign.unit = Some(crate::model::units::Unit::new(
        crate::model::units::UnitClass::Militia,
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(2),
        crate::model::units::UnitId::new(3),
    ));
    let (occupied, _) = painted_cell(&foreign, None, false);
    assert_eq!(
        occupied.style().bg,
        Some(civilization_color(Civilization::Zulu)),
        "an occupied but unconquered city keeps its old owner's colour"
    );

    // The instant the same city is captured (its owner becomes player 0)
    // its tile turns English while the victorious unit still stands on it.
    foreign.city = Some(crate::model::cities::City::new(
        "Glasgow",
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(2),
    ));
    let (conquered, _) = painted_cell(&foreign, None, false);
    assert_eq!(
        conquered.style().bg,
        Some(civilization_color(Civilization::English)),
        "a conquered city turns to the conqueror's colour immediately"
    );
}

#[test]
fn a_selected_idle_unit_in_a_city_keeps_the_city_colour_when_dimmed() {
    let idle = crate::model::units::UnitId::new(1);
    let view = city_and_unit_view();

    // Flashing: the tile turns the civilization flag colour.
    let (flashing, _) = painted_cell(&view, Some(idle), true);
    assert_eq!(
        flashing.style().bg,
        Some(civilization_color(Civilization::English))
    );

    // Dimmed again, the city's own colour shows through beneath the unit
    // (the same English flag colour, since the unit sits in its own city).
    let (dimmed, _) = painted_cell(&view, Some(idle), false);
    assert_eq!(
        dimmed.style().bg,
        Some(civilization_color(Civilization::English))
    );
}

fn improved_view() -> FakeView {
    let mut view = fake_view();
    view.tile = crate::model::cartography::Tile::new(Terrain::Grassland);
    view.tile.irrigate().unwrap();
    view.tile.build_road().unwrap();
    view
}

#[test]
fn an_irrigated_tile_shows_a_watermark_in_its_spare_column() {
    let view = improved_view();
    let (tile, _) = painted_cell(&view, None, false);
    assert_eq!(
        tile.symbol(),
        "v",
        "the terrain letter keeps the first column"
    );
    let buf = painted_buffer(&view, None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "≈");
    // The marker keeps the tile's background so the tile still reads whole.
    assert_eq!(buf.cell((1, 0)).unwrap().style().bg, tile.style().bg);
}

#[test]
fn an_irrigated_tile_with_a_road_shows_the_irrigation_watermark() {
    // Both improvements are present; the most informative one wins.
    let (_, name) = painted_cell(&improved_view(), None, false);
    assert_eq!(name, None);
    let buf = painted_buffer(&improved_view(), None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "≈");
}

#[test]
fn a_mine_and_a_road_each_show_their_own_watermark() {
    let mut mined = fake_view();
    mined.tile = crate::model::cartography::Tile::new(Terrain::Hills);
    mined.tile.mine().unwrap();
    let buf = painted_buffer(&mined, None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "⛏");

    let mut roaded = fake_view();
    roaded.tile = crate::model::cartography::Tile::new(Terrain::Plains);
    roaded.tile.build_road().unwrap();
    let buf = painted_buffer(&roaded, None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "+");
}

#[test]
fn an_plain_tile_and_fog_draw_no_watermark() {
    let plain = fake_view();
    let buf = painted_buffer(&plain, None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), " ");

    // Fog hides the improvement even though the tile owns one.
    let mut foggy = improved_view();
    foggy.explored = false;
    let buf = painted_buffer(&foggy, None, false);
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), " ");
}

#[test]
fn a_city_with_no_unit_shows_its_population() {
    let mut view = fake_view();
    view.city = Some(crate::model::cities::City::new(
        "London",
        crate::model::cartography::Location::new(2, 2),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
    ));
    let (cell, name) = painted_cell(&view, None, false);
    assert_eq!(cell.symbol(), "1");
    assert_eq!(name.as_deref(), Some("London"));
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
                    None,
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
                    None,
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
                GameScreen::new(
                    &view,
                    None,
                    (0, 0),
                    None,
                    None,
                    Duration::ZERO,
                    true,
                    &[],
                    None,
                ),
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
                    None,
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
                    None,
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
                GameScreen::new(
                    &view,
                    None,
                    (0, 0),
                    None,
                    None,
                    Duration::ZERO,
                    false,
                    &[],
                    None,
                ),
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
                    None,
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
                    None,
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
                    None,
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

/// Renders the map with a battle flash on world tile (2, 2) that started at
/// `start`, observed at clock `now`, and returns the two cells of that tile
/// plus a neighbour tile's left cell so tests can prove the flash is local.
fn render_battle_flash(now: Duration, start: Duration) -> (Buffer, u16, u16) {
    let view = FakeView {
        w: 80,
        h: 50,
        tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Plains),
        city: None,
        unit: None,
        explored: true,
    };
    let animation = BattleAnimation {
        location: crate::model::cartography::Location::new(2, 2),
        start,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                GameScreen::new(&view, None, (0, 0), None, None, now, false, &[], None)
                    .with_battle_animation(Some(animation)),
                frame.area(),
            )
        })
        .unwrap();
    let base = LEFT_COLUMN_WIDTH + 4; // tile (2, 2)'s left column on screen
    (terminal.backend().buffer().clone(), base, 2)
}

#[test]
fn the_flash_glyph_lasts_for_the_whole_flash_duration() {
    let animation = BattleAnimation {
        location: crate::model::cartography::Location::new(0, 0),
        start: Duration::from_millis(100),
    };
    assert_eq!(animation.glyph(Duration::from_millis(100)), Some("💥"));
    assert_eq!(animation.glyph(Duration::from_millis(599)), Some("💥"));
    assert_eq!(animation.glyph(Duration::from_millis(600)), Some("💥"));
    assert_eq!(animation.glyph(Duration::from_millis(1099)), Some("💥"));
}

#[test]
fn the_flash_glyph_is_gone_once_a_full_second_has_passed() {
    let animation = BattleAnimation {
        location: crate::model::cartography::Location::new(0, 0),
        start: Duration::from_millis(100),
    };
    assert_eq!(animation.glyph(Duration::from_millis(1100)), None);
}

#[test]
fn the_flash_shows_the_explosion_in_the_defended_tiles_left_column() {
    let (buffer, left, row) = render_battle_flash(Duration::from_millis(200), Duration::ZERO);
    assert_eq!(
        buffer.cell((left, row)).unwrap().symbol(),
        "💥",
        "the explosion shows in the left column"
    );
    assert_eq!(
        buffer.cell((left + 1, row)).unwrap().symbol(),
        " ",
        "the right column is cleared while the explosion is showing"
    );
    assert_eq!(
        buffer.cell((left + 2, row)).unwrap().symbol(),
        ".",
        "the neighbouring tile is untouched"
    );
    assert_eq!(
        buffer.cell((left, row + 1)).unwrap().symbol(),
        ".",
        "the tile below is untouched"
    );
}

#[test]
fn the_flash_stays_on_the_defended_tile_in_the_second_half_second() {
    let (buffer, left, row) = render_battle_flash(Duration::from_millis(600), Duration::ZERO);
    // Only the left column is a legitimate anchor for the double-width
    // explosion: drawn one cell right it would cover the neighbour tile.
    assert_eq!(
        buffer.cell((left, row)).unwrap().symbol(),
        "💥",
        "the explosion stays anchored on the defended tile"
    );
    assert_eq!(
        buffer.cell((left + 1, row)).unwrap().symbol(),
        " ",
        "the right column is clear, so the wide glyph cannot spill east"
    );
    assert_eq!(
        buffer.cell((left + 2, row)).unwrap().symbol(),
        ".",
        "the tile to the east is untouched"
    );
}

#[test]
fn the_flash_is_painted_on_top_of_city_labels() {
    // A city tile on the row above paints its name label across the row
    // where the battle is raging; the flash is drawn last, so its cells
    // win over what the label pass wrote first.
    let mut view = fake_view();
    view.city = Some(crate::model::cities::City::new(
        "London",
        crate::model::cartography::Location::new(1, 1),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
    ));
    let animation = BattleAnimation {
        location: crate::model::cartography::Location::new(2, 2),
        start: Duration::ZERO,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                GameScreen::new(
                    &view,
                    None,
                    (0, 0),
                    None,
                    None,
                    Duration::from_millis(200),
                    false,
                    &[],
                    None,
                )
                .with_battle_animation(Some(animation)),
                frame.area(),
            )
        })
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let left = LEFT_COLUMN_WIDTH + 4; // tile (2, 2)'s left column
    let row = 2;
    // The label for the city above spans these columns; the explosion still
    // owns the flash tile's two cells.
    assert_eq!(buffer.cell((left, row)).unwrap().symbol(), "💥");
    assert_eq!(buffer.cell((left + 1, row)).unwrap().symbol(), " ");
}

#[test]
fn the_flash_disappears_and_the_tile_returns_to_normal() {
    let (buffer, left, row) = render_battle_flash(Duration::from_millis(1000), Duration::ZERO);
    assert_eq!(
        buffer.cell((left, row)).unwrap().symbol(),
        ".",
        "once the flash ends the tile shows its plain terrain again"
    );
    assert_eq!(
        buffer.cell((left + 1, row)).unwrap().symbol(),
        " ",
        "the spare column is blank again once the flash ends"
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
                    None,
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
                    None,
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
        unit: None,
        explored: false,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
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
                    &[],
                    None,
                ),
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
    assert_eq!(
        cell.symbol(),
        " ",
        "an undiscovered tile must be blank fog, not its terrain"
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

#[test]
fn a_unit_on_an_unexplored_tile_stays_hidden() {
    let unit = crate::model::units::Unit::new(
        crate::model::units::UnitClass::Knight,
        crate::model::cartography::Location::new(0, 0),
        crate::model::civilizations::PlayerId::new(0),
        crate::model::cities::CityId::new(0),
        crate::model::units::UnitId::new(0),
    );
    let view = FakeView {
        w: 80,
        h: 50,
        tile: crate::model::cartography::Tile::new(crate::model::geography::Terrain::Plains),
        city: None,
        unit: Some(unit),
        explored: false,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
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
                    &[],
                    None,
                ),
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
    assert_eq!(
        cell.symbol(),
        " ",
        "an undiscovered unit must not reveal its class letter"
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
