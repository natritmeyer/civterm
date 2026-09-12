use super::*;
use crate::game_engine::Command;
use crate::model::cartography::Location;
use crate::model::cities::ProductionTarget;
use crate::model::geography::TerrainImprovement;
use crate::model::units::UnitOrder;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use strum::IntoEnumIterator;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn pressing_q_in_the_menu_exits() {
    for code in [KeyCode::Char('q'), KeyCode::Char('Q'), KeyCode::Esc] {
        let mut app = App::new();
        assert!(app.handle_key(key(code)));
    }
}

#[test]
fn new_game_opens_the_civ_selector() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Char('n')));
    assert!(matches!(app.phase, Phase::ChoosingCiv));
    let mut app = App::new();
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.phase, Phase::ChoosingCiv));
}

#[test]
fn load_only_highlights_the_item() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Char('l')));
    assert_eq!(app.selected, 1);
    assert!(matches!(app.phase, Phase::Menu));
}

#[test]
fn arrows_move_the_menu_selection() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.selected, 1);
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.selected, 2);
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.selected, 0);
    app.handle_key(key(KeyCode::Left));
    assert_eq!(app.selected, 2);
}

#[test]
fn enter_on_quit_exits() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Right));
    app.handle_key(key(KeyCode::Right));
    assert!(app.handle_key(key(KeyCode::Enter)));
}

#[test]
fn civ_arrows_and_j_k_move_the_selection() {
    let mut app = App::new();
    app.start_new_game();
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.civ_index, 1);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.civ_index, 2);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.civ_index, 1);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.civ_index, 0);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.civ_index, Civilization::iter().count() - 1);
}

#[test]
fn choosing_a_civ_stores_it_and_moves_to_competition() {
    let mut app = App::new();
    app.start_new_game();
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.chosen_civ, Some(Civilization::Babylonian));
    assert!(matches!(app.phase, Phase::ChoosingCompetition));
}

fn at_competition(app: &mut App) {
    app.start_new_game();
    app.handle_key(key(KeyCode::Enter)); // accept default civ, on to competition
}

fn at_difficulty(app: &mut App) {
    at_competition(app);
    app.handle_key(key(KeyCode::Enter)); // accept default competition, on to difficulty
}

fn at_start(app: &mut App) {
    at_difficulty(app);
    app.handle_key(key(KeyCode::Enter)); // accept default difficulty, on to start prompt
}

#[test]
fn competition_arrows_and_j_k_move_the_selection() {
    let mut app = App::new();
    at_competition(&mut app);
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.competition_index, 1);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.competition_index, 2);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.competition_index, 1);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.competition_index, 0);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(
        app.competition_index,
        (Competition::MAX - Competition::MIN) as usize
    );
}

#[test]
fn choosing_a_competition_level_stores_it_and_moves_to_difficulty() {
    let mut app = App::new();
    at_competition(&mut app);
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.chosen_competition, Some(Competition::new(3)));
    assert!(matches!(app.phase, Phase::ChoosingDifficulty));
}

#[test]
fn esc_from_competition_returns_to_the_civ_selector() {
    let mut app = App::new();
    at_competition(&mut app);
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::ChoosingCiv));
    assert_eq!(app.chosen_civ, Some(Civilization::American));
}

#[test]
fn q_from_competition_returns_to_the_menu() {
    for code in [KeyCode::Char('q'), KeyCode::Char('Q')] {
        let mut app = App::new();
        at_competition(&mut app);
        assert!(!app.handle_key(key(code)));
        assert!(matches!(app.phase, Phase::Menu));
    }
}

#[test]
fn difficulty_arrows_and_j_k_move_the_selection() {
    let mut app = App::new();
    at_difficulty(&mut app);
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.difficulty_index, 1);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.difficulty_index, 2);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.difficulty_index, 1);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.difficulty_index, 0);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.difficulty_index, Difficulty::iter().count() - 1);
}

#[test]
fn choosing_a_difficulty_stores_it_and_advances_to_start() {
    let mut app = App::new();
    at_difficulty(&mut app);
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.chosen_difficulty, Some(Difficulty::Normal));
    assert!(matches!(app.phase, Phase::ReadyToStart));
}

#[test]
fn esc_from_difficulty_returns_to_the_competition_selector() {
    let mut app = App::new();
    at_difficulty(&mut app);
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::ChoosingCompetition));
    assert_eq!(app.chosen_civ, Some(Civilization::American));
}

#[test]
fn q_from_difficulty_returns_to_the_menu() {
    for code in [KeyCode::Char('q'), KeyCode::Char('Q')] {
        let mut app = App::new();
        at_difficulty(&mut app);
        assert!(!app.handle_key(key(code)));
        assert!(matches!(app.phase, Phase::Menu));
    }
}

#[test]
fn starting_a_new_game_resets_setup() {
    let mut app = App::new();
    start_a_full_setup(&mut app);
    assert_eq!(app.chosen_competition, Some(Competition::new(1)));
    assert_eq!(app.chosen_difficulty, Some(Difficulty::Normal));
    app.start_new_game();
    assert_eq!(app.chosen_competition, None);
    assert_eq!(app.competition_index, 0);
    assert_eq!(app.chosen_difficulty, None);
    assert_eq!(app.difficulty_index, 0);
    assert_eq!(app.chosen_civ, None);
    assert!(matches!(app.phase, Phase::ChoosingCiv));
}

fn start_a_full_setup(app: &mut App) {
    at_difficulty(app);
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
}

#[test]
fn esc_from_start_prompt_returns_to_difficulty() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::ChoosingDifficulty));
    assert_eq!(app.chosen_difficulty, Some(Difficulty::Easy));
    assert!(app.engine.is_none());
}

#[test]
fn q_from_start_prompt_exits() {
    for code in [KeyCode::Char('q'), KeyCode::Char('Q')] {
        let mut app = App::new();
        at_start(&mut app);
        assert!(app.handle_key(key(code)));
    }
}

#[test]
fn arrows_and_j_k_move_the_start_selection() {
    let mut app = App::new();
    at_start(&mut app);
    assert_eq!(app.start_choice, StartChoice::Start);
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.start_choice, StartChoice::Quit);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.start_choice, StartChoice::Start);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.start_choice, StartChoice::Quit);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.start_choice, StartChoice::Start);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.start_choice, StartChoice::Quit);
}

#[test]
fn enter_on_the_selected_start_option_begins() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Enter));
    assert!(app.engine.is_some());
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn enter_on_quit_exits_from_the_start_prompt() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Down));
    assert!(app.handle_key(key(KeyCode::Enter)));
}

#[test]
fn s_from_start_prompt_creates_the_engine_and_clears_setup() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(app.engine.is_some());
    assert_eq!(app.chosen_civ, None);
    assert_eq!(app.chosen_competition, None);
    assert_eq!(app.chosen_difficulty, None);
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn q_or_esc_from_the_game_returns_to_the_menu() {
    for code in [KeyCode::Char('q'), KeyCode::Esc] {
        let mut app = App::new();
        at_start(&mut app);
        app.handle_key(key(KeyCode::Char('s')));
        assert!(matches!(app.phase, Phase::Playing));
        app.handle_key(key(code));
        assert!(matches!(app.phase, Phase::Menu));
    }
}

#[test]
fn question_mark_toggles_the_command_help_bar() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(!app.show_help);
    app.handle_key(key(KeyCode::Char('?')));
    assert!(app.show_help);
    app.handle_key(key(KeyCode::Char('?')));
    assert!(!app.show_help);
}

#[test]
fn e_toggles_the_event_log_overlay() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(!app.show_events);
    app.handle_key(key(KeyCode::Char('e')));
    assert!(app.show_events);
    app.handle_key(key(KeyCode::Char('e')));
    assert!(!app.show_events);
}

#[test]
fn ending_turns_records_events_up_to_the_log_size() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(app.event_log.is_empty());

    // End the turn past all rivals until the log has overflowed several
    // times; it must stay bounded at EVENT_LOG_SIZE messages.
    for _ in 0..8 {
        app.handle_key(key(KeyCode::Char(' ')));
    }
    assert_eq!(app.event_log.len(), EVENT_LOG_SIZE);
    assert!(
        app.event_log
            .iter()
            .all(|event| !event.message().is_empty()),
        "every recorded event should have a message"
    );
}

#[test]
fn a_brand_new_game_starts_with_an_empty_event_log() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(app.event_log.is_empty());
}

#[test]
fn starting_the_game_selects_the_players_first_unit() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        app.selected_unit,
        engine.player_units().first().map(|unit| unit.id())
    );
}

#[cfg(test)]
impl App {
    fn left_click(&mut self, column: u16, row: u16) {
        self.handle_mouse(mouse_event(
            MouseEventKind::Down(MouseButton::Left),
            column,
            row,
        ));
        self.handle_mouse(mouse_event(
            MouseEventKind::Up(MouseButton::Left),
            column,
            row,
        ));
    }
}

fn playing_app() -> (App, usize, usize) {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    app.handle_key(key(KeyCode::Char('v'))); // found the starting city
    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    let (cx, cy) = {
        let engine = app.engine.as_ref().unwrap();
        let cities = engine.player_cities();
        let city = cities.first().expect("player has a city");
        (city.location.x as usize, city.location.y as usize)
    };
    (app, cx, cy)
}

#[test]
fn left_clicking_a_player_city_selects_it() {
    let (mut app, cx, cy) = playing_app();
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    let city = app
        .engine
        .as_ref()
        .unwrap()
        .player_cities()
        .into_iter()
        .find(|c| c.location.x as usize == cx && c.location.y as usize == cy)
        .unwrap();
    assert_eq!(app.selected_city, Some(city.id()));
}

#[test]
fn clicking_an_empty_tile_clears_the_selection() {
    let (mut app, cx, cy) = playing_app();
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    assert!(app.selected_city.is_some());

    let (ex, ey) = {
        let engine = app.engine.as_ref().unwrap();
        (0..engine.height())
            .flat_map(|y| (0..engine.width()).map(move |x| (x, y)))
            .find(|(x, y)| engine.city_at(*x, *y).is_none())
            .unwrap()
    };
    app.left_click((LEFT_COLUMN_WIDTH as usize + ex * 2) as u16, ey as u16);
    assert_eq!(app.selected_city, None);
}

#[test]
fn clicking_the_left_column_clears_the_selection() {
    let (mut app, cx, cy) = playing_app();
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    assert!(app.selected_city.is_some());
    app.left_click(4, cy as u16);
    assert_eq!(app.selected_city, None);
}

#[test]
fn non_left_clicks_leave_the_selection_alone() {
    let (mut app, cx, cy) = playing_app();
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: (LEFT_COLUMN_WIDTH as usize + cx * 2) as u16,
        row: cy as u16,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    assert_eq!(app.selected_city, None);
}

#[test]
fn clicking_an_adjacent_tile_moves_the_selected_unit_there() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s'))); // begin the game
    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    let (ux, uy) = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine.player_units()[0];
        (unit.location.x as usize, unit.location.y as usize)
    };
    // Pick an adjacent tile the unit can move onto.
    let (world_x, world_y) = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ]
    .into_iter()
    .map(|(dx, dy)| {
        let engine = app.engine.as_ref().unwrap();
        let w = engine.width() as isize;
        let h = engine.height() as isize;
        let nx = (ux as isize + dx).rem_euclid(w) as usize;
        let ny = (uy as isize + dy).clamp(0, h - 1) as usize;
        (nx, ny)
    })
    .find(|&(nx, ny)| {
        let engine = app.engine.as_ref().unwrap();
        let terrain = engine.tile(nx, ny).terrain;
        terrain.is_land() && terrain.movement_cost() <= 1
    })
    .expect("some adjacent tile is passable");
    let column = (LEFT_COLUMN_WIDTH as usize + world_x * TILE_WIDTH) as u16;
    app.left_click(column, world_y as u16);
    let engine = app.engine.as_ref().unwrap();
    let moved = engine.player_units()[0];
    assert_eq!(moved.location.x as usize, world_x);
    assert_eq!(moved.location.y as usize, world_y);
}

#[test]
fn clicking_a_remote_tile_does_not_move_the_selected_unit() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    let before = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine.player_units()[0];
        (unit.location.x as usize, unit.location.y as usize)
    };
    // A tile two squares east of the unit is not adjacent, so the click
    // falls through to the plain city-selection handling.
    let w = app.engine.as_ref().unwrap().width();
    let (wx, wy) = ((before.0 + 2) % w, before.1);
    app.left_click(
        (LEFT_COLUMN_WIDTH as usize + wx * TILE_WIDTH) as u16,
        wy as u16,
    );
    let engine = app.engine.as_ref().unwrap();
    let after = engine.player_units()[0];
    assert_eq!(
        (after.location.x as usize, after.location.y as usize),
        before
    );
}

#[test]
fn adjacent_direction_takes_the_horizontal_wrap_into_account() {
    assert_eq!(adjacent_direction((5, 5), (4, 5), 80), Some(Direction::W));
    assert_eq!(adjacent_direction((5, 5), (6, 5), 80), Some(Direction::E));
    assert_eq!(adjacent_direction((5, 5), (5, 6), 80), Some(Direction::S));
    assert_eq!(adjacent_direction((5, 5), (6, 4), 80), Some(Direction::NE));
    assert_eq!(adjacent_direction((5, 5), (4, 6), 80), Some(Direction::SW));
    assert_eq!(adjacent_direction((0, 5), (79, 5), 80), Some(Direction::W));
    assert_eq!(adjacent_direction((79, 5), (0, 5), 80), Some(Direction::E));
    assert_eq!(adjacent_direction((5, 5), (7, 5), 80), None);
    assert_eq!(adjacent_direction((5, 5), (5, 5), 80), None);
    assert_eq!(adjacent_direction((5, 5), (5, 3), 80), None);
}

#[test]
fn hovering_an_adjacent_tile_reports_the_move_target() {
    // Camera at origin, unit at (5, 5). The pointer is over the tile one
    // square west of the unit.
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 4 * TILE_WIDTH) as u16, 5),
            (0, 0),
            (80, 50),
            Some((5, 5)),
        ),
        Some((4, 5)),
    );
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 6 * TILE_WIDTH) as u16, 4),
            (0, 0),
            (80, 50),
            Some((5, 5)),
        ),
        Some((6, 4)),
    );
    // Hovering the unit's own tile, a distant tile, the left column, the
    // area below the map, or without a selected unit yields no target.
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 5 * TILE_WIDTH) as u16, 5),
            (0, 0),
            (80, 50),
            Some((5, 5)),
        ),
        None,
    );
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 9 * TILE_WIDTH) as u16, 5),
            (0, 0),
            (80, 50),
            Some((5, 5)),
        ),
        None,
    );
    assert_eq!(
        hovered_adjacent_tile((4, 5), (0, 0), (80, 50), Some((5, 5))),
        None
    );
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 4 * TILE_WIDTH) as u16, 60),
            (0, 0),
            (80, 50),
            Some((5, 5)),
        ),
        None,
    );
    assert_eq!(
        hovered_adjacent_tile(
            ((LEFT_COLUMN_WIDTH as usize + 4 * TILE_WIDTH) as u16, 5),
            (0, 0),
            (80, 50),
            None,
        ),
        None,
    );
}

#[test]
fn a_new_game_starts_with_no_city_selected() {
    let (app, _, _) = playing_app();
    assert_eq!(app.selected_city, None);
}

/// Selects the player's city and pretends the city window was drawn, so
/// its mouse hit-testing is live.
fn with_city_window_open() -> (App, usize, usize, Rect) {
    let (mut app, cx, cy) = playing_app();
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    assert!(app.selected_city.is_some());
    let win = Rect {
        x: 20,
        y: 5,
        width: 60,
        height: 30,
    };
    app.moused_window
        .set(Some((win, crate::tui::city_window::close_button_rect(win))));
    (app, cx, cy, win)
}

/// Opens the production picker over the open city window and pretends the
/// panel was drawn, so its mouse hit-testing is live.
fn with_production_picker_open() -> (App, Rect, u16) {
    let (mut app, _, _, win) = with_city_window_open();
    let change = crate::tui::city_window::change_button_rect(
        crate::tui::city_window::production_panel_rect(win),
    );
    app.left_click(change.x + 1, change.y);
    assert!(app.production_picker_open);
    let panel = crate::tui::production_picker::picker_rect(win);
    app.picker_rect.set(Some(panel));
    (app, panel, change.y)
}

#[test]
fn clicking_the_close_button_closes_the_window() {
    let (mut app, _, _, win) = with_city_window_open();
    let close = crate::tui::city_window::close_button_rect(win);
    app.left_click(close.x + 1, close.y);
    assert_eq!(app.selected_city, None);
}

#[test]
fn clicks_inside_the_window_are_consumed() {
    let (mut app, _, _, win) = with_city_window_open();
    let city = app.selected_city;
    app.left_click(win.x + 2, win.y + 2);
    assert_eq!(
        app.selected_city, city,
        "in-window click must not reach the map"
    );
}

#[test]
fn clicks_outside_the_window_reach_the_map() {
    let (mut app, cx, cy, win) = with_city_window_open();
    // The map's left column is outside the window; clicking it still
    // clears the selection rather than being consumed.
    assert!(!win.contains((4, cy as u16).into()));
    app.left_click(4, cy as u16);
    assert_eq!(app.selected_city, None);
    let _ = cx;
}

#[test]
fn wheel_events_scroll_the_open_window() {
    let (mut app, _, _, _) = with_city_window_open();
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 0,
        row: 0,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    assert_eq!(app.city_window_scroll, 1);
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 0,
        row: 0,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    assert_eq!(app.city_window_scroll, 0);
}

#[test]
fn esc_closes_the_window_before_anything_else() {
    let (mut app, _, _, _) = with_city_window_open();
    app.show_help = true;
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.selected_city, None);
    assert!(matches!(app.phase, Phase::Playing));
    // With the window gone, Esc falls through to help then the menu.
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.show_help);
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::Menu));
}

#[test]
fn clicking_the_change_button_opens_the_picker() {
    let (mut app, _, _, win) = with_city_window_open();
    let change = crate::tui::city_window::change_button_rect(
        crate::tui::city_window::production_panel_rect(win),
    );
    assert!(!app.production_picker_open);
    app.left_click(change.x + 1, change.y);
    assert!(app.production_picker_open);
    assert_eq!(app.picker_cursor_col, 0);
    assert_eq!(app.picker_cursor_row, 0);
    assert_eq!(app.picker_scroll, 0);
    // The city window stays open beneath the picker.
    assert!(app.selected_city.is_some());
}

#[test]
fn clicking_a_picker_row_moves_the_single_cursor() {
    let (mut app, panel, _) = with_production_picker_open();
    let (units_col, improvements_col) = crate::tui::production_picker::column_rects(panel);
    let rows = crate::tui::production_picker::rows_rect(panel);
    app.left_click(units_col.x + 2, rows.y + 1);
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (0, 1));
    // Clicking an improvement row moves the cursor over to that list.
    app.left_click(improvements_col.x + 2, rows.y);
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (1, 0));
    let target = app.current_picker_target().expect("cursor has a target");
    assert!(matches!(target, ProductionTarget::Improvement(_)));
}

#[test]
fn picker_keyboard_moves_the_cursor_and_wraps() {
    let (mut app, _, _) = with_production_picker_open();
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (0, 1));
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (0, 0)); // wrapped
    app.handle_key(key(KeyCode::Char('l')));
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (1, 0));
    app.handle_key(key(KeyCode::Char('l'))); // only one improvement list
    assert_eq!(app.picker_cursor_col, 1);
    app.handle_key(key(KeyCode::Char('h')));
    assert_eq!((app.picker_cursor_col, app.picker_cursor_row), (0, 0));
}

#[test]
fn escaping_closes_the_picker_before_the_window_without_saving() {
    let (mut app, _, _) = with_production_picker_open();
    app.handle_key(key(KeyCode::Down));
    let before = app
        .engine
        .as_ref()
        .unwrap()
        .city(app.selected_city.unwrap())
        .unwrap()
        .production_target();
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.production_picker_open);
    assert!(app.selected_city.is_some(), "window should stay open");
    let city = app
        .engine
        .as_ref()
        .unwrap()
        .city(app.selected_city.unwrap())
        .unwrap();
    assert_eq!(city.production_target(), before);
    // One more Esc now closes the window.
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.selected_city, None);
}

#[test]
fn enter_saves_the_selected_target_and_closes_the_picker() {
    let (mut app, _, _) = with_production_picker_open();
    app.handle_key(key(KeyCode::Char('j'))); // units: Militia -> Settler
    let target = app.current_picker_target().expect("cursor has a target");
    assert_ne!(
        target,
        ProductionTarget::Unit(crate::model::units::UnitClass::Militia)
    );
    app.handle_key(key(KeyCode::Enter));
    assert!(!app.production_picker_open);
    let city = app
        .engine
        .as_ref()
        .unwrap()
        .city(app.selected_city.unwrap())
        .unwrap();
    assert_eq!(city.production_target(), Some(target));
}

#[test]
fn clicking_save_applies_the_selection_and_closes() {
    let (mut app, panel, _) = with_production_picker_open();
    app.handle_key(key(KeyCode::Char('j')));
    let target = app.current_picker_target().unwrap();
    let save = crate::tui::production_picker::save_button_rect(panel);
    app.left_click(save.x + 1, save.y);
    assert!(!app.production_picker_open);
    let city = app
        .engine
        .as_ref()
        .unwrap()
        .city(app.selected_city.unwrap())
        .unwrap();
    assert_eq!(city.production_target(), Some(target));
}

#[test]
fn clicking_cancel_closes_without_changing_production() {
    let (mut app, panel, _) = with_production_picker_open();
    app.handle_key(key(KeyCode::Char('j')));
    let cancel = crate::tui::production_picker::cancel_button_rect(panel);
    app.left_click(cancel.x + 1, cancel.y);
    assert!(!app.production_picker_open);
    let city = app
        .engine
        .as_ref()
        .unwrap()
        .city(app.selected_city.unwrap())
        .unwrap();
    assert_eq!(city.production_target(), None);
}

#[test]
fn clicks_outside_the_picker_are_swallowed_while_it_is_open() {
    let (mut app, panel, _) = with_production_picker_open();
    let city = app.selected_city;
    // Inside the window but outside the panel's rows and buttons.
    app.left_click(panel.x + 1, panel.y + 1);
    assert!(app.production_picker_open);
    assert_eq!(
        app.selected_city, city,
        "outside clicks must not reach the map"
    );
}

#[test]
fn wheel_events_hit_the_picker_not_the_window_while_open() {
    let (mut app, _, _) = with_production_picker_open();
    let before = app.city_window_scroll;
    // A fresh city offers a short list, so the picker's shared scroll
    // clamps to 0 rather than scrolling beneath the window.
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(app.picker_scroll, 0);
    assert_eq!(app.city_window_scroll, before);
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(app.picker_scroll, 0);
    assert_eq!(app.city_window_scroll, before);
}

/// Presses the left button at the first point and drags through the rest,
/// releasing on the last point, so mouse-driven gestures can be scripted in
/// terms of the (column, row) pixels they cross.
fn press_and_drag(app: &mut App, track: &[(u16, u16)]) {
    let (c0, r0) = track.first().expect("a drag track has a press");
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        *c0,
        *r0,
    ));
    for &(column, row) in &track[1..] {
        app.handle_mouse(mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            column,
            row,
        ));
    }
    let (last_c, last_r) = track.last().expect("a drag track has a release");
    app.handle_mouse(mouse_event(
        MouseEventKind::Up(MouseButton::Left),
        *last_c,
        *last_r,
    ));
}

#[test]
fn dragging_the_map_pans_the_camera() {
    let (mut app, _, _) = playing_app();
    let map_w = app.engine.as_ref().unwrap().width();
    // Press, pull twelve screen columns rightward across two drag moves,
    // and release. Six world tiles' worth of hand travel pans the camera
    // six tiles west.
    press_and_drag(&mut app, &[(48, 12), (50, 12), (60, 12)]);
    let expected_x = (0i32 - 6).rem_euclid(map_w as i32) as usize;
    assert_eq!(app.camera.get(), (expected_x, 0));
    assert!(!app.camera_follow.get(), "dragging lets the hand steer");
    assert!(!app.drag_engaged.get(), "release ends the drag");
    assert_eq!(app.selected_city, None, "a drag must not select a city");
}

#[test]
fn a_drag_past_the_slop_never_clicks() {
    let (mut app, cx, cy) = playing_app();
    // The drag starts on top of the player's city but travels beyond the
    // click slop, so the camera pans and the city is left unselected.
    let column = (LEFT_COLUMN_WIDTH as usize + cx * 2) as u16;
    press_and_drag(&mut app, &[(column, cy as u16), (column + 6, cy as u16)]);
    assert_eq!(app.selected_city, None, "drag must not act as a click");
    assert_eq!(app.camera.get().0, 77, "three tiles of carry pans west");
    assert!(!app.camera_follow.get());
}

#[test]
fn dragging_up_clamps_the_camera_to_the_bottom_of_the_map() {
    let (mut app, _, _) = playing_app();
    let map_h = app.engine.as_ref().unwrap().height();
    let pane = app.map_pane.get().expect("tests set a pane");
    let max_y = map_h.saturating_sub(pane.height as usize);
    // Lifting the mouse twenty rows scrolls down to the bottom-most
    // pane-sized window of the map, never beyond it.
    press_and_drag(&mut app, &[(48, 30), (48, 10)]);
    assert_eq!(app.camera.get().1, max_y);
}

#[test]
fn a_tiny_drag_stays_a_click() {
    let (mut app, cx, cy) = playing_app();
    // One screen cell of movement is click jitter: releasing on the
    // city tile still selects it and the camera stays put.
    let column = (LEFT_COLUMN_WIDTH as usize + cx * 2) as u16;
    press_and_drag(&mut app, &[(column, cy as u16), (column + 1, cy as u16)]);
    assert!(
        app.selected_city.is_some(),
        "jitter must not cancel a click"
    );
    assert_eq!(app.camera.get(), (0, 0));
    assert!(app.camera_follow.get());
}

#[test]
fn camera_follow_resumes_when_the_unit_moves_after_a_drag() {
    let (mut app, _, _) = playing_app();
    press_and_drag(&mut app, &[(48, 12), (50, 12), (60, 12)]);
    assert!(!app.camera_follow.get());
    // Any attempt to move the selected unit hands the camera back to
    // follow-the-selection regardless of whether the move is legal.
    app.handle_key(key(KeyCode::Left));
    assert!(app.camera_follow.get());
}

#[test]
fn space_ends_the_turn() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let first_turn = app.engine.as_ref().unwrap().turn();

    // With rivals, one space only passes play to the next player. Press
    // space a bounded number of times until the turn number actually
    // advances.
    let mut turn = first_turn;
    for _ in 0..8 {
        app.handle_key(key(KeyCode::Char(' ')));
        turn = app.engine.as_ref().unwrap().turn();
        if turn != first_turn {
            break;
        }
    }
    assert!(
        turn > first_turn,
        "ending the turn did not advance the game"
    );
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        app.selected_unit,
        engine.player_units().first().map(|unit| unit.id())
    );
}

#[test]
fn camera_centres_on_the_focused_tile() {
    // A 10x10 pane centred on (7, 9) half-width 5, half-height 5.
    // Camera is far away so the focus is outside the middle 75%.
    let camera = camera_for(Some((7, 9)), (20, 20), (10, 10), (0, 0));
    assert_eq!(camera, (2, 4));
}

#[test]
fn camera_stays_put_when_focus_is_in_the_middle_70_percent() {
    // A 10x10 pane centred on (7, 7) so camera is (2, 2).
    // Focus at (7, 7) is dead centre — well within the middle 75%.
    let camera = camera_for(Some((7, 7)), (20, 20), (10, 10), (2, 2));
    assert_eq!(camera, (2, 2));
}

#[test]
fn camera_recentres_when_focus_leaves_the_middle_70_percent() {
    // 10x10 pane, camera at (2, 2), visible x: 2..=11, visible y: 2..=11.
    // Middle 75% margin = 10/8 = 1, so safe x: 3..=10, safe y: 3..=10.
    // Focus at (2, 2) is outside the safe range → camera re-centres.
    // Horizontal wrapping: centre at (2 - 5).rem_euclid(20) = 17.
    let camera = camera_for(Some((2, 2)), (20, 20), (10, 10), (2, 2));
    assert_eq!(camera, (17, 0));
}

#[test]
fn camera_is_clamped_to_the_map_edges_vertically() {
    // Near the origin: y cannot go negative.
    assert_eq!(
        camera_for(Some((0, 0)), (20, 20), (10, 10), (5, 5)),
        (15, 0)
    );
    // A pane larger than the map: with the wider 70 % margin the focus
    // at x=5 falls outside the safe zone and re-centres horizontally.
    assert_eq!(camera_for(Some((5, 5)), (10, 10), (40, 30), (0, 0)), (5, 0));
    // Far y edge: keep the bottommost row on the map.
    // Horizontally the camera wraps: centre on x=0 gives (0-5).rem_euclid(20)=15.
    assert_eq!(
        camera_for(Some((0, 19)), (20, 20), (10, 10), (0, 0)),
        (15, 10)
    );
    // No focus keeps the current camera.
    assert_eq!(camera_for(None, (80, 50), (40, 40), (10, 10)), (10, 10));
}

#[test]
fn camera_wraps_around_the_horizontal_edges() {
    // Focus at the far east (x=19) on an 80-wide map, pane 40 wide.
    // Centre at (19 - 20).rem_euclid(80) = 79.
    let camera = camera_for(Some((19, 0)), (80, 50), (40, 40), (0, 0));
    assert_eq!(camera.0, 79);
    // Focus at x=0 on an 80-wide map, pane 40 wide.
    // Centre at (0 - 20).rem_euclid(80) = 60.
    let camera = camera_for(Some((0, 10)), (80, 50), (40, 40), (0, 0));
    assert_eq!(camera.0, 60);
}

#[test]
fn camera_stays_put_when_focus_wraps_into_the_middle_70_percent() {
    // Map 80 wide, pane 40 wide, camera at (79, 25).
    // Viewport x: 79, 0, 1, ..., 38. Focus at x=5 is at view_col (5+80-79)%80 = 6.
    // margin_x = 40*15/100 = 6. 6 >= 6 && 6 < 34 → inside.
    // Viewport y: 25..64. Focus at y=31, view_row = 6. margin_y = 6. 6 >= 6 → inside.
    let camera = camera_for(Some((5, 31)), (80, 50), (40, 40), (79, 25));
    assert_eq!(camera, (79, 25));
}

#[test]
fn moving_in_a_legal_direction_moves_the_selected_unit() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));

    let engine = app.engine.as_ref().unwrap();
    let unit = engine
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == app.selected_unit.unwrap())
        .unwrap();
    let before = unit.location;

    // Find a neighbouring tile the settler can afford to step onto (open
    // land costs 1 move; a settler has 1 move), regardless of the world.
    // Use rem_euclid for the horizontal axis to match the engine's
    // east/west wrapping.
    let map_w = engine.width() as isize;
    let mut pressed = None;
    for (code, dx, dy) in [
        (KeyCode::Right, 1, 0),
        (KeyCode::Left, -1, 0),
        (KeyCode::Up, 0, -1),
        (KeyCode::Down, 0, 1),
    ] {
        let nx = (before.x as isize + dx).rem_euclid(map_w) as usize;
        let ny = (before.y as isize + dy).clamp(0, engine.height() as isize - 1) as usize;
        let terrain = engine.tile(nx, ny).terrain;
        if terrain.is_land() && terrain.movement_cost() <= 1 {
            pressed = Some(code);
            break;
        }
    }
    let Some(code) = pressed else {
        return; // no land neighbour; nothing legal to assert
    };

    app.handle_key(key(code));
    let engine = app.engine.as_ref().unwrap();
    let after = engine
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == app.selected_unit.unwrap())
        .unwrap()
        .location;
    // Account for horizontal wrapping when checking adjacency.
    let dx_raw = after.x as isize - before.x as isize;
    let wrapped_dx = if dx_raw.abs() > 1 {
        dx_raw.signum() * (map_w - dx_raw.abs())
    } else {
        dx_raw
    };
    let dy = after.y as isize - before.y as isize;
    assert!(
        wrapped_dx.abs() + dy.abs() == 1,
        "unit did not move by exactly one tile: {before:?} -> {after:?}"
    );
}

#[test]
fn v_founds_a_city_with_the_selected_settler() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let engine = app.engine.as_ref().unwrap();
    // The game begins with a single settler.
    assert_eq!(engine.player_units().len(), 1);
    let settler = engine.player_units()[0];
    assert_eq!(settler.unit_class, UnitClass::Settler);
    let location = settler.location;

    app.handle_key(key(KeyCode::Char('v')));

    let engine = app.engine.as_ref().unwrap();
    // The settler is consumed and a city now sits on its tile.
    assert!(
        engine.player_units().is_empty(),
        "the founding settler should be consumed"
    );
    let city = engine
        .city_at(location.x as usize, location.y as usize)
        .unwrap();
    assert_eq!(city.population(), 1);
}

#[test]
fn a_founded_city_is_named_after_its_civilizations_capital() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let engine = app.engine.as_ref().unwrap();
    let settler = engine.player_units()[0];
    let name = city_name_for(engine, settler.id());
    assert_eq!(name, "Washington");
}

#[test]
fn city_names_are_allocated_in_order_of_founding() {
    assert_eq!(next_city_name(Civilization::English, 0), "London");
    assert_eq!(next_city_name(Civilization::English, 1), "York");
    assert_eq!(next_city_name(Civilization::English, 2), "Manchester");
    assert_eq!(next_city_name(Civilization::American, 1), "Boston");
    assert_eq!(next_city_name(Civilization::Zulu, 3), "Isandhlwana");
}

#[test]
fn names_fall_back_to_a_number_once_the_city_list_runs_out() {
    assert_eq!(next_city_name(Civilization::English, 20), "City 21");
    assert_eq!(next_city_name(Civilization::English, 42), "City 43");
}

#[test]
fn diagonal_commands_move_the_selected_unit_diagonally() {
    // y/u/b/n map to NW/NE/SW/SE.
    for (code, tile_dx, tile_dy) in [
        (KeyCode::Char('y'), -1, -1),
        (KeyCode::Char('u'), 1, -1),
        (KeyCode::Char('b'), -1, 1),
        (KeyCode::Char('n'), 1, 1),
    ] {
        let mut app = App::new();
        at_start(&mut app);
        app.handle_key(key(KeyCode::Char('s'))); // begin the game

        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|u| u.id() == app.selected_unit.unwrap())
            .unwrap();
        let before = unit.location;
        let w = engine.width() as isize;

        let nx = (before.x as isize + tile_dx).rem_euclid(w);
        let ny = (before.y as isize + tile_dy).clamp(0, engine.height() as isize - 1);
        let terrain = engine.tile(nx as usize, ny as usize).terrain;
        if ny == before.y as isize + tile_dy && terrain.is_land() && terrain.movement_cost() <= 1 {
            app.handle_key(key(code));
            let engine = app.engine.as_ref().unwrap();
            let after = engine
                .player_units()
                .into_iter()
                .find(|u| u.id() == app.selected_unit.unwrap())
                .unwrap()
                .location;
            let dx = (after.x as isize - before.x as isize + w) % w;
            let dx = if dx > w / 2 { dx - w } else { dx };
            let dy = after.y as isize - before.y as isize;
            assert_eq!(
                (dx, dy),
                (tile_dx, tile_dy),
                "diagonal key did not move the unit as expected: {before:?} -> {after:?}"
            );
        }
    }
}

#[test]
fn tab_cycles_between_the_players_units() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let engine = app.engine.as_ref().unwrap();
    let ids: Vec<UnitId> = engine.player_units().iter().map(|unit| unit.id()).collect();
    if ids.len() < 2 {
        // With a single starting settler there's nothing to cycle between.
        return;
    }
    assert_eq!(app.selected_unit, Some(ids[0]));
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[1]));
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[0]));
}

#[test]
fn help_bar_overwrites_the_bottom_two_rows_without_shifting_the_game() {
    let render = |app: &App| {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    };

    let mut begun = App::new();
    at_start(&mut begun);
    begun.handle_key(key(KeyCode::Char('s')));

    let without_help = render(&begun);
    begun.handle_key(key(KeyCode::Char('?')));
    assert!(begun.show_help);
    let with_help = render(&begun);

    // The help bar is two rows deep and overwrites the bottom two rows;
    // every row above them is identical.
    for y in 0..38 {
        for x in 0..120 {
            assert_eq!(
                with_help.cell((x, y)).unwrap().symbol(),
                without_help.cell((x, y)).unwrap().symbol(),
                "row {y} col {x} shifted by the help bar"
            );
        }
    }
    // The help text spans the bottom two rows.
    let bottom_rows: String = (0..120)
        .map(|x| with_help.cell((x, 38)).unwrap().symbol().to_string())
        .chain((0..120).map(|x| with_help.cell((x, 39)).unwrap().symbol().to_string()))
        .collect();
    assert!(bottom_rows.contains("end turn"));
    assert!(bottom_rows.contains("move"));
}

#[test]
fn play_commands_include_unit_actions_when_a_unit_is_focused() {
    let commands = playing_commands(true, false);
    assert!(commands.iter().any(|(k, _)| *k == "f"));
    assert!(commands.iter().any(|(k, _)| *k == "arrows"));
    assert!(commands.iter().any(|(k, _)| *k == "?"));
    assert!(commands.iter().any(|(k, _)| *k == "tab"));
}

#[test]
fn found_city_only_shows_for_a_selected_settler() {
    let commands = playing_commands(true, true);
    assert!(commands.iter().any(|(k, _)| *k == "v"));
    let commands = playing_commands(true, false);
    assert!(!commands.iter().any(|(k, _)| *k == "v"));
}

#[test]
fn play_commands_offer_navigation_when_nothing_is_focused() {
    let commands = playing_commands(false, false);
    assert!(commands.iter().any(|(k, _)| *k == "tab"));
    assert!(!commands.iter().any(|(k, _)| *k == "f"));
}

#[test]
fn esc_and_q_return_to_the_menu() {
    for code in [KeyCode::Esc, KeyCode::Char('q')] {
        let mut app = App::new();
        app.start_new_game();
        assert!(!app.handle_key(key(code)));
        assert!(matches!(app.phase, Phase::Menu));
    }
}

#[test]
fn status_bar_hides_for_two_seconds() {
    assert_eq!(fade_progress(Duration::from_secs(1)), 0.0);
    assert_eq!(fade_progress(Duration::from_secs(2)), 0.0);
    assert_eq!(fade_progress(Duration::from_millis(2500)), 0.5);
}

#[test]
fn status_bar_finishes_fading_after_three_seconds() {
    assert_eq!(fade_progress(Duration::from_secs(3)), 1.0);
    assert_eq!(fade_progress(Duration::from_secs(10)), 1.0);
}

/// Play until the research-completion dialog opens. The starting city
/// begins researching Construction (auto target at 10 research points),
/// so a handful of wrapped turns is enough to finish it.
fn app_with_research_dialog() -> App {
    let (mut app, _, _) = playing_app();
    let mut turns = 0;
    while app.research_dialog.is_none() && turns < 80 {
        app.handle_key(key(KeyCode::Char(' ')));
        turns += 1;
    }
    assert!(
        app.research_dialog.is_some(),
        "research dialog never opened after {turns} presses"
    );
    app
}

#[test]
fn ending_turns_opens_the_research_dialog_on_a_discovery() {
    let app = app_with_research_dialog();
    let dialog = app.research_dialog.as_ref().unwrap();
    assert_eq!(dialog.discovered, Advancement::Construction);
    assert_eq!(dialog.cursor, 0);
    assert!(!dialog.choices.contains(&Advancement::Construction));
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        dialog.choices,
        engine.researchable_advancements_for(PlayerId::new(0))
    );
}

#[test]
fn the_research_dialog_navigates_and_confirms() {
    let mut app = app_with_research_dialog();
    let third = app.research_dialog.as_ref().unwrap().choices[2];
    app.handle_key(key(KeyCode::Char('j')));
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.research_dialog.as_ref().unwrap().cursor, 2);
    app.handle_key(key(KeyCode::Enter));
    assert!(app.research_dialog.is_none());
    assert_eq!(app.research_dialog_rect.get(), None);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.advancement_in_progress(), Some(third));
    assert_eq!(engine.research_progress(), 0);
}

#[test]
fn the_research_dialog_swallows_game_keys() {
    let mut app = app_with_research_dialog();
    for code in [KeyCode::Char('q'), KeyCode::Char('v'), KeyCode::Tab] {
        let was_quit = app.handle_key(key(code));
        assert!(!was_quit, "dialog must capture {code:?}");
    }
    assert!(app.research_dialog.is_some());
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn clicking_ok_confirms_the_research_dialog() {
    let mut app = app_with_research_dialog();
    let target = app.research_dialog.as_ref().unwrap().choices[0];
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        assert!(app.research_dialog_rect.get().is_some());
        terminal.size().unwrap()
    };
    let ok = research_dialog::ok_button_rect(research_dialog::dialog_rect(area.into()));
    app.left_click(ok.x + ok.width / 2, ok.y.max(1));
    assert!(app.research_dialog.is_none());
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.advancement_in_progress(), Some(target));
}

#[test]
fn clicking_a_dialog_row_moves_the_cursor_then_ok_confirms() {
    let mut app = app_with_research_dialog();
    let third = app.research_dialog.as_ref().unwrap().choices[2];
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        terminal.size().unwrap()
    };
    let rows = research_dialog::list_rect(research_dialog::dialog_rect(area.into()));
    app.left_click(rows.x + 1, rows.y + 2);
    assert_eq!(app.research_dialog.as_ref().unwrap().cursor, 2);
    let ok = research_dialog::ok_button_rect(research_dialog::dialog_rect(area.into()));
    app.left_click(ok.x + ok.width / 2, ok.y.max(1));
    assert!(app.research_dialog.is_none());
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.advancement_in_progress(), Some(third));
}

/// A game begun (but not yet founding a city), so the starting settler
/// still stands on its tile ready to take a work order.
fn app_with_settler() -> App {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(
        app.engine
            .as_ref()
            .unwrap()
            .player_units()
            .iter()
            .any(|u| u.unit_class == UnitClass::Settler)
    );
    app
}

/// A game where the starting settler has a rival militia standing on an
/// adjacent land tile, with the two civilizations at war. Returns the app,
/// the settler's id, the direction the settler can step, and the tile the
/// attack will land on.
fn app_with_adjacent_enemy() -> Option<(App, UnitId, Direction, Location)> {
    let mut app = app_with_settler();
    let (settler_id, location) = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|unit| unit.unit_class == UnitClass::Settler)
            .expect("the starting settler exists");
        (unit.id(), unit.location)
    };
    let adjacent = {
        let engine = app.engine.as_ref().unwrap();
        let (width, height) = (engine.width(), engine.height());
        Direction::iter().find_map(|direction| {
            let (dx, dy) = direction.delta();
            let nx = (location.x as isize + dx).rem_euclid(width as isize) as usize;
            let ny = location.y as isize + dy;
            if ny < 0 || ny >= height as isize {
                return None;
            }
            let terrain = engine.tile(nx, ny as usize).terrain;
            if terrain.is_land() && terrain.movement_cost() <= 1 {
                Some((direction, Location::new(nx as u16, ny as u16)))
            } else {
                None
            }
        })
    };
    let (direction, enemy_tile) = adjacent?;
    let engine = app.engine.as_mut().unwrap();
    engine.submit(Command::DeclareWar {
        opponent: PlayerId::new(1),
    });
    engine.spawn_unit(
        UnitClass::Militia,
        enemy_tile,
        PlayerId::new(1),
        CityId::new(0),
    );
    Some((app, settler_id, direction, enemy_tile))
}

/// A game where the starting settler has a rival militia standing on an
/// adjacent land tile, the two civilizations never having been at war.
/// Returns the app, the settler's id and starting tile, the direction the
/// settler can step, and the rival's tile.
fn app_with_adjacent_foreigner() -> Option<(App, UnitId, Direction, Location, Location)> {
    let mut app = app_with_settler();
    let (settler_id, home) = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|unit| unit.unit_class == UnitClass::Settler)
            .expect("the starting settler exists");
        (unit.id(), unit.location)
    };
    let adjacent = {
        let engine = app.engine.as_ref().unwrap();
        let (width, height) = (engine.width(), engine.height());
        Direction::iter().find_map(|direction| {
            let (dx, dy) = direction.delta();
            let nx = (home.x as isize + dx).rem_euclid(width as isize) as usize;
            let ny = home.y as isize + dy;
            if ny < 0 || ny >= height as isize {
                return None;
            }
            let terrain = engine.tile(nx, ny as usize).terrain;
            if terrain.is_land() && terrain.movement_cost() <= 1 {
                Some((direction, Location::new(nx as u16, ny as u16)))
            } else {
                None
            }
        })
    };
    let (direction, enemy_tile) = adjacent?;
    let engine = app.engine.as_mut().unwrap();
    engine.spawn_unit(
        UnitClass::Militia,
        enemy_tile,
        PlayerId::new(1),
        CityId::new(0),
    );
    Some((app, settler_id, direction, enemy_tile, home))
}

/// A game where the starting settler has a rival militia standing two
/// land tiles away, the two civilizations never having met. Returns the
/// app, the settler's id, the direction of the first step, and the empty
/// tile that step lands on (adjacent to the rival, so moving there meets
/// them).
fn app_with_unknown_neighbor() -> Option<(App, UnitId, Direction, Location)> {
    let mut app = app_with_settler();
    let (settler_id, home) = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|unit| unit.unit_class == UnitClass::Settler)
            .expect("the starting settler exists");
        (unit.id(), unit.location)
    };
    let step = {
        let engine = app.engine.as_ref().unwrap();
        let (width, height) = (engine.width(), engine.height());
        let step_to = |from: Location, direction: Direction| -> Option<Location> {
            let (dx, dy) = direction.delta();
            let x = (from.x as isize + dx).rem_euclid(width as isize) as usize;
            let y = from.y as isize + dy;
            if y < 0 || y >= height as isize {
                None
            } else {
                Some(Location::new(x as u16, y as u16))
            }
        };
        Direction::iter().find_map(|direction| {
            let meet = step_to(home, direction)?;
            let far = step_to(meet, direction)?;
            let walkable = |tile: Location| {
                let terrain = engine.tile(tile.x as usize, tile.y as usize).terrain;
                terrain.is_land() && terrain.movement_cost() <= 1
            };
            if walkable(meet) && walkable(far) {
                Some((direction, meet))
            } else {
                None
            }
        })
    };
    let (direction, meet_tile) = step?;
    let far_tile = {
        let engine = app.engine.as_ref().unwrap();
        let (width, height) = (engine.width(), engine.height());
        let (dx, dy) = direction.delta();
        let x = (meet_tile.x as isize + dx).rem_euclid(width as isize) as u16;
        let y = meet_tile.y as isize + dy;
        Location::new(x, y.clamp(0, height as isize - 1) as u16)
    };
    let engine = app.engine.as_mut().unwrap();
    engine.spawn_unit(
        UnitClass::Militia,
        far_tile,
        PlayerId::new(1),
        CityId::new(0),
    );
    Some((app, settler_id, direction, meet_tile))
}

#[test]
fn moving_next_to_an_unknown_rival_opens_a_contact_diplomacy_window() {
    let Some((mut app, settler_id, direction, meet_tile)) = app_with_unknown_neighbor() else {
        return;
    };
    app.move_selected_unit(direction);
    let dialog = app
        .diplomacy
        .expect("first contact opens the war-or-peace window");
    assert_eq!(dialog.opponent, PlayerId::new(1));
    assert_eq!(dialog.origin, DiplomacyOrigin::Contact);
    assert_eq!(dialog.choice, DiplomacyChoice::Peace);
    assert!(dialog.pending.is_none());
    // The settler still completed its step onto the meet tile.
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert_eq!(unit.location, meet_tile);
}

#[test]
fn declaring_war_from_a_contact_makes_the_rival_attackable() {
    let Some((mut app, _, direction, _)) = app_with_unknown_neighbor() else {
        return;
    };
    app.move_selected_unit(direction);
    app.handle_key(key(KeyCode::Char('h'))); // WAR
    app.handle_key(key(KeyCode::Enter));
    assert!(app.diplomacy.is_none(), "the dialog closes on confirmation");
    // With moves restored, the settler now attacks the rival outright —
    // no second diplomacy window, just combat.
    app.end_turn();
    app.end_turn();
    app.move_selected_unit(direction);
    assert!(app.diplomacy.is_none(), "war bypasses the diplomacy window");
    assert!(
        app.battle_animation.is_some(),
        "war makes the approach a battle"
    );
}

#[test]
fn keeping_peace_from_a_contact_still_blocks_foreign_passage() {
    let Some((mut app, settler_id, direction, meet_tile)) = app_with_unknown_neighbor() else {
        return;
    };
    app.move_selected_unit(direction);
    app.handle_key(key(KeyCode::Enter)); // PEACE
    assert!(app.diplomacy.is_none());
    app.end_turn();
    app.end_turn();
    // Still at peace, stepping onto the rival's tile reopens the window as
    // a blocked move.
    app.move_selected_unit(direction);
    let dialog = app
        .diplomacy
        .expect("peaceful passage still prompts diplomacy");
    assert_eq!(dialog.opponent, PlayerId::new(1));
    assert_eq!(dialog.origin, DiplomacyOrigin::Movement);
    assert_eq!(dialog.pending, Some((settler_id, direction)));
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert_eq!(unit.location, meet_tile, "the blocked settler never moved");
}

#[test]
fn stepping_onto_a_peaceful_foreign_tile_opens_a_blocked_move_window() {
    let Some((mut app, settler_id, direction, _, home)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    let dialog = app
        .diplomacy
        .expect("a peaceful foreign step prompts diplomacy");
    assert_eq!(dialog.opponent, PlayerId::new(1));
    assert_eq!(dialog.origin, DiplomacyOrigin::Movement);
    assert_eq!(dialog.choice, DiplomacyChoice::Peace);
    assert_eq!(dialog.pending, Some((settler_id, direction)));
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert_eq!(unit.location, home, "the blocked settler never moved");
}

#[test]
fn declaring_war_from_the_blocked_move_attacks_the_rival() {
    let Some((mut app, _, direction, enemy_tile, _)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    app.handle_key(key(KeyCode::Char('h'))); // WAR
    app.handle_key(key(KeyCode::Enter));
    assert!(app.diplomacy.is_none(), "war closes the window");
    assert!(
        app.battle_animation.is_some(),
        "the retaken move attacks the rival"
    );
    let animation = app.battle_animation.unwrap();
    assert_eq!(animation.location, enemy_tile);
}

#[test]
fn keeping_peace_from_the_blocked_move_leaves_the_unit_in_place() {
    let Some((mut app, settler_id, direction, _, home)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    app.handle_key(key(KeyCode::Enter)); // PEACE
    assert!(app.diplomacy.is_none());
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert_eq!(unit.location, home);
}

#[test]
fn the_diplomacy_window_swallows_game_keys() {
    let Some((mut app, settler_id, direction, _, home)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    assert!(app.diplomacy.is_some());
    app.handle_key(key(KeyCode::Char('k'))); // would move the unit
    assert!(app.diplomacy.is_some(), "the dialog captures the key");
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert_eq!(unit.location, home, "the swallowed key moved the unit");
}

#[test]
fn the_open_diplomacy_window_draws_over_the_map() {
    let Some((mut app, _, direction, _, _)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    assert!(
        app.diplomacy_rect.get().is_some(),
        "the drawn window records its rectangle"
    );
}

#[test]
fn moving_onto_an_enemy_starts_a_battle_flash_on_the_defended_tile() {
    let Some((mut app, settler_id, direction, enemy_tile)) = app_with_adjacent_enemy() else {
        return;
    };
    app.move_selected_unit(direction);
    let animation = app
        .battle_animation
        .expect("attacking an enemy starts a battle flash");
    assert_eq!(
        animation.location, enemy_tile,
        "the flash sits on the tile the attack was aimed at"
    );
    assert!(animation.start <= app.started_at.elapsed());

    // Combat is randomly resolved, but the stale selection is dropped only
    // when the attacking settler was removed by a repelled attack.
    let engine = app.engine.as_ref().unwrap();
    let survived = engine
        .player_units()
        .iter()
        .any(|unit| unit.id() == settler_id);
    if survived {
        assert_eq!(app.selected_unit, Some(settler_id));
    } else {
        assert_eq!(app.selected_unit, None);
    }
}

#[test]
fn a_move_that_meets_no_enemy_starts_no_battle_flash() {
    let mut app = app_with_settler();
    let direction = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine.player_units()[0];
        Direction::iter().find(|direction| {
            let (dx, dy) = direction.delta();
            let nx = (unit.location.x as isize + dx).rem_euclid(engine.width() as isize) as usize;
            let ny = unit.location.y as isize + dy;
            if ny < 0 || ny >= engine.height() as isize {
                return false;
            }
            let ny = ny as usize;
            let tile = engine.tile(nx, ny).terrain;
            tile.is_land() && tile.movement_cost() <= 1 && engine.units_at(nx, ny).is_empty()
        })
    };
    let Some(direction) = direction else {
        return; // no affordable empty neighbour; nothing legal to assert
    };
    app.move_selected_unit(direction);
    assert!(
        app.battle_animation.is_none(),
        "a move onto an empty tile is not a battle"
    );
}

#[test]
fn an_expired_battle_flash_is_dropped_from_the_app_state() {
    let Some((mut app, _settler, direction, _tile)) = app_with_adjacent_enemy() else {
        return;
    };
    app.move_selected_unit(direction);
    assert!(app.battle_animation.is_some());
    app.clear_expired_battle_animation(app.started_at.elapsed());
    assert!(
        app.battle_animation.is_some(),
        "a fresh flash is not yet over"
    );
    let far_future = app.started_at.elapsed() + BATTLE_FLASH_DURATION + Duration::from_secs(1);
    app.clear_expired_battle_animation(far_future);
    assert_eq!(
        app.battle_animation, None,
        "once the flash has run its course the state is cleared"
    );
}

#[test]
fn pressing_w_opens_the_work_picker_for_the_selected_settler() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    assert!(app.work_picker_open);
    assert_eq!(app.work_picker_cursor, 0);
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.work_picker_open);
    assert_eq!(app.work_picker_rect.get(), None);
}

#[test]
fn a_game_that_has_founded_its_first_city_has_no_settler_left_to_work() {
    let (mut app, _, _) = playing_app();
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.work_picker_open);
}

#[test]
fn the_work_picker_only_offers_buildable_improvements() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let unit = app.engine.as_ref().unwrap().player_units()[0];
    let tile = app
        .engine
        .as_ref()
        .unwrap()
        .tile(unit.location.x as usize, unit.location.y as usize);
    assert_eq!(app.work_rows(), work_picker::buildable_improvements(tile));
    assert!(
        app.work_rows().contains(&TerrainImprovement::Road),
        "a settler on land can always build a road: {:?}",
        app.work_rows()
    );
}

#[test]
fn saving_the_work_picker_orders_the_settler_and_the_road_lands_next_turn() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let road = app
        .work_rows()
        .iter()
        .position(|&improvement| improvement == TerrainImprovement::Road)
        .expect("a settler on land can always build a road");
    for _ in 0..road {
        app.handle_key(key(KeyCode::Char('j')));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(!app.work_picker_open);
    let engine = app.engine.as_ref().unwrap();
    let unit = engine.player_units()[0];
    assert_eq!(unit.order(), UnitOrder::Improving(TerrainImprovement::Road));
    assert_eq!(unit.moves_remaining(), 0);
    let location = unit.location;
    assert!(
        !engine
            .tile(location.x as usize, location.y as usize)
            .has_road()
    );

    // The settler keeps working, and the road lands on its next own turn:
    // end the human turn, play through the rivals, and wait until play
    // wraps back to the human.
    let mut presses = 0;
    loop {
        app.handle_key(key(KeyCode::Char(' ')));
        presses += 1;
        if app.engine.as_ref().unwrap().current_player_id() == PlayerId::new(0) || presses >= 6 {
            break;
        }
    }
    let engine = app.engine.as_ref().unwrap();
    let unit = engine.player_units()[0];
    assert!(
        engine
            .tile(location.x as usize, location.y as usize)
            .has_road()
    );
    assert_eq!(unit.order(), UnitOrder::Idle);
}

#[test]
fn esc_closes_the_work_picker_without_an_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.work_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.player_units()[0].order(), UnitOrder::Idle);
}

#[test]
fn the_work_picker_requires_a_real_selected_settler() {
    let mut app = app_with_settler();
    app.selected_unit = None;
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.work_picker_open);
    // A selection that picks out no living unit must not open either: the
    // engine itself rejects non-settler work orders.
    app.selected_unit = Some(crate::model::units::UnitId::new(999));
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.work_picker_open);
}

#[test]
fn pressing_c_cancels_the_selected_units_work_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let first = app.work_rows()[0];
    app.handle_key(key(KeyCode::Enter));
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        engine.player_units()[0].order(),
        UnitOrder::Improving(first)
    );
    app.handle_key(key(KeyCode::Char('c')));
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.player_units()[0].order(), UnitOrder::Idle);
}

#[test]
fn the_work_picker_swallows_game_keys() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let turn = app.engine.as_ref().unwrap().turn();
    for code in [KeyCode::Char('q'), KeyCode::Char('l'), KeyCode::Char(' ')] {
        let was_quit = app.handle_key(key(code));
        assert!(!was_quit, "work picker must capture {code:?}");
    }
    assert!(app.work_picker_open);
    assert!(matches!(app.phase, Phase::Playing));
    assert_eq!(
        app.engine.as_ref().unwrap().turn(),
        turn,
        "no end turn while the picker is open"
    );
}

#[test]
fn clicking_a_row_then_save_issues_the_work_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let expected = app.work_rows()[0];
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        assert!(app.work_picker_rect.get().is_some());
        terminal.size().unwrap()
    };
    let panel = work_picker::work_picker_rect(area.into());
    let rows = work_picker::rows_rect(panel);
    app.left_click(rows.x + 1, rows.y);
    assert_eq!(app.work_picker_cursor, 0);
    let save = work_picker::save_button_rect(panel);
    app.left_click(save.x + save.width / 2, save.y.max(1));
    assert!(!app.work_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        engine.player_units()[0].order(),
        UnitOrder::Improving(expected)
    );
}

#[test]
fn clicking_cancel_closes_the_work_picker_without_an_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        terminal.size().unwrap()
    };
    let panel = work_picker::work_picker_rect(area.into());
    let cancel = work_picker::cancel_button_rect(panel);
    app.left_click(cancel.x + cancel.width / 2, cancel.y.max(1));
    assert!(!app.work_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.player_units()[0].order(), UnitOrder::Idle);
}
