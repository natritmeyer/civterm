use super::*;
use crate::game_engine::{Command, DiplomatAction, Player, SabotageNotice, StealOutcome};
use crate::model::advancements::Advancement;
use crate::model::cartography::Location;
use crate::model::cities::{City, CityImprovement, ProductionTarget};
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

/// Run one draw pass. The floating windows record the rectangle they were drawn
/// into, and that rectangle is what the mouse and hover guards match against, so
/// a test that wants either has to draw first.
fn draw(app: &App) {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, app)).unwrap();
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
fn a_floating_window_holds_the_unit_flash_off() {
    // Plain play leaves the idle-unit flash live; opening any floating window
    // or dialog suppresses it (`GameScreen::with_flash_enabled` is driven off
    // `window_is_open`), and closing the panel re-arms it.
    let (mut app, _, _) = playing_app();
    assert!(!window_is_open(&app), "plain play keeps the flash live");

    app.selected_city = Some(app.engine.as_ref().unwrap().player_cities()[0].id());
    assert!(window_is_open(&app), "the city window counts as a window");
    app.selected_city = None;

    app.command_picker_open = true;
    assert!(
        window_is_open(&app),
        "the command picker counts as a window"
    );
    app.command_picker_open = false;

    app.quit_dialog = Some(QuitChoice::Continue);
    assert!(window_is_open(&app), "the quit dialog counts as a window");
    app.quit_dialog = None;

    app.save_prompt = Some(SaveLoadState {
        kind: SaveLoadKind::Load,
        input: String::new(),
        error: None,
    });
    assert!(
        window_is_open(&app),
        "the save-load prompt counts as a window"
    );
    app.save_prompt = None;

    app.research_dialog = Some(ResearchDialogState {
        discovered: crate::model::advancements::Advancement::Wheel,
        choices: vec![crate::model::advancements::Advancement::Wheel],
        cursor: 0,
        scroll: 0,
    });
    assert!(
        window_is_open(&app),
        "the research dialog counts as a window"
    );
    app.research_dialog = None;

    app.diplomacy = Some(DiplomacyState {
        opponent: crate::model::civilizations::PlayerId::new(1),
        origin: DiplomacyOrigin::Contact,
        choice: DiplomacyChoice::Peace,
        pending: None,
    });
    assert!(
        window_is_open(&app),
        "the diplomacy dialog counts as a window"
    );
    app.diplomacy = None;

    app.steal_notice = Some(StealOutcome::Stolen {
        advancement: Advancement::Wheel,
        rival: "Zulu".to_string(),
    });
    assert!(
        window_is_open(&app),
        "the technology window counts as a window"
    );
    app.steal_notice = None;

    app.sabotage_notice = Some(SabotageNotice {
        improvement: CityImprovement::CityWalls,
        city_name: "Umgungundlovu".to_string(),
    });
    assert!(
        window_is_open(&app),
        "the sabotage window counts as a window"
    );
    app.sabotage_notice = None;

    assert!(
        !window_is_open(&app),
        "closing everything re-arms the flash"
    );
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
            Some(CityId::new(0)),
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

/// Plants a fresh militia on the player's city tile, for tests that need a
/// unit to move or to be selected. `playing_app` spends the starting settler
/// founding the city, so nothing is left in the field without this.
fn spawn_militia_at_city(app: &mut App) -> UnitId {
    let (location, city_id) = {
        let city = app.engine.as_ref().unwrap().player_cities()[0];
        (city.location, city.id())
    };
    app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Militia,
        location,
        PlayerId::new(0),
        Some(city_id),
    )
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
        Some(city_id),
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
    // Back in the available-units loop. The city window still owns the
    // keyboard, so Tab is ignored while it is open; it lands on the garrison
    // once the window is dismissed.
    app.handle_key(key(KeyCode::Tab));
    assert_ne!(
        app.selected_unit,
        Some(garrison),
        "Tab must not reach the map while the city window is open"
    );
    app.handle_key(key(KeyCode::Esc));
    assert!(app.selected_city.is_none());
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

/// The reported bug: with a city window up, space fell through to the
/// end-turn key and handed the whole round to the rivals.
#[test]
fn space_does_not_end_the_turn_while_the_city_window_is_open() {
    let (mut app, _, _, _) = with_city_window_open();
    let before = app.engine.as_ref().unwrap().turn();
    app.handle_key(key(KeyCode::Char(' ')));
    assert_eq!(
        app.engine.as_ref().unwrap().turn(),
        before,
        "space must not advance the turn behind the city window"
    );
    assert!(
        app.selected_city.is_some(),
        "space is swallowed; the window is still up"
    );
    assert_eq!(
        app.engine.as_ref().unwrap().current_player_id(),
        PlayerId::new(0),
        "the player still holds the turn"
    );
}

/// Enter shares the end-turn key, so it is swallowed too.
#[test]
fn enter_does_not_end_the_turn_while_the_city_window_is_open() {
    let (mut app, _, _, _) = with_city_window_open();
    let before = app.engine.as_ref().unwrap().turn();
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.engine.as_ref().unwrap().turn(), before);
}

/// Only Esc is a city-window key. Every map command is inert while it is up,
/// so a stray letter cannot move a unit or found a city behind the panel.
#[test]
fn map_commands_are_inert_while_the_city_window_is_open() {
    let (mut app, _, _, _) = with_city_window_open();
    let unit_id = spawn_militia_at_city(&mut app);
    let snapshot = {
        let engine = app.engine.as_ref().unwrap();
        let unit = engine
            .game
            .units
            .iter()
            .find(|u| u.id() == unit_id)
            .unwrap();
        (unit_id, unit.location, engine.player_cities().len())
    };
    let focus_before = app.selected_unit;
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Char('u'),
        KeyCode::Char('h'),
        KeyCode::Char('m'),
        KeyCode::Char('k'),
        KeyCode::Tab,
        KeyCode::Char('v'),
        KeyCode::Char('f'),
        KeyCode::Char('c'),
        KeyCode::Char('w'),
        KeyCode::Char('j'),
    ] {
        app.handle_key(key(code));
    }
    let engine = app.engine.as_ref().unwrap();
    let unit = engine
        .game
        .units
        .iter()
        .find(|u| u.id() == snapshot.0)
        .expect("the unit is still in the world");
    assert_eq!(
        unit.location, snapshot.1,
        "no unit may move behind the window"
    );
    assert_eq!(
        engine.player_cities().len(),
        snapshot.2,
        "no city may be founded behind the window"
    );
    assert_eq!(
        app.selected_unit, focus_before,
        "no key may move the selection behind the window"
    );
    assert!(app.selected_city.is_some(), "the window survives every key");
}

/// Quit and help are app-level toggles, not map commands, but the window is
/// modal all the same: the player dismisses it first.
#[test]
fn the_city_window_swallows_the_quit_and_help_keys() {
    let (mut app, _, _, _) = with_city_window_open();
    app.handle_key(key(KeyCode::Char('q')));
    assert!(
        app.quit_dialog.is_none(),
        "quit must not open behind the window"
    );
    app.handle_key(key(KeyCode::Char('?')));
    assert!(!app.show_help, "help must not open behind the window");
}

/// A press on the map pane beside the window must not become a click: the
/// press dismisses the window and nothing else. The unit is planted on the
/// tile north of the press, so a click that reached the map would move it.
#[test]
fn a_click_outside_the_city_window_does_not_move_a_unit() {
    let (mut app, _, _, win) = with_city_window_open();
    // Pick a screen cell in the map pane, then read back the world tile it
    // stands for, exactly as `map_click` maps it. The unit goes north of it,
    // one square away, which is what makes the click a move.
    let column = LEFT_COLUMN_WIDTH + 4;
    let row = 4;
    let (world_x, world_y) = {
        let engine = app.engine.as_ref().unwrap();
        let (camera_x, camera_y) = app.camera.get();
        let tile_col = (column as usize - LEFT_COLUMN_WIDTH as usize) / TILE_WIDTH;
        let world_x = (camera_x + tile_col) % engine.width();
        let world_y = camera_y + row as usize;
        assert!(world_y > 0, "the test needs a tile north of the press");
        (world_x, world_y)
    };
    let unit_id = {
        let city_id = {
            let engine = app.engine.as_ref().unwrap();
            engine.player_cities()[0].id()
        };
        let engine = app.engine.as_mut().unwrap();
        engine.game.spawn_unit(
            UnitClass::Militia,
            crate::model::cartography::Location::new(world_x as u16, world_y as u16 - 1),
            PlayerId::new(0),
            Some(city_id),
        )
    };
    let home = {
        let engine = app.engine.as_ref().unwrap();
        engine
            .game
            .units
            .iter()
            .find(|u| u.id() == unit_id)
            .unwrap()
            .location
    };
    app.selected_unit = Some(unit_id);
    assert!(
        !win.contains((column, row).into()),
        "the press is off the window"
    );
    app.left_click(column, row);

    assert!(
        app.selected_city.is_none(),
        "a press outside dismisses the window"
    );
    let engine = app.engine.as_ref().unwrap();
    let unit = engine
        .game
        .units
        .iter()
        .find(|u| u.id() == unit_id)
        .expect("the unit is still in the world");
    assert_eq!(
        unit.location, home,
        "the dismissing click must not also act on the map tile at ({world_x}, {world_y})"
    );
}

/// A click that begins beside the window and ends on the map is a drag, and a
/// drag needs a press on the pane. With the window up no press reaches the
/// pane, so the camera stays put.
#[test]
fn a_drag_cannot_start_outside_the_city_window() {
    let (mut app, _, _, win) = with_city_window_open();
    let camera = app.camera.get();
    let inside = (win.x + 1, win.y + 1);
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        inside.0,
        inside.1,
    ));
    app.handle_mouse(mouse_event(
        MouseEventKind::Drag(MouseButton::Left),
        inside.0 + 6,
        inside.1 + 2,
    ));
    assert_eq!(app.camera.get(), camera, "a drag must not pan the camera");
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
        Some(CityId::new(0)),
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
                Some(CityId::new(0)),
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
            Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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

/// The contact window is modal, so the move that opened it could not arm the
/// auto-advance. Closing it returns the player to the ordinary run loop: a
/// unit spent by that move hands the focus on to the next unit with budget.
#[test]
fn closing_a_contact_diplomacy_window_resumes_the_unit_auto_advance() {
    let Some((mut app, settler_id, direction, _)) = app_with_unknown_neighbor() else {
        return;
    };
    let home = {
        let engine = app.engine.as_ref().unwrap();
        engine
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == settler_id)
            .map(|unit| unit.location)
            .expect("the settler is still on the map")
    };
    let extra = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Settler,
        home,
        PlayerId::new(0),
        None,
    );
    app.move_selected_unit(direction); // meets the rival, dialog opens
    assert!(app.diplomacy.is_some(), "first contact prompts diplomacy");
    assert!(
        app.unit_advance_deadline.is_none(),
        "the modal window keeps a pending jump from firing under it"
    );
    app.handle_key(key(KeyCode::Enter)); // PEACE
    assert!(app.diplomacy.is_none(), "the dialog has closed");
    assert!(
        app.unit_advance_deadline.is_some(),
        "closing the window re-arms the jump to the next unit"
    );
    fire_pending_unit_advance(&mut app);
    assert_eq!(
        app.selected_unit,
        Some(extra),
        "the focus lands on the next unit that can still take an order"
    );
}

/// Esc closes the contact window the same way confirming peace does: the
/// ordinary run loop resumes, and a spent focus still jumps on.
#[test]
fn escaping_the_contact_diplomacy_window_resumes_the_unit_auto_advance() {
    let Some((mut app, settler_id, direction, _)) = app_with_unknown_neighbor() else {
        return;
    };
    let home = {
        let engine = app.engine.as_ref().unwrap();
        engine
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == settler_id)
            .map(|unit| unit.location)
            .expect("the settler is still on the map")
    };
    let extra = app.engine.as_mut().unwrap().game.spawn_unit(
        UnitClass::Settler,
        home,
        PlayerId::new(0),
        None,
    );
    app.move_selected_unit(direction);
    assert!(app.diplomacy.is_some());
    app.handle_key(key(KeyCode::Esc));
    assert!(app.diplomacy.is_none(), "Esc dismisses the dialog");
    assert!(app.unit_advance_deadline.is_some());
    fire_pending_unit_advance(&mut app);
    assert_eq!(app.selected_unit, Some(extra));
}

/// A blocked move never spends the unit, so keeping peace must not hand the
/// focus on: the settler is still the one to command, and its flash resumes
/// on its own tile.
#[test]
fn peace_from_a_blocked_move_keeps_the_focus_on_the_unspent_unit() {
    let Some((mut app, settler_id, direction, _, _)) = app_with_adjacent_foreigner() else {
        return;
    };
    app.move_selected_unit(direction);
    assert!(app.diplomacy.is_some());
    app.handle_key(key(KeyCode::Enter)); // PEACE
    assert!(app.diplomacy.is_none());
    assert!(
        app.unit_advance_deadline.is_none(),
        "nothing to advance: the blocked unit still owns the focus"
    );
    let unit = app
        .engine
        .as_ref()
        .unwrap()
        .player_units()
        .into_iter()
        .find(|unit| unit.id() == settler_id)
        .unwrap();
    assert!(unit.moves_remaining() > 0, "the blocked step spent nothing");
    assert_eq!(app.selected_unit, Some(settler_id));
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
        Some(city_id),
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
        Some(city_id),
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
        Some(CityId::new(0)),
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
        Some(CityId::new(0)),
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
            battle: false,
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
            battle: false,
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
            battle: false,
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

/// The rival's attack reaches the player as a battle frame, so the replay
/// flashes the explosion over the tile the fight was fought on.
#[test]
fn a_rival_attack_replays_as_a_battle_frame() {
    let mut app = App::new();
    app.phase = Phase::Playing;
    let mut engine = Engine::new(
        5,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    for y in 0..3 {
        for x in 0..5 {
            engine
                .game
                .map
                .tile_at_mut(Location::new(x as u16, y as u16))
                .terrain = Terrain::Grassland;
        }
    }
    // The human keeps a city, so control wraps back to them and the round
    // ends with the human's discovery map in play.
    engine.game.cities.push(City::new(
        "London",
        Location::new(4, 2),
        PlayerId::new(0),
        CityId::new(0),
    ));
    engine.game.spawn_unit(
        UnitClass::Militia,
        Location::new(0, 0),
        PlayerId::new(0),
        Some(CityId::new(0)),
    );
    engine.game.spawn_unit(
        UnitClass::Legion,
        Location::new(1, 0),
        PlayerId::new(1),
        Some(CityId::new(1)),
    );
    // The human has spied the fight, so the attack is replayed rather than
    // staying in the fog.
    engine.game.players[0].reveal_tiles_at(Location::new(0, 0), 2);
    app.engine = Some(engine);

    app.end_turn(); // the rival's turn resolves

    let animation = app
        .rival_animation
        .as_ref()
        .expect("the rival's attack is replayed");
    let frame = animation
        .frames
        .iter()
        .find(|frame| frame.battle)
        .expect("the attack is replayed as a battle, so the tile it struck flashes");
    assert_eq!(
        frame.to,
        Location::new(0, 0),
        "the fight is fought on the tile the rival attacked"
    );
    assert!(
        app.war_notice.is_some(),
        "and the war is still announced by its own window"
    );
}

#[test]
fn a_city_that_starved_is_announced_by_its_own_window() {
    let mut app = App::new();
    at_start(&mut app);
    app.phase = Phase::Playing;
    let mut engine = Engine::new(
        5,
        3,
        Player::new(Civilization::English),
        vec![Player::new(Civilization::Zulu)],
    );
    // A mountain centre yields no food, so a size-two city starves each round
    // and loses a citizen.
    engine.game.map.tile_at_mut(Location::new(2, 1)).terrain = Terrain::Mountain;
    let mut london = City::new(
        "London",
        Location::new(2, 1),
        PlayerId::new(0),
        CityId::new(0),
    );
    london.grow();
    engine.game.cities.push(london);
    // A rival city keeps the round from ending on the human's turn alone.
    engine.game.cities.push(City::new(
        "Zimbabwe",
        Location::new(0, 0),
        PlayerId::new(1),
        CityId::new(1),
    ));
    app.engine = Some(engine);

    app.end_turn();

    assert!(
        app.starvation_notice.is_some(),
        "the lost citizen is announced by a window"
    );
    assert!(window_is_open(&app), "which counts as a window");
    assert_eq!(
        app.engine.as_ref().unwrap().player_cities()[0].population(),
        1,
        "and the citizen is actually lost"
    );

    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.starvation_notice.is_none(),
        "OK acknowledges the loss and closes the window"
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

/// A freshly started game (`s` on the start prompt), with every rival
/// eliminated and the human left standing.
fn finished_app() -> App {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let rival_count = app.engine.as_ref().unwrap().game.players.len() - 1;
    for index in 1..=rival_count {
        app.engine
            .as_mut()
            .unwrap()
            .eliminate_player(PlayerId::new(index));
    }
    app
}

#[test]
fn victory_overlay_rises_when_the_last_rival_falls() {
    let mut app = finished_app();
    app.check_game_over();
    let state = app.game_over.as_ref().unwrap();
    assert_eq!(state.outcome, GameOutcome::Victory);
    assert_eq!(state.civ_name, "American");
    assert!(state.first_selected, "the first button starts selected");
}

#[test]
fn defeat_overlay_rises_when_the_human_falls() {
    let mut app = finished_app();
    app.engine
        .as_mut()
        .unwrap()
        .eliminate_player(PlayerId::new(0));
    app.check_game_over();
    let state = app.game_over.as_ref().unwrap();
    assert_eq!(state.outcome, GameOutcome::Defeat);
    assert_eq!(state.civ_name, "American");
}

#[test]
fn end_turn_raises_the_victory_overlay_once_all_rivals_are_gone() {
    let mut app = finished_app();
    app.handle_key(key(KeyCode::Char(' '))); // end turn
    assert!(matches!(
        app.game_over.as_ref().map(|state| state.outcome),
        Some(GameOutcome::Victory)
    ));
}

#[test]
fn continue_playing_from_the_victory_overlay_returns_to_the_game() {
    let mut app = finished_app();
    app.check_game_over();
    app.handle_key(key(KeyCode::Enter)); // first button: Continue playing
    assert!(app.game_over.is_none());
    assert!(matches!(app.phase, Phase::Playing));
    assert!(!app.exit_requested);
}

#[test]
fn start_again_from_the_defeat_screen_returns_to_the_menu() {
    let mut app = finished_app();
    app.engine
        .as_mut()
        .unwrap()
        .eliminate_player(PlayerId::new(0));
    app.check_game_over();
    app.handle_key(key(KeyCode::Enter)); // first button: Start again
    assert!(app.game_over.is_none());
    assert!(matches!(app.phase, Phase::Menu));
}

#[test]
fn quitting_from_the_end_screen_ends_the_process() {
    let mut app = finished_app();
    app.check_game_over();
    app.handle_key(key(KeyCode::Right)); // second button: Quit
    app.handle_key(key(KeyCode::Enter));
    assert!(app.exit_requested);
}

#[test]
fn the_arrow_keys_move_the_game_over_cursor_between_the_buttons() {
    let mut app = finished_app();
    app.check_game_over();
    assert!(app.game_over.as_ref().unwrap().first_selected);
    app.handle_key(key(KeyCode::Right));
    assert!(!app.game_over.as_ref().unwrap().first_selected);
    app.handle_key(key(KeyCode::Right));
    assert!(!app.game_over.as_ref().unwrap().first_selected);
    app.handle_key(key(KeyCode::Left));
    assert!(app.game_over.as_ref().unwrap().first_selected);
    app.handle_key(key(KeyCode::Tab));
    assert!(!app.game_over.as_ref().unwrap().first_selected);
    app.handle_key(key(KeyCode::Tab));
    assert!(app.game_over.as_ref().unwrap().first_selected);
}

#[test]
fn clicking_a_game_over_button_activates_it() {
    let mut app = finished_app();
    app.check_game_over();
    let area = Rect::new(0, 0, 120, 40);
    let (first, second) = game_over_button_rects(area, GameOutcome::Victory);
    app.game_over_buttons.set(Some((first, second)));

    // A click on the Quit button ends the process.
    app.left_click(second.x + 1, second.y);
    assert!(app.exit_requested);
}

#[test]
fn clicks_outside_the_game_over_buttons_do_nothing() {
    let mut app = finished_app();
    app.check_game_over();
    app.left_click(4, 4);
    assert!(app.game_over.is_some());
    assert!(!app.exit_requested);
}

/// The app tests share one pinned world seed. Without it every test that
/// starts a game draws a fresh random map, and any assertion about the terrain
/// becomes a coin flip — `clicking_a_city_next_to_a_spent_unit_still_opens_
/// the_city_window` failed on roughly 1.5% of maps, the ones that happen to
/// ring the starting city with water and forest and leave no passable
/// neighbour to park its unit on.
#[test]
fn every_game_the_tests_start_shares_one_pinned_world_seed() {
    let (first, cx, cy) = playing_app();
    let (second, dx, dy) = playing_app();
    assert_eq!(
        (cx, cy),
        (dx, dy),
        "the starting city lands in the same place"
    );

    let terrain_of = |app: &App| {
        let engine = app.engine.as_ref().unwrap();
        (0..engine.height())
            .flat_map(|y| (0..engine.width()).map(move |x| (x, y)))
            .map(|(x, y)| engine.tile(x, y).terrain)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        terrain_of(&first),
        terrain_of(&second),
        "two games started in one run generate the same world"
    );
}

/// The pinned seed must still yield a world these tests can work with: a
/// starting city ringed by at least one passable tile to stand on.
#[test]
fn the_pinned_seed_leaves_the_starting_city_a_passable_neighbour() {
    let (app, cx, cy) = playing_app();
    let engine = app.engine.as_ref().unwrap();
    let passable = [
        (0isize, -1isize),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ]
    .into_iter()
    .filter(|(dx, dy)| {
        let nx = (cx as isize + dx).rem_euclid(engine.width() as isize) as usize;
        let ny = (cy as isize + dy).clamp(0, engine.height() as isize - 1) as usize;
        let tile = engine.tile(nx, ny);
        tile.terrain.is_land() && tile.terrain.movement_cost() <= 1
    })
    .count();
    assert!(
        passable > 0,
        "the pinned world rings its city with at least one passable tile"
    );
}

/// A fixture `BuildComplete` record, as the engine would hand over after a
/// round in which the named city finished building `target`.
fn build(city: CityId, city_name: &str, target: ProductionTarget) -> BuildComplete {
    BuildComplete {
        city,
        city_name: city_name.to_string(),
        target,
    }
}

/// An app with `count` player cities, each given a distinct name so the notice
/// queue can be checked by name rather than by id.
fn app_with_cities(count: usize) -> (App, Vec<CityId>) {
    let mut app = App::new();
    at_start(&mut app);
    app.handle_key(key(KeyCode::Char('s')));
    let names = ["London", "York", "Norwich", "Bristol"];
    let ids = {
        let engine = app.engine.as_mut().unwrap();
        let human = crate::model::civilizations::PlayerId::new(0);
        (0..count)
            .map(|index| {
                let location =
                    crate::model::cartography::Location::new(10 + (index as u16) * 4, 10);
                engine.game.add_city(human, names[index], location)
            })
            .collect()
    };
    (app, ids)
}

#[test]
fn a_completed_build_opens_a_window_naming_the_city_and_target() {
    let (mut app, cities) = app_with_cities(1);
    app.open_build_notices(vec![build(
        cities[0],
        "London",
        ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
    )]);

    let notice = app.current_build_notice().expect("a window is up");
    assert_eq!(notice.city, cities[0]);
    assert_eq!(notice.city_name, "London");

    // The window is modal: game keys do not leak through to the map.
    app.handle_key(key(KeyCode::Char('w')));
    assert!(
        !app.command_picker_open,
        "the map stays inert behind the window"
    );
}

#[test]
fn ok_on_a_build_window_moves_on_to_the_next_city() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    assert_eq!(app.current_build_notice().unwrap().city_name, "London");
    app.build_notice_confirm();
    assert_eq!(
        app.current_build_notice().unwrap().city_name,
        "York",
        "acknowledging the first window shows the next city"
    );
    app.build_notice_confirm();
    assert!(
        app.current_build_notice().is_none(),
        "the window closes once the queue empties"
    );
}

#[test]
fn ok_is_reachable_from_the_keyboard() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.current_build_notice().unwrap().city_name, "York");
    app.handle_key(key(KeyCode::Char(' ')));
    assert!(app.current_build_notice().is_none());
}

#[test]
fn next_order_opens_the_announced_city_window_and_parks_the_queue() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    app.handle_key(key(KeyCode::Char('n')));

    assert_eq!(
        app.selected_city,
        Some(cities[0]),
        "the announced city's own window opens"
    );
    assert!(
        app.current_build_notice().is_none(),
        "the parked window yields the screen to the city window"
    );
    assert!(
        app.build_notice.is_some(),
        "the remaining city is still queued"
    );
}

#[test]
fn closing_the_city_window_returns_to_the_loop_at_the_next_city() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);
    app.build_notice_next_order();
    assert_eq!(app.selected_city, Some(cities[0]));

    // Give the city a next order the way the player would, then close.
    app.handle_key(key(KeyCode::Esc));

    assert_eq!(app.selected_city, None, "the city window is closed");
    assert_eq!(
        app.current_build_notice()
            .map(|done| done.city_name.as_str()),
        Some("York"),
        "the loop carries on where it left off rather than being abandoned"
    );
}

#[test]
fn the_loop_survives_several_cities_each_taking_a_new_order() {
    let (mut app, cities) = app_with_cities(3);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[2],
            "Norwich",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    let mut visited = Vec::new();
    for expected in [&cities[0], &cities[1], &cities[2]] {
        assert_eq!(app.current_build_notice().map(|d| d.city), Some(*expected));
        visited.push(app.current_build_notice().unwrap().city_name.clone());
        app.build_notice_next_order();
        assert_eq!(app.selected_city, Some(*expected));
        app.handle_key(key(KeyCode::Esc));
    }

    assert_eq!(visited, vec!["London", "York", "Norwich"]);
    assert!(
        app.current_build_notice().is_none(),
        "the loop ends once every city has been answered"
    );
    assert!(app.build_notice.is_none(), "no stale queue is left behind");
}

#[test]
fn a_build_window_for_a_city_that_no_longer_exists_moves_on() {
    let (mut app, cities) = app_with_cities(2);
    let ghost = cities[0];
    // Capture the city out from under the queued notice.
    app.engine
        .as_mut()
        .unwrap()
        .game
        .cities
        .retain(|city| city.id() != ghost);
    app.open_build_notices(vec![
        build(
            ghost,
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    app.build_notice_next_order();

    assert_eq!(
        app.selected_city, None,
        "no window opens for a city that is gone"
    );
    assert_eq!(
        app.current_build_notice().map(|d| d.city_name.as_str()),
        Some("York"),
        "the notice moves on to the next city instead of stalling"
    );
}

#[test]
fn clicking_ok_and_next_order_drive_the_same_loop_as_the_keys() {
    let (mut app, cities) = app_with_cities(2);
    let area = Rect::new(0, 0, 120, 40);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);
    let panel = crate::tui::build_complete_dialog::dialog_rect(area);
    app.build_notice_rect.set(Some(panel));

    let next_order = crate::tui::build_complete_dialog::next_order_button_rect(panel);
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        next_order.x + 1,
        next_order.y,
    ));
    assert_eq!(app.selected_city, Some(cities[0]));

    app.handle_key(key(KeyCode::Esc));
    app.build_notice_rect.set(Some(panel));
    let ok = crate::tui::build_complete_dialog::ok_button_rect(panel);
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        ok.x + 1,
        ok.y,
    ));
    assert!(
        app.current_build_notice().is_none(),
        "OK clicked on the last city closes the window"
    );
}

/// The window is modal, so it must not be possible to wander off into the
/// menu or the map while completions are still unacknowledged — the queue
/// would then follow the player to a screen it does not belong on. Answering
/// the queue releases the keyboard again.
#[test]
fn the_build_window_is_modal_until_the_queue_is_answered() {
    let (mut app, cities) = app_with_cities(1);
    app.open_build_notices(vec![build(
        cities[0],
        "London",
        ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
    )]);

    app.handle_key(key(KeyCode::Char('q')));
    assert!(
        matches!(app.phase, Phase::Playing),
        "the game keys do not reach the quit path behind the window"
    );

    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.current_build_notice().is_none(),
        "the queue is answered"
    );
    app.handle_key(key(KeyCode::Char('q')));
    assert!(
        matches!(app.phase, Phase::Playing),
        "`q` now opens the quit dialog instead"
    );
    assert!(app.quit_dialog.is_some());
}

/// The window's state is not enough — it has to reach the screen. Rendering the
/// app with a queued completion must actually paint the city name, what it
/// finished, and both buttons, and remember its rectangle for hit-testing.
#[test]
fn a_completed_build_is_drawn_over_the_map_with_both_buttons() {
    let (mut app, cities) = app_with_cities(1);
    app.open_build_notices(vec![build(
        cities[0],
        "London",
        ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
    )]);

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    let buf = terminal.backend().buffer().clone();

    let panel = app
        .build_notice_rect
        .get()
        .expect("the drawn window is remembered for hit-testing");
    let screen = (0..panel.height)
        .map(|i| {
            (panel.x..panel.right())
                .map(|x| buf.cell((x, panel.y + i)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(screen.contains("London"), "the city is named: {screen}");
    assert!(screen.contains("Militia"), "the unit is named: {screen}");
    assert!(screen.contains("OK"), "OK is offered: {screen}");
    assert!(
        screen.contains("Next Order"),
        "Next Order is offered: {screen}"
    );
}

/// While the player is inside the city window opened by "Next Order", the notice
/// is parked: it must not paint over the city they are ordering.
#[test]
fn the_parked_build_window_does_not_paint_over_the_city_window() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    assert!(app.build_notice_rect.get().is_some(), "drawn while queued");

    app.build_notice_next_order();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();

    assert!(
        app.build_notice_rect.get().is_none(),
        "a parked notice is not on screen, so the city window owns it"
    );
    let buf = terminal.backend().buffer().clone();
    let panel = crate::tui::build_complete_dialog::dialog_rect(Rect::new(0, 0, 120, 40));
    let screen = (0..panel.height)
        .map(|i| {
            (panel.x..panel.right())
                .map(|x| buf.cell((x, panel.y + i)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        !screen.contains("Build complete"),
        "the window is gone from the screen: {screen}"
    );
    assert!(
        !screen.contains("York"),
        "the next city is not announced until the city window closes: {screen}"
    );
}

/// A city finishing a build and a tech completing can come out of the same
/// round. The research dialog captures the keyboard and draws first, so the
/// build queue must be parked behind it and picked up once the choice is made
/// — not discarded, which would lose every city the player never heard about.
#[test]
fn a_research_dialog_parks_the_build_queue_instead_of_dropping_it() {
    let (mut app, cities) = app_with_cities(2);
    app.open_build_notices(vec![
        build(
            cities[0],
            "London",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
        build(
            cities[1],
            "York",
            ProductionTarget::Unit(crate::model::units::UnitClass::Militia),
        ),
    ]);
    // The same thing the EndTurn path does when a round advances a technology.
    app.research_dialog = Some(ResearchDialogState {
        discovered: crate::model::advancements::Advancement::Wheel,
        choices: Vec::new(),
        cursor: 0,
        scroll: 0,
    });
    if let Some(notice) = &mut app.build_notice {
        notice.suspended = true;
    }

    // Parked: the queue survives, but nothing is on screen to answer.
    assert!(app.build_notice.is_some(), "the queue is not discarded");
    assert!(app.current_build_notice().is_none(), "but it is parked");

    app.research_dialog_confirm();

    assert!(
        app.research_dialog.is_none(),
        "the research choice is answered"
    );
    let shown = app.current_build_notice().expect("the loop resumes");
    assert_eq!(
        shown.city_name, "London",
        "at the first city that finished, not past it"
    );
    app.build_notice_confirm();
    assert_eq!(
        app.current_build_notice()
            .expect("the loop continues")
            .city_name,
        "York",
        "no city is skipped by the detour through research"
    );
}

/// The event window is the player's own log. Ending a turn runs every rival's
/// turn, which floods the engine's buffer with their housekeeping; none of it
/// may reach the log the player reads.
/// The event window is the player's own log. Ending a turn runs every rival's
/// turn, which floods the engine's buffer with their housekeeping — "Roman
/// begins turn", "Egyptian begin researching", "Unit 1 moves SW". None of that
/// may reach the log the player reads.
#[test]
fn the_event_log_holds_only_the_players_own_events() {
    let mut app = App::new();
    at_competition(&mut app);
    app.handle_key(key(KeyCode::Enter)); // default competition, on to difficulty
    app.handle_key(key(KeyCode::Enter)); // default difficulty, on to start prompt
    app.handle_key(key(KeyCode::Char('s')));
    for _ in 0..3 {
        app.handle_key(key(KeyCode::Enter)); // end turn
    }

    let human = app
        .engine
        .as_ref()
        .unwrap()
        .civilization_of(PlayerId::new(0))
        .display_name()
        .to_string();
    let log: Vec<String> = app
        .event_log
        .iter()
        .map(|e| e.message().to_string())
        .collect();
    assert!(
        log.iter().any(|m| m.contains(&human)),
        "the player still hears about their own turn: {log:?}"
    );
    // Every rival civilization in the game, by name, must be absent.
    // Every other civilization in the match, found through the units they own.
    let rival_names: Vec<String> = {
        let engine = app.engine.as_ref().unwrap();
        let mut names: Vec<String> = engine
            .game
            .players
            .iter()
            .filter(|p| p.civilization.display_name() != human)
            .map(|p| p.civilization.display_name().to_string())
            .collect();
        names.sort();
        names.dedup();
        names
    };
    assert!(
        !rival_names.is_empty(),
        "the fixture needs rivals to be meaningful"
    );
    for message in &log {
        for rival in &rival_names {
            assert!(
                !message.contains(rival.as_str()),
                "{rival:?} news leaked into the player's log: {message:?}"
            );
        }
    }
}

/// A playing app with the player's diplomat standing on open land, a rival city
/// on the next land tile east of him, and `gold` in the treasury. The two
/// civilizations have already met, so the step raises the diplomat window on its
/// own rather than behind the war-or-peace choice.
fn app_with_diplomat_beside_a_rival_city(gold: u32) -> (App, UnitId, Direction, Location) {
    let mut app = app_with_settler();
    let (diplomat_id, city_tile) = {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        let (width, height) = (engine.width(), engine.height());
        engine.game.players[0].set_gold(gold);
        engine.game.make_peace(PlayerId::new(0), PlayerId::new(1));
        let land = |location: Location| {
            let terrain = engine
                .tile(location.x as usize, location.y as usize)
                .terrain;
            terrain.is_land() && terrain.movement_cost() <= 1
        };
        let mut site = None;
        'search: for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                let here = Location::new(x as u16, y as u16);
                let east = Location::new(x as u16 + 1, y as u16);
                if land(here) && land(east) {
                    site = Some((here, east));
                    break 'search;
                }
            }
        }
        let (diplomat_tile, city_tile) = site.expect("the pinned world has open ground");
        engine
            .game
            .add_city(PlayerId::new(1), "Umgungundlovu", city_tile);
        let diplomat =
            engine
                .game
                .spawn_unit(UnitClass::Diplomat, diplomat_tile, PlayerId::new(0), None);
        (diplomat, city_tile)
    };
    (app, diplomat_id, Direction::E, city_tile)
}

/// Stand a rival garrison in the city (the one thing Incite a Revolt and
/// Subvert a City need; the diplomat acts alone) and a second unit of the
/// player's own beside him, so tests can pin that the subvert sweep leaves the
/// player's units standing.
fn garrison_the_city_with_both_sides(app: &mut App, city_tile: Location) {
    let engine = app.engine.as_mut().expect("a new game has an engine");
    engine.game.spawn_unit(
        UnitClass::Legion,
        city_tile,
        PlayerId::new(1),
        Some(CityId::new(0)),
    );
    engine
        .game
        .spawn_unit(UnitClass::Legion, city_tile, PlayerId::new(0), None);
}

/// The action rows the open window is offering, as `(label, cost)`.
fn offered_actions(app: &App) -> Vec<(String, u32)> {
    app.diplomat_actions
        .as_ref()
        .expect("the diplomat window is open")
        .options
        .iter()
        .map(|option| (option.action.label().to_string(), option.cost))
        .collect()
}

/// The treasury, as the window's confirmations will find it.
fn treasury(app: &App) -> u32 {
    app.engine
        .as_ref()
        .expect("a new game has an engine")
        .game
        .players[0]
        .gold()
}

/// Where `id` is standing, for tests that want to prove it did not move.
fn unit_location(app: &App, id: UnitId) -> Location {
    app.engine
        .as_ref()
        .and_then(|engine| engine.unit(id))
        .expect("the unit is still in the game")
        .location
}

/// The rival city, as the window's actions will find it.
fn rival_city(app: &App) -> crate::model::cities::City {
    app.engine
        .as_ref()
        .expect("a new game has an engine")
        .game
        .cities[0]
        .clone()
}

#[test]
fn walking_a_diplomat_into_a_rival_city_offers_him_its_five_actions() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    let state = app
        .diplomat_actions
        .as_ref()
        .expect("a diplomat who enters a rival city is given the window");
    assert_eq!(state.unit, diplomat_id);
    assert_eq!(state.city_name, "Umgungundlovu");
    assert_eq!(state.cursor, 0, "the window opens on the first action");
    assert_eq!(
        offered_actions(&app),
        vec![
            ("Investigate City".to_string(), 25),
            ("Steal Technology".to_string(), 50),
            ("Industrial Sabotage".to_string(), 50),
            ("Incite a Revolt".to_string(), 100),
            ("Subvert City".to_string(), 200),
        ],
        "the five things he came for, with what each costs"
    );
    let engine = app.engine.as_ref().unwrap();
    assert_eq!(
        engine.units_at(city_tile.x as usize, city_tile.y as usize)[0].id(),
        diplomat_id,
        "and he did get inside"
    );
}

#[test]
fn the_diplomat_window_is_modal_so_the_map_waits_beneath_it() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    let settler = app
        .selected_unit
        .expect("the starting settler is the focused unit");
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some());

    // Fortifying the settler is a perfectly ordinary move command, and the
    // window has nothing to say about it — so if the order goes through, the
    // map got the key.
    app.handle_key(key(KeyCode::Char('f')));
    assert_eq!(
        app.engine.as_ref().unwrap().unit(settler).unwrap().order(),
        UnitOrder::Idle,
        "the map never saw the key"
    );
    app.handle_key(key(KeyCode::Char('q')));
    assert!(
        matches!(app.phase, Phase::Playing),
        "and q did not reach the quit path either"
    );
    assert_eq!(
        unit_location(&app, diplomat_id),
        city_tile,
        "and the diplomat is still standing where the window found him"
    );
}

/// A diplomat dismissed at the door goes back to the tile he came from with his
/// moves restored, so he is unconsumed and can act again.
#[test]
fn a_dismissed_diplomat_can_still_walk_out_of_the_city() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    let outside = unit_location(&app, diplomat_id);
    app.move_unit(diplomat_id, direction);
    assert_eq!(unit_location(&app, diplomat_id), city_tile);
    assert!(app.diplomat_actions.is_some());

    app.handle_key(key(KeyCode::Esc));
    assert_eq!(
        unit_location(&app, diplomat_id),
        outside,
        "esc returns him to the tile he entered from"
    );
    // With moves restored he remains unconsumed and can move again.
    app.move_unit(diplomat_id, Direction::W);
    assert_ne!(
        unit_location(&app, diplomat_id),
        outside,
        "moves were restored, so he can step away again"
    );
    assert!(
        app.diplomat_actions.is_none(),
        "and moving out does not raise the window a second time"
    );
}

/// The garrison a diplomat buys is the player's unit the moment the window is
/// confirmed: Tab cycles to it, and while it still stands on the rival's square
/// the only thing it may do is walk off — no fortify or sentry is taken there.
#[test]
fn a_revolted_garrison_joins_the_tab_cycle_and_must_leave_first() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    garrison_the_city_with_both_sides(&mut app, city_tile);
    let garrison = {
        let engine = app.engine.as_ref().expect("a new game has an engine");
        engine
            .game
            .units
            .iter()
            .find(|unit| unit.owner() == PlayerId::new(1) && unit.unit_class == UnitClass::Legion)
            .expect("the rival Legion guards the city")
            .id()
    };
    // A real garrison is a fortify order with its turn spent, which is what
    // the purchase clears: make it look like one before the diplomat arrives.
    {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        let idx = engine
            .game
            .units
            .iter()
            .position(|unit| unit.id() == garrison)
            .expect("the garrison exists");
        let unit = &mut engine.game.units[idx];
        unit.fortify();
        unit.spend_turn();
    }
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some(), "the window opens");
    // The cursor starts on Investigate City; Incite a Revolt is the fourth row.
    for _ in 0..3 {
        app.handle_key(key(KeyCode::Down));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.diplomat_actions.is_none(),
        "the action closes the window"
    );
    assert!(
        app.engine
            .as_ref()
            .unwrap()
            .game
            .at_war(PlayerId::new(0), PlayerId::new(1)),
        "a coup is a declaration of war"
    );

    // The bought soldier is in the player's ordinary cycle: Tab reaches it
    // just like any own unit with moves to spare.
    let mut seen = Vec::new();
    for _ in 0..app.engine.as_ref().unwrap().player_units().len() {
        app.handle_key(key(KeyCode::Tab));
        seen.push(app.selected_unit);
    }
    assert!(
        seen.contains(&Some(garrison)),
        "Tab cycles to the bought garrison: {seen:?}"
    );
    assert_eq!(
        app.engine.as_ref().unwrap().unit(garrison).unwrap().order(),
        UnitOrder::Idle,
        "and the rival's fortify order was cancelled"
    );

    // On the rival's own square the only command is the walk-out: fortify is
    // refused and nothing is applied.
    app.selected_unit = Some(garrison);
    app.handle_key(key(KeyCode::Char('f')));
    assert_eq!(
        app.engine.as_ref().unwrap().unit(garrison).unwrap().order(),
        UnitOrder::Idle,
        "no fortify is taken on the rival square"
    );
    assert!(
        app.event_log
            .iter()
            .any(|event| event.message().contains("must leave the city first")),
        "the refusal reaches the player's log"
    );

    // A step west clears the city tile, and the order the square refused is
    // then taken freely on the unit's own ground.
    app.handle_key(key(KeyCode::Char('h')));
    assert_ne!(
        unit_location(&app, garrison),
        city_tile,
        "the first act is the walk-out"
    );
    app.handle_key(key(KeyCode::Char('f')));
    assert_eq!(
        app.engine.as_ref().unwrap().unit(garrison).unwrap().order(),
        UnitOrder::Fortified,
        "off the rival square the order is taken"
    );
}

/// The window is a window: it counts as one for the map's idle flash, and it
/// holds the pointer's move hint off, so no click can land on a tile the player
/// is not looking at while he is still choosing an action.
#[test]
fn the_diplomat_window_counts_as_a_window_and_holds_the_hover_off() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.camera.set((0, 0));
    let settler = app
        .selected_unit
        .expect("the starting settler is the focused unit");
    let (sx, sy) = {
        let location = unit_location(&app, settler);
        (location.x as usize, location.y as usize)
    };
    app.handle_mouse(mouse_event(
        MouseEventKind::Moved,
        LEFT_COLUMN_WIDTH + (sx + 1) as u16 * TILE_WIDTH as u16,
        sy as u16,
    ));
    assert!(
        app.hovered_move_target(app.engine.as_ref().unwrap())
            .is_some(),
        "the pointer hints at a move in plain play"
    );

    app.move_unit(diplomat_id, direction);
    draw(&app);
    assert!(
        window_is_open(&app),
        "the diplomat window counts as a window"
    );
    // The move armed the follow-camera and the draw recentred it, so put it back
    // where this test put it: the pointer and the settler have both stayed put,
    // and so `hover_blocked` is the only thing that can suppress the hint.
    app.camera.set((0, 0));
    assert!(
        app.hovered_move_target(app.engine.as_ref().unwrap())
            .is_none(),
        "and it holds the move hint off"
    );

    app.handle_key(key(KeyCode::Esc));
    draw(&app);
    assert!(!window_is_open(&app), "closing it re-arms the flash");
    // The move set the follow-camera, and the draw recentred it on the focused
    // unit, so put the camera back where this test put it: the pointer has not
    // moved, and neither has the settler.
    app.camera.set((0, 0));
    assert!(
        app.hovered_move_target(app.engine.as_ref().unwrap())
            .is_some(),
        "and the pointer hints at a move again"
    );
}

#[test]
fn confirming_an_action_spends_the_diplomat_and_closes_the_window() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    app.handle_key(key(KeyCode::Enter));
    assert!(app.diplomat_actions.is_none(), "the window closes");
    let engine = app.engine.as_ref().unwrap();
    assert!(
        engine
            .player_units()
            .iter()
            .all(|unit| unit.id() != diplomat_id),
        "his mission was the action: the diplomat is spent"
    );
    assert_eq!(engine.game.players[0].gold(), 475, "and it cost 25 gold");
    assert!(
        rival_city(&app).investigated(),
        "what he learned outlives him"
    );
    assert_eq!(
        app.selected_city,
        Some(rival_city(&app).id()),
        "investigating opens the city's window there and then"
    );
    draw(&app);
}

/// Steal Technology closes the diplomat window and announces its prize: which
/// advance came away and from which rival, in a window of its own whose OK
/// button dismisses it.
#[test]
fn stealing_a_technology_announces_the_prize_in_its_own_window() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    let rival_name = {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        // The player's Alphabet and Masonry make the rival's Mathematics
        // researchable, so the theft has a takeable prize to come away with.
        engine.game.players[PlayerId::new(0).index()].add_advancement(Advancement::Alphabet);
        engine.game.players[PlayerId::new(0).index()].add_advancement(Advancement::Masonry);
        engine.game.players[PlayerId::new(1).index()].add_advancement(Advancement::Mathematics);
        engine.game.players[PlayerId::new(1).index()]
            .civilization
            .display_name()
            .to_string()
    };
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some(), "the window opens");
    // The cursor starts on Investigate City; Steal Technology is the second row.
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.diplomat_actions.is_none(),
        "confirming the theft closes the diplomat window"
    );
    let engine = app.engine.as_ref().expect("a new game has an engine");
    assert!(
        engine
            .player_units()
            .iter()
            .all(|unit| unit.id() != diplomat_id),
        "the theft spends its diplomat"
    );
    assert!(
        engine.game.players[PlayerId::new(0).index()].has_advancement(Advancement::Mathematics),
        "and the advance comes away"
    );
    let notice = app
        .steal_notice
        .as_ref()
        .expect("the theft is announced in a window of its own");
    assert_eq!(
        notice,
        &StealOutcome::Stolen {
            advancement: Advancement::Mathematics,
            rival: rival_name,
        },
        "it names the prize and its former owner"
    );
    assert!(window_is_open(&app), "the announcement counts as a window");

    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.steal_notice, None, "OK acknowledges the prize");
    assert!(!window_is_open(&app), "and the map is free again");
}

/// A Steal Technology attempt against a rival that knows nothing the player
/// could take reports itself in a window of its own and then hands the
/// diplomat back the way he came in: nothing was bought, so nothing is spent —
/// not the gold, not the man.
#[test]
fn an_empty_theft_reports_itself_and_withdraws_the_diplomat() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        // The player already knows everything the rival does, so there is no
        // takeable advance left in the cup — the refusal is all that remains.
        let rival_advances = engine.game.players[PlayerId::new(1).index()]
            .advances_made()
            .to_vec();
        for advancement in rival_advances {
            engine.game.players[PlayerId::new(0).index()].add_advancement(advancement);
        }
    }
    let entrant_tile = Location::new(city_tile.x - 1, city_tile.y);
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some(), "the action window opens");
    // The cursor starts on Investigate City; Steal Technology is the second row.
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.diplomat_actions.is_none(),
        "confirming the theft closes the action window"
    );
    let engine = app.engine.as_ref().expect("a new game has an engine");
    let diplomat = engine
        .unit(diplomat_id)
        .expect("an empty theft spends no diplomat");
    assert_eq!(
        diplomat.location, entrant_tile,
        "and he steps back onto the tile he walked in from"
    );
    assert_eq!(
        engine.game.players[PlayerId::new(0).index()].gold(),
        500,
        "and no gold leaves the treasury"
    );
    let notice = app
        .steal_notice
        .as_ref()
        .expect("the empty theft reports itself in a window of its own");
    assert!(
        matches!(notice, StealOutcome::NothingToSteal { .. }),
        "the window is the refusal, not a prize"
    );
    assert!(window_is_open(&app), "the report counts as a window");
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.steal_notice, None, "OK acknowledges the report");
    assert!(!window_is_open(&app), "and the map is free again");
}

/// Industrial Sabotage closes the diplomat window and reports the damage in a
/// window of its own: which improvement came down and in which city, dismissed
/// by its OK button.
#[test]
fn sabotaging_reports_what_was_destroyed_in_its_own_window() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        engine
            .game
            .cities
            .iter_mut()
            .find(|city| city.location == city_tile && city.owner() == PlayerId::new(1))
            .expect("the rival city exists")
            .add_improvement(CityImprovement::CityWalls);
    }
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some(), "the window opens");
    // The cursor starts on Investigate City; Industrial Sabotage is the third row.
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.diplomat_actions.is_none(),
        "confirming the sabotage closes the diplomat window"
    );
    let engine = app.engine.as_ref().expect("a new game has an engine");
    assert!(
        engine
            .player_units()
            .iter()
            .all(|unit| unit.id() != diplomat_id),
        "the sabotage spends its diplomat"
    );
    assert!(
        !engine
            .game
            .cities
            .iter()
            .find(|city| city.location == city_tile && city.owner() == PlayerId::new(1))
            .expect("the rival city survives")
            .improvements()
            .contains(&CityImprovement::CityWalls),
        "and the walls come down"
    );
    assert_eq!(
        app.sabotage_notice,
        Some(SabotageNotice {
            improvement: CityImprovement::CityWalls,
            city_name: "Umgungundlovu".to_string(),
        }),
        "the window names what was destroyed and where"
    );
    assert!(window_is_open(&app), "the report counts as a window");

    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.sabotage_notice, None, "OK acknowledges the damage");
    assert!(!window_is_open(&app), "and the map is free again");
}

/// The report is a single showing: a plain map click cannot open an
/// investigated foreign city's window, because a foreign city stays
/// unselectable and the investigation flow is the only way one gets in.
#[test]
fn an_investigated_foreign_city_window_cannot_be_reopened() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(
        app.selected_city,
        Some(rival_city(&app).id()),
        "the investigation opened the intel window"
    );
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(
        app.selected_city, None,
        "dismissing it clears the selection"
    );

    app.map_pane
        .set(Some(Rect::new(LEFT_COLUMN_WIDTH, 0, 200, 40)));
    app.left_click(
        (LEFT_COLUMN_WIDTH as usize + city_tile.x as usize * TILE_WIDTH) as u16,
        city_tile.y,
    );
    assert_eq!(
        app.selected_city, None,
        "a foreign city stays unselectable: the window is not coming back"
    );
}

/// A city can be investigated again and again: `investigated` records what was
/// learned, it never bars the purchase, and a second diplomat walking into the
/// same city buys the same look — the report opens again, at the same price.
#[test]
fn a_second_diplomat_can_investigate_the_same_city() {
    let (mut app, first, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(first, direction);
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(
        app.selected_city,
        Some(rival_city(&app).id()),
        "the first look opens the report window"
    );
    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.selected_city, None, "dismissing the report clears it");
    let second = {
        let engine = app.engine.as_mut().expect("a new game has an engine");
        engine.game.spawn_unit(
            UnitClass::Diplomat,
            Location::new(city_tile.x - 1, city_tile.y),
            PlayerId::new(0),
            None,
        )
    };
    app.move_unit(second, direction);
    assert!(
        app.diplomat_actions.is_some(),
        "the second man walking into the same city is offered his five actions"
    );
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(
        app.selected_city,
        Some(rival_city(&app).id()),
        "and the second purchase opens the report window again"
    );
    assert_eq!(
        treasury(&app),
        450,
        "each look is bought at the same 25 gold"
    );
    assert!(
        rival_city(&app).investigated(),
        "and what the first man learned still stands"
    );
}

/// The flag `handle_key` hands back is the run loop's instruction to quit the
/// process, and every test in this file throws it away. So the one that has to
/// read it is this: pressing Return to confirm a row used to end the game as
/// happily as confirming it, and nothing else in the suite would have noticed.
#[test]
fn a_key_in_the_diplomat_window_leaves_the_game_running() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    assert!(app.diplomat_actions.is_some());

    // A cursor step is ordinary input, and must not end the run either.
    assert!(
        !app.handle_key(key(KeyCode::Down)),
        "moving the cursor must not end the game"
    );
    assert!(app.diplomat_actions.is_some(), "and the window stays up");
    // Back to the first row, which is the one this fixture can afford.
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.diplomat_actions.as_ref().unwrap().cursor, 0);

    assert!(
        !app.handle_key(key(KeyCode::Enter)),
        "confirming with Return must not end the game"
    );
    assert!(app.diplomat_actions.is_none(), "the action went through");
    assert_eq!(
        app.engine.as_ref().unwrap().game.players[0].gold(),
        475,
        "and it cost the 25 gold of the row on the cursor"
    );
    assert!(matches!(app.phase, Phase::Playing), "and play continues");
}

#[test]
fn ok_does_nothing_at_all_for_a_row_the_engine_refuses() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(0);
    app.move_unit(diplomat_id, direction);
    assert!(
        app.diplomat_actions.as_ref().unwrap().options[0]
            .blocked
            .is_some(),
        "with an empty treasury the first action is out of reach"
    );
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.diplomat_actions.is_some(),
        "confirming it neither acts nor closes the window"
    );
    assert_eq!(
        app.engine.as_ref().unwrap().game.players[0].gold(),
        0,
        "and nothing was spent"
    );
    assert!(
        app.engine
            .as_ref()
            .unwrap()
            .player_units()
            .iter()
            .any(|unit| unit.id() == diplomat_id),
        "so the player is left holding an unspent diplomat, which is the point"
    );
}

#[test]
fn esc_closes_the_window_without_spending_the_diplomat() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    app.handle_key(key(KeyCode::Esc));
    assert!(app.diplomat_actions.is_none());
    assert_eq!(treasury(&app), 500, "nothing was spent");
    assert!(
        app.engine
            .as_ref()
            .unwrap()
            .player_units()
            .iter()
            .any(|unit| unit.id() == diplomat_id),
        "and the diplomat is still standing in the city"
    );
}

#[test]
fn the_cursor_wraps_over_the_five_actions() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    let cursor = |app: &App| app.diplomat_actions.as_ref().unwrap().cursor;
    for _ in 0..DiplomatAction::ALL.len() {
        app.handle_key(key(KeyCode::Down));
    }
    assert_eq!(cursor(&app), 0, "down past the last action wraps round");
    for _ in 0..DiplomatAction::ALL.len() {
        app.handle_key(key(KeyCode::Up));
    }
    assert_eq!(cursor(&app), 0, "and so does up past the first");
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    assert_eq!(cursor(&app), 2);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(cursor(&app), 1);
}

#[test]
fn a_click_picks_the_row_and_a_second_click_on_ok_acts_on_it() {
    let (mut app, diplomat_id, direction, city_tile) = app_with_diplomat_beside_a_rival_city(500);
    garrison_the_city_with_both_sides(&mut app, city_tile);
    app.move_unit(diplomat_id, direction);
    draw(&app);
    let panel = app
        .diplomat_actions_rect
        .get()
        .expect("the drawn window records its rectangle");

    // A click on the Subvert row moves the cursor onto it and does nothing else.
    let subvert = app
        .diplomat_actions
        .as_ref()
        .unwrap()
        .options
        .iter()
        .position(|option| option.action == DiplomatAction::SubvertCity)
        .unwrap();
    let row = diplomat_actions_dialog::row_rect(panel, subvert).expect("the row was drawn");
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        row.x + 1,
        row.y,
    ));
    assert_eq!(
        app.diplomat_actions.as_ref().unwrap().cursor,
        subvert,
        "the click selected the row it landed on"
    );
    assert_eq!(
        rival_city(&app).owner(),
        PlayerId::new(1),
        "and acted on nothing"
    );

    // The OK button is what performs it, and the window closes as it does.
    let ok = diplomat_actions_dialog::ok_button_rect(panel);
    app.handle_mouse(mouse_event(
        MouseEventKind::Down(MouseButton::Left),
        ok.x + 1,
        ok.y,
    ));
    assert!(app.diplomat_actions.is_none(), "the window closes");
    assert_eq!(treasury(&app), 300, "for 200 gold");
    // A city of population one falls outright rather than changing hands, so
    // the tile is bare — and the garrison the subversion was aimed at went with
    // it, while the player's own second unit did not.
    assert!(
        app.engine.as_ref().unwrap().game.cities.is_empty(),
        "the subverted city is gone"
    );
    let engine = app.engine.as_ref().unwrap();
    assert!(
        engine
            .game
            .units
            .iter()
            .all(|unit| unit.owner() != PlayerId::new(1)),
        "and so is the garrison that held it"
    );
    assert!(
        engine
            .game
            .units
            .iter()
            .any(|unit| unit.owner() == PlayerId::new(0) && unit.unit_class == UnitClass::Legion),
        "but the player's own unit standing beside him is untouched"
    );
}

#[test]
fn the_open_diplomat_window_draws_over_the_map() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| App::draw(frame, &app)).unwrap();
    let panel = app
        .diplomat_actions_rect
        .get()
        .expect("the drawn window records its rectangle");
    let buffer = terminal.backend().buffer();

    let row = diplomat_actions_dialog::row_rect(panel, 0).expect("the first row is drawn");
    let text: String = (0..row.width - 1)
        .map(|i| buffer.cell((row.x + 1 + i, row.y)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("Investigate City"),
        "the window names what it is offering, drew {text:?}"
    );
}

/// The bug the player hit: the diplomat window's rectangle is computed on every
/// frame it is up, and a terminal dragged down to a column or two used to
/// overflow that subtraction and take the game down mid-decision. Every size a
/// player can produce by resizing has to draw, and be clickable, without a
/// panic.
#[test]
fn the_diplomat_window_survives_a_terminal_squeezed_to_nothing() {
    let (mut app, diplomat_id, direction, _) = app_with_diplomat_beside_a_rival_city(500);
    app.move_unit(diplomat_id, direction);
    for (width, height) in [(120u16, 40u16), (80, 24), (20, 8), (10, 5), (4, 3), (1, 1)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| App::draw(frame, &app)).unwrap();
        // The click guard works from the rect the draw just recorded, so the
        // press below is matched against the degenerate panel.
        for (column, row) in [
            (0u16, 0u16),
            (0, height - 1),
            (width - 1, 0),
            (width / 2, height / 2),
        ] {
            app.handle_mouse(mouse_event(
                MouseEventKind::Down(MouseButton::Left),
                column,
                row,
            ));
            app.handle_mouse(mouse_event(
                MouseEventKind::Up(MouseButton::Left),
                column,
                row,
            ));
        }
    }
    // Esc still closes it, so a squeezed terminal cannot trap the player.
    app.handle_key(key(KeyCode::Esc));
    assert!(app.diplomat_actions.is_none());
    draw(&app);
}
