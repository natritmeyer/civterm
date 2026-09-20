use super::*;
use crate::game_engine::{Command, Player};
use crate::model::cartography::Location;
use crate::model::cities::{City, ProductionTarget};
use crate::model::geography::{Terrain, TerrainImprovement};
use crate::model::units::UnitOrder;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use std::time::Duration;
use strum::IntoEnumIterator;

/// Fast-forward the app clock so transient animations (battle flash,
/// rival-move replay) expire, allowing a test to press End Turn repeatedly
/// without waiting for real wall-clock time.
fn warp_app(app: &mut App, secs: u64) {
    app.started_at -= Duration::from_secs(secs);
    let now = app.started_at.elapsed();
    app.clear_expired_battle_animation(now);
    app.clear_expired_rival_animation(now);
}

/// Wind the app clock past any pending unit auto-advance deadline and run
/// the loop tick that fires it.
fn fire_pending_unit_advance(app: &mut App) {
    app.started_at -= Duration::from_millis(400);
    let now = app.started_at.elapsed();
    app.maybe_finish_pending_unit_advance(now);
}

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
fn load_opens_the_load_prompt() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Char('l')));
    assert_eq!(app.selected, 1);
    assert!(matches!(app.phase, Phase::Menu));
    let prompt = app.save_prompt.as_ref().unwrap();
    assert_eq!(prompt.kind, SaveLoadKind::Load);
    assert_eq!(prompt.input, "");
}

#[test]
fn enter_on_the_load_item_opens_the_load_prompt() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Right));
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.phase, Phase::Menu));
    let prompt = app.save_prompt.as_ref().unwrap();
    assert_eq!(prompt.kind, SaveLoadKind::Load);
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
fn esc_from_the_game_returns_to_the_menu() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(matches!(app.phase, Phase::Playing));
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::Menu));
}

#[test]
fn q_from_the_game_keeps_the_game_and_asks_what_to_do() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    assert!(matches!(app.phase, Phase::Playing));
    app.handle_key(key(KeyCode::Char('q')));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Continue));
    assert!(matches!(app.phase, Phase::Playing));
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
        warp_app(&mut app, 60);
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
fn clicking_a_city_next_to_a_spent_unit_still_opens_the_city_window() {
    let (mut app, cx, cy) = playing_app();
    let city_id = app.engine.as_ref().unwrap().player_cities()[0].id();
    // Park a spent settler on a passable tile beside the city and give it
    // the focus.
    let (nx, ny) = [
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
        let nx = (cx as isize + dx).rem_euclid(w) as usize;
        let ny = (cy as isize + dy).clamp(0, h - 1) as usize;
        (nx, ny)
    })
    .find(|&(x, y)| {
        let engine = app.engine.as_ref().unwrap();
        let tile = engine.tile(x, y);
        tile.terrain.is_land() && tile.terrain.movement_cost() <= 1
    })
    .expect("some tile beside the city is passable");
    let parked = {
        let engine = app.engine.as_mut().unwrap();
        let unit = engine.game.spawn_unit(
            UnitClass::Settler,
            Location::new(nx as u16, ny as u16),
            PlayerId::new(0),
            CityId::new(0),
        );
        let index = engine
            .game
            .units
            .iter()
            .position(|u| u.id() == unit)
            .unwrap();
        engine.game.units[index].spend_turn();
        unit
    };
    app.selected_unit = Some(parked);
    assert_eq!(app.selected_city, None);

    // The click on the city must not be swallowed as a move attempt by the
    // spent neighbour: the window opens and the parked settler stays put.
    app.left_click(
        (LEFT_COLUMN_WIDTH as usize + cx * TILE_WIDTH) as u16,
        cy as u16,
    );
    assert_eq!(app.selected_city, Some(city_id));
    let engine = app.engine.as_ref().unwrap();
    let parked = engine
        .player_units()
        .into_iter()
        .find(|u| u.id() == parked)
        .unwrap();
    assert_eq!(
        (parked.location.x as usize, parked.location.y as usize),
        (nx, ny)
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
fn tile_under_pointer_maps_the_pointer_to_the_world() {
    // Pointer on the camera's own top-left tile.
    assert_eq!(
        tile_under_pointer((LEFT_COLUMN_WIDTH, 0), (4, 2), (80, 50)),
        Some((4, 2))
    );
    // Three tiles east and five south of the camera's top-left corner.
    assert_eq!(
        tile_under_pointer(
            (LEFT_COLUMN_WIDTH + 3 * TILE_WIDTH as u16, 5),
            (4, 2),
            (80, 50),
        ),
        Some((7, 7))
    );
    // Eastward wrap at the map's right edge.
    assert_eq!(
        tile_under_pointer(
            (LEFT_COLUMN_WIDTH + 78 * TILE_WIDTH as u16, 0),
            (0, 0),
            (80, 50)
        ),
        Some((78, 0))
    );
    assert_eq!(
        tile_under_pointer(
            (LEFT_COLUMN_WIDTH + 79 * TILE_WIDTH as u16, 0),
            (1, 0),
            (80, 50)
        ),
        Some((0, 0))
    );
    // The left column and the void below the map's south edge hold no tile.
    assert_eq!(tile_under_pointer((4, 2), (0, 0), (80, 50)), None);
    assert_eq!(
        tile_under_pointer(
            (LEFT_COLUMN_WIDTH + TILE_WIDTH as u16, 60),
            (0, 0),
            (80, 50),
        ),
        None
    );
}

#[test]
fn the_hovered_tile_follows_the_pointer_over_the_map() {
    let (mut app, cx, cy) = playing_app();
    // Camera at the origin, pointer a few tiles east and south of it.
    app.camera.set((0, 0));
    app.handle_mouse(mouse_event(
        MouseEventKind::Moved,
        LEFT_COLUMN_WIDTH + 3 * TILE_WIDTH as u16,
        7,
    ));
    assert_eq!(app.hovered_tile(app.engine.as_ref().unwrap()), Some((3, 7)));
    // The player's own city tile is a discovered tile like any other.
    app.handle_mouse(mouse_event(
        MouseEventKind::Moved,
        LEFT_COLUMN_WIDTH + cx as u16 * 2,
        cy as u16,
    ));
    assert_eq!(
        app.hovered_tile(app.engine.as_ref().unwrap()),
        Some((cx, cy))
    );
    // Over the left column there is no map tile to inspect.
    app.handle_mouse(mouse_event(MouseEventKind::Moved, 4, 7));
    assert_eq!(app.hovered_tile(app.engine.as_ref().unwrap()), None);
}

#[test]
fn a_modal_hides_the_hovered_tile() {
    let (mut app, _, _, _) = with_city_window_open();
    app.handle_mouse(mouse_event(
        MouseEventKind::Moved,
        LEFT_COLUMN_WIDTH + 3 * TILE_WIDTH as u16,
        7,
    ));
    // While the city window floats over the map the pointer belongs to the
    // window, so no tile is hovered for the focus panel.
    assert_eq!(app.hovered_tile(app.engine.as_ref().unwrap()), None);
    app.moused_window.set(None);
    assert_eq!(app.hovered_tile(app.engine.as_ref().unwrap()), Some((3, 7)));
}

#[test]
fn a_new_game_starts_with_no_city_selected() {
    let (app, _, _) = playing_app();
    assert_eq!(app.selected_city, None);
}

/// Opens the quit dialog over a fresh playing app, exactly as pressing q
/// would.
fn with_quit_dialog_open() -> (App, Rect) {
    let (mut app, _, _) = playing_app();
    app.handle_key(key(KeyCode::Char('q')));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Continue));
    let panel = quit_dialog::dialog_rect(Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 40,
    });
    app.quit_dialog_rect.set(Some(panel));
    (app, panel)
}

#[test]
fn q_in_play_opens_the_quit_dialog_instead_of_leaving_the_game() {
    let (mut app, _, _) = playing_app();
    assert!(!app.handle_key(key(KeyCode::Char('q'))));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Continue));
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn quit_dialog_cursors_cycle_continue_save_quit() {
    let (mut app, _) = with_quit_dialog_open();
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Save));
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Quit));
    app.handle_key(key(KeyCode::Right));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Continue));
    app.handle_key(key(KeyCode::Left));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Quit));
    app.handle_key(key(KeyCode::Char('l')));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Continue));
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.quit_dialog, Some(QuitChoice::Quit));
}

#[test]
fn esc_closes_the_quit_dialog_and_play_continues() {
    let (mut app, _) = with_quit_dialog_open();
    assert!(!app.handle_key(key(KeyCode::Esc)));
    assert_eq!(app.quit_dialog, None);
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn continue_confirms_and_stays_in_the_game() {
    let (mut app, _) = with_quit_dialog_open();
    assert!(!app.handle_key(key(KeyCode::Enter)));
    assert_eq!(app.quit_dialog, None);
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn choosing_save_from_the_quit_dialog_opens_the_save_prompt() {
    let (mut app, _) = with_quit_dialog_open();
    app.handle_key(key(KeyCode::Right));
    assert!(!app.handle_key(key(KeyCode::Enter)));
    assert_eq!(app.quit_dialog, None);
    let prompt = app.save_prompt.as_ref().expect("save prompt is open");
    assert_eq!(prompt.kind, SaveLoadKind::Save);
    assert!(matches!(app.phase, Phase::Playing));
}

#[test]
fn choosing_quit_from_the_quit_dialog_ends_the_process() {
    let (mut app, _) = with_quit_dialog_open();
    app.handle_key(key(KeyCode::Right));
    app.handle_key(key(KeyCode::Right));
    assert!(app.handle_key(key(KeyCode::Enter)));
}

/// The quit dialog's buttons are clickable: a click acts as if that answer
/// had been confirmed with the keyboard.
#[test]
fn clicking_a_quit_dialog_button_confirms_that_answer() {
    let (mut app, panel) = with_quit_dialog_open();
    app.left_click(
        quit_dialog::continue_button_rect(panel).x + 1,
        quit_dialog::continue_button_rect(panel).y,
    );
    assert_eq!(app.quit_dialog, None);
    assert!(matches!(app.phase, Phase::Playing));
    assert!(!app.exit_requested);

    let (mut app, panel) = with_quit_dialog_open();
    app.left_click(
        quit_dialog::save_button_rect(panel).x + 1,
        quit_dialog::save_button_rect(panel).y,
    );
    assert_eq!(app.quit_dialog, None);
    assert_eq!(app.save_prompt.as_ref().unwrap().kind, SaveLoadKind::Save);

    let (mut app, panel) = with_quit_dialog_open();
    app.left_click(
        quit_dialog::quit_button_rect(panel).x + 1,
        quit_dialog::quit_button_rect(panel).y,
    );
    assert_eq!(app.quit_dialog, None);
    assert!(app.exit_requested);
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
fn clicking_unfortify_returns_a_garrison_to_the_command_loop() {
    let (mut app, _, _) = playing_app();
    let (city_id, city_location) = {
        let engine = app.engine.as_ref().unwrap();
        let city = engine.player_cities()[0];
        (city.id(), city.location)
    };
    // A rested garrison: it fortified a previous turn, so its moves were
    // restored but the fortify order keeps it out of the available-units loop.
    let garrison = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Militia,
        city_location,
        PlayerId::new(0),
        city_id,
    );
    {
        let engine = app.engine.as_mut().unwrap();
        let unit = engine
            .game
            .units
            .iter_mut()
            .find(|u| u.id() == garrison)
            .unwrap();
        unit.fortify();
        unit.restore_moves();
        assert!(unit.moves_remaining() > 0);
    }
    let win = Rect {
        x: 20,
        y: 5,
        width: 60,
        height: 30,
    };
    app.moused_window
        .set(Some((win, crate::tui::city_window::close_button_rect(win))));
    app.selected_city = Some(city_id);
    let button = {
        let engine = app.engine.as_ref().unwrap();
        crate::tui::city_window::unfortify_button_rects(win, engine, city_id)[0].1
    };
    app.left_click(button.x + 1, button.y);

    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .game
        .units
        .iter()
        .find(|u| u.id() == garrison)
        .unwrap();
    assert_eq!(unit.order(), UnitOrder::Idle);
    assert!(
        unit.moves_remaining() > 0,
        "unfortify must not spend the garrison's turn"
    );
    // Back in the available-units loop: the next Tab lands on it.
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(garrison));
    // The two starting settlers (the human's and the rival's) spawned first,
    // so the garrison is the third unit in the world.
    assert_eq!(
        app.event_log.last().map(|event| event.message()),
        Some("Unit 2 is no longer fortified")
    );
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
    // y/i/n/, map to NW/NE/SW/SE around the J home key.
    for (code, tile_dx, tile_dy) in [
        (KeyCode::Char('y'), -1, -1),
        (KeyCode::Char('i'), 1, -1),
        (KeyCode::Char('n'), -1, 1),
        (KeyCode::Char(','), 1, 1),
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
fn the_home_row_keys_move_the_selected_unit_orthogonally() {
    // u/m/h/k map to N/S/W/E around the J home key; the arrow keys stay.
    for (code, tile_dx, tile_dy) in [
        (KeyCode::Char('u'), 0, -1),
        (KeyCode::Char('k'), 1, 0),
        (KeyCode::Char('m'), 0, 1),
        (KeyCode::Char('h'), -1, 0),
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
        let ny = before.y as isize + tile_dy;
        let in_bounds = ny >= 0 && ny < engine.height() as isize;
        let terrain = engine
            .tile(
                nx as usize,
                ny.clamp(0, engine.height() as isize - 1) as usize,
            )
            .terrain;
        if in_bounds && terrain.is_land() && terrain.movement_cost() <= 1 {
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
                "home-row key did not move the unit as expected: {before:?} -> {after:?}"
            );
        }
    }
}

/// An app on an 80x50 all-grassland map whose human settler sits one step
/// off the west edge tile, far from the bottom-right camera it starts at.
fn app_with_distant_settler() -> App {
    let mut app = App::new();
    app.phase = Phase::Playing;
    let mut engine = Engine::new(80, 50, Player::new(Civilization::English), vec![]);
    for y in 0..50 {
        for x in 0..80 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(5, 25),
        PlayerId::new(0),
        CityId::new(0),
    );
    app.engine = Some(engine);
    let settler = app.engine.as_ref().unwrap().player_units()[0].id();
    app.selected_unit = Some(settler);
    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    app.camera.set((70, 40));
    app.camera_follow.set(false);
    app
}

#[test]
fn j_centres_the_camera_on_the_selected_unit() {
    let mut app = app_with_distant_settler();
    app.handle_key(key(KeyCode::Char('j')));
    let expected = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine.player_units()[0];
        camera_for(
            Some((unit.location.x as usize, unit.location.y as usize)),
            (engine.width(), engine.height()),
            (200 / TILE_WIDTH, 40),
            (70, 40),
        )
    };
    assert_eq!(
        app.camera.get(),
        expected,
        "j re-centres the camera on the selected unit"
    );
    assert!(app.camera_follow.get(), "j resumes following the unit");
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

/// A small open grassland world with a single civilian human player (no
/// rivals), staging a settler on each of the given tiles. Returns the app
/// with its engine installed and the spawned unit ids in order.
fn app_with_settlers_at(tiles: &[(u16, u16)]) -> (App, Vec<UnitId>) {
    let mut app = App::new();
    app.phase = Phase::Playing;
    let mut engine = Engine::new(12, 3, Player::new(Civilization::English), vec![]);
    for y in 0..3 {
        for x in 0..12 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    let ids: Vec<UnitId> = tiles
        .iter()
        .map(|(x, y)| {
            engine.game.spawn_unit(
                UnitClass::Settler,
                Location::new(*x, *y),
                PlayerId::new(0),
                CityId::new(0),
            )
        })
        .collect();
    app.engine = Some(engine);
    (app, ids)
}

#[test]
fn tab_jumps_to_the_next_unit_that_still_has_movement() {
    let (mut app, ids) = app_with_settlers_at(&[(1, 1), (3, 1), (5, 1)]);
    // The first settler has already spent this turn's movement.
    app.engine.as_mut().unwrap().game.units[0].spend_turn();

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[1]));
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[2]));
    // Wrapping back lands on the first movable unit, never the spent one.
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[1]));
    assert!(app.event_log.is_empty());

    // Exhausting the second settler leaves the third as the only moveable.
    app.engine.as_mut().unwrap().game.units[1].spend_turn();
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[2]));

    // With every active unit spent, tab reports the turn is done rather than
    // silently re-selecting a spent unit.
    app.engine.as_mut().unwrap().game.units[2].spend_turn();
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(ids[2]));
    assert_eq!(
        app.event_log.last().map(|event| event.message()),
        Some("No more units left to command this turn")
    );
}

#[test]
fn tab_skips_fortified_sentried_and_loaded_units() {
    let (mut app, _ids) = app_with_settlers_at(&[(1, 1), (3, 1), (5, 1)]);
    {
        let engine = app.engine.as_mut().unwrap();
        engine.game.units[0].fortify();
        engine.game.units[1].sentry();
        let ship = engine.game.spawn_unit(
            UnitClass::Trireme,
            Location::new(7, 1),
            PlayerId::new(0),
            CityId::new(0),
        );
        // The settler at (5,1) rides the ship: no field agency of its own,
        // and the ship itself has already sailed this turn.
        engine.game.units[2].board(ship);
        let ship_index = engine
            .game
            .units
            .iter()
            .position(|unit| unit.id() == ship)
            .unwrap();
        engine.game.units[ship_index].spend_turn();
    }

    // Every unit is fortified, sentried, aboard the spent ship, or a spent
    // ship itself: none still holds map agency this turn.
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, None);
    assert_eq!(
        app.event_log.last().map(|event| event.message()),
        Some("No more units left to command this turn")
    );

    // A freshly idle unit makes tab work again instead of repeating the note —
    // the single message already logged stays put, nothing new is added.
    let extra = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Settler,
        Location::new(9, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.selected_unit, Some(extra));
    assert_eq!(app.event_log.len(), 1);
}

#[test]
fn a_spent_unit_advances_to_the_next_unit_with_movement_after_a_beat() {
    let (mut app, ids) = app_with_settlers_at(&[(1, 1), (3, 1), (5, 1)]);
    app.handle_key(key(KeyCode::Tab)); // ids[0]
    assert_eq!(app.selected_unit, Some(ids[0]));
    // One grassland step spends a settler's single move; a deadline is armed
    // but the focus stays on the spent unit until the deadline arrives.
    app.handle_key(key(KeyCode::Right));
    assert_eq!(
        app.engine.as_ref().unwrap().player_units()[0].moves_remaining(),
        0
    );
    assert!(app.unit_advance_deadline.is_some());
    assert_eq!(app.selected_unit, Some(ids[0]));
    fire_pending_unit_advance(&mut app);
    assert_eq!(app.selected_unit, Some(ids[1]));
    assert!(app.unit_advance_deadline.is_none());

    // The next spent unit advances the focus again...
    app.handle_key(key(KeyCode::Right));
    assert_eq!(
        app.engine.as_ref().unwrap().player_units()[1].moves_remaining(),
        0
    );
    fire_pending_unit_advance(&mut app);
    assert_eq!(app.selected_unit, Some(ids[2]));

    // ...until the last unit spends itself: then the wait just reports that
    // the turn is done instead of landing anywhere.
    app.handle_key(key(KeyCode::Right));
    fire_pending_unit_advance(&mut app);
    assert_eq!(app.selected_unit, Some(ids[2]));
    assert_eq!(
        app.event_log.last().map(|event| event.message()),
        Some("No more units left to command this turn")
    );
    assert!(app.unit_advance_deadline.is_none());
}

#[test]
fn a_unit_with_movement_left_keeps_the_focus() {
    let (mut app, _ids) = app_with_settlers_at(&[(1, 1), (3, 1), (5, 1)]);
    // A fast rider keeps budget after a single step, so nothing arms at all.
    let cavalry = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Cavalry,
        Location::new(7, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Tab));
    }
    assert_eq!(app.selected_unit, Some(cavalry));
    app.handle_key(key(KeyCode::Right));
    let rider = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == cavalry)
        .unwrap();
    assert_eq!(rider.moves_remaining(), 2);
    assert!(app.unit_advance_deadline.is_none());
    // Waiting out the delay changes nothing while the focus still has budget.
    fire_pending_unit_advance(&mut app);
    assert_eq!(app.selected_unit, Some(cavalry));
}

#[test]
fn founding_a_city_with_the_selected_settler_advances_the_focus() {
    let (mut app, ids) = app_with_settlers_at(&[(1, 1), (3, 1), (5, 1)]);
    app.handle_key(key(KeyCode::Tab)); // ids[0]
    assert_eq!(app.selected_unit, Some(ids[0]));
    app.handle_key(key(KeyCode::Char('v'))); // found a city with the settler
    assert_eq!(
        app.engine.as_ref().unwrap().player_units().len(),
        2,
        "the founding settler leaves the field"
    );
    assert!(app.unit_advance_deadline.is_some());
    fire_pending_unit_advance(&mut app);
    // The focus moved off the vanished settler onto one of the surviving
    // settlers that still has budget (list order is engine-internal, so only
    // membership, not which one, is guaranteed).
    let landed = app.selected_unit.expect("a survivor is focused");
    assert!(ids.contains(&landed) && landed != ids[0]);
    assert!(app.unit_advance_deadline.is_none());
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
    assert!(commands.iter().any(|(k, _)| *k == "u/k/h/m"));
    assert!(commands.iter().any(|(k, _)| *k == "y/i/n/,"));
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
        warp_app(&mut app, 60);
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
    // Park the rival's starting settler so its (now live) AI turn has nothing
    // to build or march with. These scenarios stage a rival that stays put;
    // without this the rival founds a city and its militia abandons its tile
    // to garrison it, which the tests below rely on being a stable state.
    // Commands are refused for a unit the engine does not currently steer, so
    // set the order directly on the model like the tests otherwise do.
    {
        let engine = app.engine.as_mut().unwrap();
        for unit in engine.game.units.iter_mut() {
            if unit.owner() == PlayerId::new(1) && unit.unit_class == UnitClass::Settler {
                unit.fortify();
                unit.spend_turn();
            }
        }
    }
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
fn pressing_w_opens_the_command_picker_for_the_selected_settler() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    assert!(app.command_picker_open);
    assert_eq!(app.command_picker_cursor, 0);
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.command_picker_open);
    assert_eq!(app.command_picker_rect.get(), None);
}

#[test]
fn a_game_that_has_founded_its_first_city_has_no_unit_left_to_command() {
    let (mut app, _, _) = playing_app();
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.command_picker_open);
}

#[test]
fn the_command_picker_lists_orders_and_the_buildable_improvements() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let unit = app.engine.as_ref().unwrap().player_units()[0];
    let tile = app
        .engine
        .as_ref()
        .unwrap()
        .tile(unit.location.x as usize, unit.location.y as usize);
    let rows = app.command_rows();
    assert_eq!(rows[0], command_picker::CommandChoice::Fortify);
    assert_eq!(rows[1], command_picker::CommandChoice::Sentry);
    let improvements: Vec<TerrainImprovement> = rows
        .iter()
        .filter_map(|command| match command {
            command_picker::CommandChoice::Work(improvement) => Some(*improvement),
            _ => None,
        })
        .collect();
    assert_eq!(
        improvements,
        command_picker::buildable_improvements(tile),
        "the work rows are exactly what the settler can build"
    );
    assert!(
        improvements.contains(&TerrainImprovement::Road),
        "a settler on land can always build a road: {improvements:?}"
    );
}

#[test]
fn saving_the_command_picker_orders_the_settler_and_the_road_lands_next_turn() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let road = app
        .command_rows()
        .iter()
        .position(|command| {
            matches!(
                command,
                command_picker::CommandChoice::Work(TerrainImprovement::Road)
            )
        })
        .expect("a settler on land can always build a road");
    for _ in 0..road {
        app.handle_key(key(KeyCode::Char('j')));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(!app.command_picker_open);
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
fn esc_closes_the_command_picker_without_an_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.command_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.player_units()[0].order(), UnitOrder::Idle);
}

#[test]
fn the_command_picker_requires_a_real_selected_unit() {
    let mut app = app_with_settler();
    app.selected_unit = None;
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.command_picker_open);
    // A selection that picks out no living unit must not open either.
    app.selected_unit = Some(crate::model::units::UnitId::new(999));
    app.handle_key(key(KeyCode::Char('w')));
    assert!(!app.command_picker_open);
}

#[test]
fn pressing_f_fortifies_the_selected_unit() {
    let mut app = app_with_settler();
    let selected = app.selected_unit.expect("a unit is selected");
    app.handle_key(key(KeyCode::Char('f')));
    let engine = app.engine.as_ref().unwrap();
    let unit = engine
        .game
        .units
        .iter()
        .find(|u| u.id() == selected)
        .unwrap();
    assert_eq!(unit.order(), UnitOrder::Fortified);
    assert_eq!(unit.moves_remaining(), 0, "fortifying spends the turn");
    let expected = format!("Unit {} fortifies", selected.index());
    assert_eq!(
        app.event_log.last().map(|event| event.message()),
        Some(expected.as_str())
    );
    // A fortified unit left the loop: the auto-advance is armed to move on.
    assert!(
        app.unit_advance_deadline.is_some(),
        "a spent focus arms the pending unit advance"
    );
}

#[test]
fn pressing_c_cancels_the_selected_units_work_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let (work, first) = {
        let rows = app.command_rows();
        let index = rows
            .iter()
            .position(|command| matches!(command, command_picker::CommandChoice::Work(_)))
            .expect("a settler on land can build something");
        let first = match rows[index] {
            command_picker::CommandChoice::Work(improvement) => improvement,
            _ => unreachable!("position filtered for a work row"),
        };
        (index, first)
    };
    for _ in 0..work {
        app.handle_key(key(KeyCode::Char('j')));
    }
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
fn the_command_picker_swallows_game_keys() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let turn = app.engine.as_ref().unwrap().turn();
    for code in [KeyCode::Char('q'), KeyCode::Char('m'), KeyCode::Char(' ')] {
        let was_quit = app.handle_key(key(code));
        assert!(!was_quit, "command picker must capture {code:?}");
    }
    assert!(app.command_picker_open);
    assert!(matches!(app.phase, Phase::Playing));
    assert_eq!(
        app.engine.as_ref().unwrap().turn(),
        turn,
        "no end turn while the picker is open"
    );
}

#[test]
fn clicking_a_row_then_save_issues_the_command() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        assert!(app.command_picker_rect.get().is_some());
        terminal.size().unwrap()
    };
    let panel = command_picker::command_picker_rect(area.into());
    let rows = command_picker::rows_rect(panel);
    app.left_click(rows.x + 1, rows.y);
    assert_eq!(app.command_picker_cursor, 0);
    let save = command_picker::save_button_rect(panel);
    app.left_click(save.x + save.width / 2, save.y.max(1));
    assert!(!app.command_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        engine.player_units()[0].order(),
        UnitOrder::Fortified,
        "the first command row fortifies the selected unit"
    );
}

#[test]
fn clicking_a_row_then_save_issues_the_chosen_work_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let road = app
        .command_rows()
        .iter()
        .position(|command| {
            matches!(
                command,
                command_picker::CommandChoice::Work(TerrainImprovement::Road)
            )
        })
        .expect("a settler on land can always build a road");
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        terminal.size().unwrap()
    };
    let panel = command_picker::command_picker_rect(area.into());
    let rows = command_picker::rows_rect(panel);
    app.left_click(rows.x + 1, rows.y + road as u16);
    assert_eq!(app.command_picker_cursor, road);
    let save = command_picker::save_button_rect(panel);
    app.left_click(save.x + save.width / 2, save.y.max(1));
    assert!(!app.command_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        engine.player_units()[0].order(),
        UnitOrder::Improving(TerrainImprovement::Road)
    );
}

#[test]
fn clicking_cancel_closes_the_command_picker_without_an_order() {
    let mut app = app_with_settler();
    app.handle_key(key(KeyCode::Char('w')));
    let area = {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        terminal.size().unwrap()
    };
    let panel = command_picker::command_picker_rect(area.into());
    let cancel = command_picker::cancel_button_rect(panel);
    app.left_click(cancel.x + cancel.width / 2, cancel.y.max(1));
    assert!(!app.command_picker_open);
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.player_units()[0].order(), UnitOrder::Idle);
}

#[test]
fn clicking_a_player_unit_selects_it_and_opens_its_command_picker() {
    let mut app = app_with_settler();
    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    let (wx, wy) = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .player_units()
            .into_iter()
            .find(|u| u.unit_class == UnitClass::Settler)
            .unwrap();
        (unit.location.x as usize, unit.location.y as usize)
    };
    app.left_click((LEFT_COLUMN_WIDTH as usize + wx * 2) as u16, wy as u16);
    assert!(
        app.command_picker_open,
        "clicking an own unit opens its command window"
    );
    let engine = app.engine.as_ref().unwrap();
    let unit = engine
        .player_units()
        .into_iter()
        .find(|u| u.unit_class == UnitClass::Settler)
        .unwrap();
    assert_eq!(app.selected_unit, Some(unit.id()));
    assert_eq!(app.selected_city, None);
    assert!(app.camera_follow.get());
}

#[test]
fn clicking_a_city_tile_prefers_the_city_window_over_a_unit_on_it() {
    let (mut app, cx, cy) = playing_app();
    let (city_id, city_location) = {
        let engine = app.engine.as_ref().unwrap();
        let city = engine.player_cities()[0];
        (city.id(), city.location)
    };
    let garrison = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Militia,
        city_location,
        PlayerId::new(0),
        city_id,
    );
    app.selected_unit = Some(garrison);
    app.open_command_picker();
    assert!(app.command_picker_open);
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    assert_eq!(app.selected_city, Some(city_id));
    assert!(
        !app.command_picker_open,
        "the city window wins over an occupying unit"
    );
}

#[test]
fn the_command_picker_offers_unfortify_for_a_fortified_garrison() {
    let (mut app, _, _) = playing_app();
    let (city_id, city_location) = {
        let engine = app.engine.as_ref().unwrap();
        let city = engine.player_cities()[0];
        (city.id(), city.location)
    };
    // A rested garrison: it fortified a previous turn, so its moves were
    // restored but the fortify order keeps it out of the available-units loop.
    let garrison = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Militia,
        city_location,
        PlayerId::new(0),
        city_id,
    );
    {
        let engine = app.engine.as_mut().unwrap();
        let unit = engine
            .game
            .units
            .iter_mut()
            .find(|u| u.id() == garrison)
            .unwrap();
        unit.fortify();
        unit.restore_moves();
    }
    app.selected_unit = Some(garrison);
    assert_eq!(
        app.command_rows(),
        vec![command_picker::CommandChoice::Unfortify],
        "a fortified garrison can only be roused"
    );
}

#[test]
fn the_load_prompt_accumulates_typed_paths_and_esc_closes_it() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Char('l')));
    for ch in "/tmp/saved.civ".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    assert_eq!(app.save_prompt.as_ref().unwrap().input, "/tmp/saved.civ");
    app.handle_key(key(KeyCode::Backspace));
    assert_eq!(app.save_prompt.as_ref().unwrap().input, "/tmp/saved.ci");
    app.handle_key(key(KeyCode::Esc));
    assert!(app.save_prompt.is_none());
}

#[test]
fn saving_writes_the_game_to_disk() {
    let (mut app, _, _) = playing_app();
    let file = std::env::temp_dir().join(format!("civterm-app-save-{}.civ", std::process::id()));
    let path = file.to_string_lossy().to_string();
    app.open_save_prompt();
    for _ in 0.."civterm.civ".len() {
        app.handle_key(key(KeyCode::Backspace));
    }
    for ch in path.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(app.save_prompt.is_none());
    assert!(file.exists());
    let _ = std::fs::remove_file(file);
}

#[test]
fn an_empty_save_path_keeps_the_prompt_open_with_an_error() {
    let (mut app, _, _) = playing_app();
    app.open_save_prompt();
    for _ in 0..("civterm.civ".len() + 1) {
        app.handle_key(key(KeyCode::Backspace));
    }
    app.handle_key(key(KeyCode::Enter));
    let prompt = app.save_prompt.as_ref().unwrap();
    assert_eq!(prompt.error.as_deref(), Some("Enter a path"));
    assert_eq!(prompt.kind, SaveLoadKind::Save);
}

#[test]
fn loading_a_missing_file_keeps_the_prompt_open_with_an_error() {
    let mut app = App::new();
    app.handle_key(key(KeyCode::Char('l')));
    let target = std::env::temp_dir().join("civterm-no-such-app-file.civ");
    let path = target.to_string_lossy().to_string();
    for ch in path.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.phase, Phase::Menu));
    let prompt = app.save_prompt.as_ref().unwrap();
    assert!(prompt.error.is_some());
}

#[test]
fn a_saved_game_loads_back_into_play() {
    let (mut app, _, _) = playing_app();
    let file =
        std::env::temp_dir().join(format!("civterm-app-roundtrip-{}.civ", std::process::id()));
    let path = file.to_string_lossy().to_string();
    app.open_save_prompt();
    for _ in 0.."civterm.civ".len() {
        app.handle_key(key(KeyCode::Backspace));
    }
    for ch in path.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(app.save_prompt.is_none());
    assert!(file.exists());

    // Quit to the menu and load the saved game back in.
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.phase, Phase::Menu));
    app.handle_key(key(KeyCode::Char('l')));
    for ch in path.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.phase, Phase::Playing));
    assert!(app.save_prompt.is_none());
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(engine.current_player(), Civilization::American);
    assert!(!engine.player_cities().is_empty());
    let _ = std::fs::remove_file(file);
}

/// A game on a small open grassland world where the rival has a city and a
/// legion that must march to garrison it. The human keeps only a settler.
/// `reveal` optionally marks some human-explored tiles; with a radius-8
/// reveal over the march the whole 15x3 map counts as explored.
fn app_with_marching_rival(reveal: Option<(Location, u8)>) -> App {
    let mut app = App::new();
    app.phase = Phase::Playing;
    let mut engine = Engine::new(
        15,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..15 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    engine.game.spawn_unit(
        UnitClass::Settler,
        Location::new(1, 0),
        PlayerId::new(0),
        CityId::new(0),
    );
    engine.game.cities.push(City::new(
        "Ulundi",
        Location::new(14, 1),
        PlayerId::new(1),
        CityId::new(1),
    ));
    engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(4, 1),
        PlayerId::new(1),
        CityId::new(0),
    );
    if let Some((origin, radius)) = reveal {
        engine.game.players[0].reveal_tiles_at(origin, radius);
    }
    app.engine = Some(engine);
    app
}

#[test]
fn ending_the_turn_starts_the_rival_replay_from_the_rounds_motion() {
    let mut app = app_with_marching_rival(Some((Location::new(7, 1), 8)));
    let legion = {
        let engine = app.engine.as_ref().unwrap();
        engine
            .game
            .units
            .iter()
            .find(|unit| unit.owner() == PlayerId::new(1))
            .unwrap()
            .id()
    };
    assert!(app.rival_animation.is_none());

    app.handle_key(key(KeyCode::Char(' '))); // end the turn: the legion marches

    let animation = app
        .rival_animation
        .as_ref()
        .expect("a round that moved rival units fires the replay");
    assert!(
        !animation.frames.is_empty(),
        "the garrison march is replayed step by step"
    );
    assert!(
        animation.frames.iter().all(|frame| frame.unit_id == legion),
        "every frame is the legion's"
    );
    let stationed = app
        .engine
        .as_ref()
        .unwrap()
        .game
        .units
        .iter()
        .find(|unit| unit.id() == legion)
        .unwrap();
    assert_eq!(
        animation.frames.last().unwrap().to,
        stationed.location,
        "the final frame ends where the legion now stands"
    );
    assert_eq!(
        app.engine.as_ref().unwrap().current_player(),
        Civilization::English
    );
}

#[test]
fn game_keys_are_idle_while_the_rival_replay_runs() {
    let mut app = app_with_marching_rival(None);
    let unit_id = app.engine.as_ref().unwrap().player_units()[0].id();
    app.rival_animation = Some(RivalMoveAnimation {
        start: Duration::ZERO,
        frames: vec![RivalMoveFrame {
            unit_id,
            from: Location::new(1, 0),
            to: Location::new(2, 0),
        }],
    });

    // No game key is honoured while the replay runs.
    app.handle_key(key(KeyCode::Char('?')));
    assert!(!app.show_help, "the help toggle is idle during the replay");
    app.handle_key(key(KeyCode::Char('k'))); // would move the settler east
    let settler = app.engine.as_ref().unwrap().player_units()[0];
    assert_eq!(settler.location, Location::new(1, 0));

    // Once the replay has run its course the keys work again.
    warp_app(&mut app, 60);
    app.handle_key(key(KeyCode::Char('?')));
    assert!(app.show_help, "the help toggle works after the replay");
}

#[test]
fn the_rival_replay_pans_the_camera_to_the_moving_unit() {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    app.camera.set((0, 0));
    app.camera_follow.set(false); // the player has dragged the map by hand
    let unit_id = app.engine.as_ref().unwrap().player_units()[0].id();
    let width = app.engine.as_ref().unwrap().width();

    // A step far to the east moves the camera even though the player is not
    // in follow mode: the replay is the focus while it plays.
    app.rival_animation = Some(RivalMoveAnimation {
        start: Duration::ZERO,
        frames: vec![RivalMoveFrame {
            unit_id,
            from: Location::new((width - 20) as u16, 5),
            to: Location::new((width - 19) as u16, 5),
        }],
    });
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    assert!(
        app.camera.get().0 > 0,
        "the replay pans east toward the unit"
    );

    // A step inside the central 70% leaves the camera alone.
    app.camera.set((4, 4));
    app.rival_animation = Some(RivalMoveAnimation {
        start: Duration::ZERO,
        frames: vec![RivalMoveFrame {
            unit_id,
            from: Location::new(24, 24),
            to: Location::new(25, 24),
        }],
    });
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    assert_eq!(app.camera.get(), (4, 4), "an in-band step does not pan");
}

#[test]
fn a_rival_march_within_unexplored_territory_never_replays() {
    let mut app = app_with_marching_rival(None);
    assert!(app.rival_animation.is_none());

    app.handle_key(key(KeyCode::Char(' '))); // the legion marches unseen

    assert!(
        app.rival_animation.is_none(),
        "a march the human cannot trace stays in the fog"
    );
    // The round itself still resolved: the legion moved, the turn wrapped.
    assert_eq!(
        app.engine.as_ref().unwrap().current_player(),
        Civilization::English
    );
}

#[test]
fn a_rival_step_arriving_in_sight_is_replayed() {
    // The garrison legion marches a single deterministic step from (4,1) to
    // (3,2). The human has spied only the destination (2..3 wrapped column
    // patch around it), so the step is shown emerging from the fog.
    let mut app = app_with_marching_rival(Some((Location::new(2, 2), 1)));

    app.handle_key(key(KeyCode::Char(' ')));

    let animation = app
        .rival_animation
        .as_ref()
        .expect("a step whose destination is explored is replayed");
    assert_eq!(animation.frames.len(), 1);
    let frame = &animation.frames[0];
    assert_eq!(frame.from, Location::new(4, 1));
    assert_eq!(frame.to, Location::new(3, 2));
    let explored = |loc: Location| {
        app.engine
            .as_ref()
            .unwrap()
            .explored(loc.x as usize, loc.y as usize)
    };
    assert!(
        !explored(frame.from) && explored(frame.to),
        "the march emerges from the fog into the explored tile"
    );
    assert_eq!(
        frame.to,
        app.engine
            .as_ref()
            .unwrap()
            .game
            .units
            .iter()
            .find(|unit| unit.owner() == PlayerId::new(1))
            .unwrap()
            .location,
        "the replayed arrival is where the legion now stands"
    );
}

#[test]
fn a_rival_step_leaving_sight_is_replayed() {
    // The human has spied the starting tile (around (4,1)) but nothing the
    // legion walks onto, so the single step is shown vanishing into the fog.
    let mut app = app_with_marching_rival(Some((Location::new(5, 2), 1)));

    app.handle_key(key(KeyCode::Char(' ')));

    let animation = app
        .rival_animation
        .as_ref()
        .expect("a step whose starting tile is explored is replayed");
    assert_eq!(animation.frames.len(), 1);
    let frame = &animation.frames[0];
    assert_eq!(frame.from, Location::new(4, 1));
    assert_eq!(frame.to, Location::new(3, 2));
    let explored = |loc: Location| {
        app.engine
            .as_ref()
            .unwrap()
            .explored(loc.x as usize, loc.y as usize)
    };
    assert!(
        explored(frame.from) && !explored(frame.to),
        "the march leaves the explored tile and vanishes into the fog"
    );
}

#[test]
fn clicking_close_while_the_picker_is_open_closes_the_window() {
    // The production picker floats over the window's middle, never covering
    // the top-right "✕ Close", and swallows most clicks while up — but the
    // close button must stay live: clicking it dismisses the window and the
    // picker with it instead of doing nothing.
    let (mut app, panel, _) = with_production_picker_open();
    let Some((win, close)) = app.moused_window.get() else {
        panic!("no moused window");
    };
    assert!(
        !panel.contains((close.x, close.y).into()),
        "the picker must not cover the close button"
    );

    app.left_click(close.x + 3, close.y);

    assert_eq!(
        app.selected_city, None,
        "close must dismiss the city window while the picker is open"
    );
    assert!(
        !app.production_picker_open,
        "closing the window takes its picker down too"
    );
    let _ = win;
}

#[test]
fn clicking_the_painted_close_button_closes_the_window() {
    // The close button's painted position must match its hit rect: the city
    // window widget centres itself over the area it is given, so if the app
    // hands it the clamped window rect the box lands a cell off from where
    // `close_button_rect` (and every overlay) is computed. This test draws
    // the real frame, clicks the painted ✕ exactly where it appears on
    // screen, and expects the window to close.
    let (mut app, _, _) = playing_app();
    let (cx, cy) = {
        let engine = app.engine.as_ref().unwrap();
        let city = engine.player_cities()[0];
        (city.location.x as usize, city.location.y as usize)
    };
    app.left_click((LEFT_COLUMN_WIDTH as usize + cx * 2) as u16, cy as u16);
    assert!(app.selected_city.is_some());
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    let (window, close) = app.moused_window.get().expect("moused_window");
    let painted = (0..40u16)
        .flat_map(|y| (0..120u16).map(move |x| (x, y)))
        .find(|(x, y)| buffer.cell((*x, *y)).is_some_and(|c| c.symbol() == "✕"))
        .expect("close button is painted");
    assert!(
        close.contains(painted.into()),
        "painted ✕ at {painted:?} must be inside the hit rect {close:?} (window {window:?})"
    );

    app.left_click(painted.0, painted.1);
    assert_eq!(
        app.selected_city, None,
        "clicking the painted ✕ must close the window"
    );
}
