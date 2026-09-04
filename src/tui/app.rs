use std::cell::Cell;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::civ_selector::CivSelector;
use super::competition_selector::CompetitionSelector;
use super::difficulty_selector::DifficultySelector;
use super::game_screen::GameScreen;
use super::playing_help::PlayingHelp;
use super::splash::SplashScreen;
use super::start_confirm::StartConfirm;
use super::status_bar::{ITEMS, StatusBar};
use crate::game_engine::event::Event as GameEvent;
use crate::game_engine::{Command, Engine, GameView, Player};
use crate::model::cartography::Direction;
use crate::model::civilizations::Civilization;
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use crate::model::units::{UnitClass, UnitId};
use strum::IntoEnumIterator;

pub const STATUS_DELAY: Duration = Duration::from_secs(2);
pub const STATUS_FADE: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
/// The number of most-recent event messages the log keeps in view.
pub const EVENT_LOG_SIZE: usize = 5;

#[derive(PartialEq)]
enum Phase {
    Menu,
    ChoosingCiv,
    ChoosingCompetition,
    ChoosingDifficulty,
    ReadyToStart,
    Playing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartChoice {
    Start,
    Quit,
}

impl StartChoice {
    fn next(self) -> Self {
        match self {
            StartChoice::Start => StartChoice::Quit,
            StartChoice::Quit => StartChoice::Start,
        }
    }

    fn index(self) -> usize {
        match self {
            StartChoice::Start => 0,
            StartChoice::Quit => 1,
        }
    }
}

pub struct App {
    started_at: Instant,
    selected: usize,
    phase: Phase,
    civ_index: usize,
    chosen_civ: Option<Civilization>,
    competition_index: usize,
    chosen_competition: Option<Competition>,
    difficulty_index: usize,
    chosen_difficulty: Option<Difficulty>,
    start_choice: StartChoice,
    engine: Option<Engine>,
    selected_unit: Option<UnitId>,
    camera: Cell<(usize, usize)>,
    show_help: bool,
    show_events: bool,
    event_log: Vec<GameEvent>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            selected: 0,
            phase: Phase::Menu,
            civ_index: 0,
            chosen_civ: None,
            competition_index: 0,
            chosen_competition: None,
            difficulty_index: 0,
            chosen_difficulty: None,
            start_choice: StartChoice::Start,
            engine: None,
            selected_unit: None,
            camera: Cell::new((0, 0)),
            show_help: false,
            show_events: false,
            event_log: Vec::new(),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| Self::draw(frame, self))?;
            if event::poll(POLL_INTERVAL)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key(key)
            {
                return Ok(());
            }
        }
    }

    fn draw(frame: &mut Frame, app: &App) {
        match app.phase {
            Phase::Menu => {
                frame.render_widget(SplashScreen::new(), frame.area());
                let progress = fade_progress(app.started_at.elapsed());
                if progress > 0.0 {
                    let area = Rect {
                        x: frame.area().x,
                        y: frame.area().bottom().saturating_sub(1),
                        width: frame.area().width,
                        height: 1,
                    };
                    frame.render_widget(StatusBar::new(progress, app.selected), area);
                }
            }
            Phase::ChoosingCiv => frame.render_widget(
                CivSelector::new(app.civ_index, app.chosen_civ),
                frame.area(),
            ),
            Phase::ChoosingCompetition => frame.render_widget(
                CompetitionSelector::new(app.competition_index, app.chosen_competition),
                frame.area(),
            ),
            Phase::ChoosingDifficulty => frame.render_widget(
                DifficultySelector::new(app.difficulty_index, app.chosen_difficulty),
                frame.area(),
            ),
            Phase::ReadyToStart => frame.render_widget(
                StartConfirm::new(
                    app.chosen_civ.unwrap(),
                    app.chosen_competition.unwrap(),
                    app.chosen_difficulty.unwrap(),
                    app.start_choice.index(),
                ),
                frame.area(),
            ),
            Phase::Playing => {
                if let Some(engine) = &app.engine {
                    let area = frame.area();
                    let focus = focus_coordinate(engine, app.selected_unit);
                    let map_pane_width = area
                        .width
                        .saturating_sub(super::game_screen::LEFT_COLUMN_WIDTH);
                    let pane_cols = (map_pane_width as usize) / 2;
                    let camera = camera_for(
                        focus,
                        (engine.width(), engine.height()),
                        (pane_cols, area.height as usize),
                        app.camera.get(),
                    );
                    app.camera.set(camera);
                    frame.render_widget(
                        GameScreen::new(
                            engine,
                            focus,
                            camera,
                            app.selected_unit,
                            app.started_at.elapsed(),
                            app.show_events,
                            &app.event_log[app.event_log.len().saturating_sub(EVENT_LOG_SIZE)..],
                        ),
                        area,
                    );
                    if app.show_help {
                        let bar = Rect {
                            x: area.x,
                            y: area.bottom().saturating_sub(2),
                            width: area.width,
                            height: 2,
                        };
                        frame.render_widget(
                            PlayingHelp::new(&playing_commands(
                                app.selected_unit.is_some(),
                                selected_can_found(app, engine),
                            )),
                            bar,
                        );
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.phase {
            Phase::Menu => self.handle_menu_key(key),
            Phase::ChoosingCiv => self.handle_civ_key(key),
            Phase::ChoosingCompetition => self.handle_competition_key(key),
            Phase::ChoosingDifficulty => self.handle_difficulty_key(key),
            Phase::ReadyToStart => self.handle_start_key(key),
            Phase::Playing => self.handle_playing_key(key),
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'q' => true,
                'n' => {
                    self.start_new_game();
                    false
                }
                'l' => {
                    self.selected = 1;
                    false
                }
                _ => false,
            },
            KeyCode::Right => {
                self.selected = advance(self.selected, ITEMS.len());
                false
            }
            KeyCode::Left => {
                self.selected = retreat(self.selected, ITEMS.len());
                false
            }
            KeyCode::Enter => match self.selected {
                0 => {
                    self.start_new_game();
                    false
                }
                2 => true,
                _ => false,
            },
            KeyCode::Esc => true,
            _ => false,
        }
    }

    fn handle_civ_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.civ_index = retreat(self.civ_index, Civilization::iter().count());
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.civ_index = advance(self.civ_index, Civilization::iter().count());
                false
            }
            KeyCode::Enter => {
                self.chosen_civ = Civilization::iter().nth(self.civ_index);
                self.phase = Phase::ChoosingCompetition;
                self.competition_index = 0;
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::Menu;
                false
            }
            _ => false,
        }
    }

    fn handle_competition_key(&mut self, key: KeyEvent) -> bool {
        let levels = (Competition::MAX - Competition::MIN + 1) as usize;
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.competition_index = retreat(self.competition_index, levels);
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.competition_index = advance(self.competition_index, levels);
                false
            }
            KeyCode::Enter => {
                self.chosen_competition = Some(Competition::new(
                    self.competition_index as u8 + Competition::MIN,
                ));
                self.phase = Phase::ChoosingDifficulty;
                self.difficulty_index = 0;
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::ChoosingCiv;
                false
            }
            _ => false,
        }
    }

    fn handle_difficulty_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.difficulty_index = retreat(self.difficulty_index, Difficulty::iter().count());
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.difficulty_index = advance(self.difficulty_index, Difficulty::iter().count());
                false
            }
            KeyCode::Enter => {
                self.chosen_difficulty = Difficulty::iter().nth(self.difficulty_index);
                self.phase = Phase::ReadyToStart;
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::ChoosingCompetition;
                false
            }
            _ => false,
        }
    }

    fn handle_start_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                's' => {
                    self.start_game();
                    false
                }
                'q' => true,
                'k' => {
                    self.start_choice = self.start_choice.next();
                    false
                }
                'j' => {
                    self.start_choice = self.start_choice.next();
                    false
                }
                _ => false,
            },
            KeyCode::Up => {
                self.start_choice = self.start_choice.next();
                false
            }
            KeyCode::Down => {
                self.start_choice = self.start_choice.next();
                false
            }
            KeyCode::Enter => match self.start_choice {
                StartChoice::Start => {
                    self.start_game();
                    false
                }
                StartChoice::Quit => true,
            },
            KeyCode::Esc => {
                self.phase = Phase::ChoosingDifficulty;
                false
            }
            _ => false,
        }
    }

    fn start_game(&mut self) {
        let rival_count = self
            .chosen_competition
            .unwrap_or(Competition::new(Competition::MIN))
            .rivals() as usize;
        let chosen = self.chosen_civ.unwrap();
        let mut pool: Vec<Civilization> =
            Civilization::iter().filter(|civ| *civ != chosen).collect();
        // Seed the rival draw (and the map, inside `Engine::new_random`) from
        // the system clock so each new game randomizes both.
        let mut rng = crate::utils::Rng::new(crate::utils::random_seed());
        let mut rivals = Vec::with_capacity(rival_count);
        while rivals.len() < rival_count.min(pool.len()) {
            let idx = rng.in_range(pool.len() as u32) as usize;
            rivals.push(pool.swap_remove(idx));
        }
        let mut engine = Engine::new_random(
            crate::game_engine::DEFAULT_MAP_WIDTH,
            crate::game_engine::DEFAULT_MAP_HEIGHT,
            Player::new(chosen),
            rivals.into_iter().map(Player::new).collect(),
        );
        engine.populate_starting_world();
        self.engine = Some(engine);
        self.select_first_unit();
        self.phase = Phase::Playing;
        self.reset_setup();
        self.event_log.clear();
    }

    fn start_new_game(&mut self) {
        self.reset_setup();
        self.phase = Phase::ChoosingCiv;
    }

    fn handle_playing_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
                false
            }
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                } else {
                    self.phase = Phase::Menu;
                }
                false
            }
            KeyCode::Tab => {
                self.cycle_unit_selection();
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selected_unit(Direction::N);
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selected_unit(Direction::S);
                false
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_selected_unit(Direction::W);
                false
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_selected_unit(Direction::E);
                false
            }
            KeyCode::Char('y') => {
                self.move_selected_unit(Direction::NW);
                false
            }
            KeyCode::Char('u') => {
                self.move_selected_unit(Direction::NE);
                false
            }
            KeyCode::Char('b') => {
                self.move_selected_unit(Direction::SW);
                false
            }
            KeyCode::Char('n') => {
                self.move_selected_unit(Direction::SE);
                false
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.end_turn();
                false
            }
            KeyCode::Char('v') => {
                self.found_selected_city();
                false
            }
            KeyCode::Char('e') => {
                self.show_events = !self.show_events;
                false
            }
            _ => false,
        }
    }

    fn select_first_unit(&mut self) {
        if let Some(engine) = &self.engine {
            self.selected_unit = engine.player_units().first().map(|unit| unit.id());
        } else {
            self.selected_unit = None;
        }
    }

    fn cycle_unit_selection(&mut self) {
        let Some(engine) = &self.engine else {
            return;
        };
        let units = engine.player_units();
        if units.is_empty() {
            self.selected_unit = None;
            return;
        }
        let next = match self.selected_unit {
            Some(current) => {
                let index = units
                    .iter()
                    .position(|unit| unit.id() == current)
                    .unwrap_or(0);
                (index + 1) % units.len()
            }
            None => 0,
        };
        self.selected_unit = Some(units[next].id());
    }

    fn move_selected_unit(&mut self, direction: Direction) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::Move { unit, direction });
            self.record_events(events);
        }
    }

    fn found_selected_city(&mut self) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let name = city_name_for(engine, unit);
            let events = engine.submit(Command::FoundCity { unit, name });
            self.record_events(events);
        }
    }

    fn end_turn(&mut self) {
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::EndTurn);
            self.record_events(events);
        }
        self.select_first_unit();
    }

    fn record_events(&mut self, events: Vec<GameEvent>) {
        if events.is_empty() {
            return;
        }
        // Keep only the most recent few messages so the log view stays small.
        self.event_log.extend(events);
        let overflow = self.event_log.len().saturating_sub(EVENT_LOG_SIZE);
        if overflow > 0 {
            self.event_log.drain(..overflow);
        }
    }

    fn reset_setup(&mut self) {
        self.civ_index = 0;
        self.chosen_civ = None;
        self.competition_index = 0;
        self.chosen_competition = None;
        self.difficulty_index = 0;
        self.chosen_difficulty = None;
        self.start_choice = StartChoice::Start;
    }
}

fn advance(selected: usize, total: usize) -> usize {
    (selected + 1) % total
}

fn retreat(selected: usize, total: usize) -> usize {
    (selected + total - 1) % total
}

/// The name for the next city founded by the given settler's civilization:
/// the civilization's display name.
fn city_name_for(engine: &Engine, unit: UnitId) -> String {
    let unit = engine.player_units().into_iter().find(|u| u.id() == unit);
    let Some(unit) = unit else {
        return "City".to_string();
    };
    engine
        .civilization_of(unit.owner())
        .capital_name()
        .to_string()
}

/// The command keystrokes available in the current playing context.
fn playing_commands(selected: bool, can_found: bool) -> Vec<(&'static str, &'static str)> {
    let mut commands: Vec<(&'static str, &'static str)> = Vec::new();
    if selected {
        commands.push(("arrows", "move"));
        commands.push(("y/u/b/n", "diag"));
        commands.push(("f", "fortify"));
        commands.push(("s", "sentry"));
        commands.push(("w", "work"));
        commands.push(("c", "cancel"));
        if can_found {
            commands.push(("v", "found"));
        }
    }
    commands.push(("tab", "next unit"));
    commands.push(("space", "end turn"));
    commands.push(("e", "events"));
    commands.push(("?", "help"));
    commands.push(("q", "quit"));
    commands
}

/// Whether the selected unit is a settler that can found a city.
fn selected_can_found(app: &App, engine: &Engine) -> bool {
    let Some(unit) = app.selected_unit else {
        return false;
    };
    engine
        .player_units()
        .into_iter()
        .find(|u| u.id() == unit)
        .is_some_and(|u| u.unit_class == UnitClass::Settler)
}

/// Resolve the selected unit to its current map coordinate, for rendering the
/// focus panel.
fn focus_coordinate(engine: &Engine, selected: Option<UnitId>) -> Option<(usize, usize)> {
    let unit = selected.and_then(|id| {
        engine
            .player_units()
            .into_iter()
            .find(|unit| unit.id() == id)
    })?;
    Some((unit.location.x as usize, unit.location.y as usize))
}

/// The world-tile coordinate for the top-left of the map pane. Centres on
/// `focus` when there is one, clamped so the camera never shows tiles beyond a
/// map edge. `pane` is the pane size measured in world tiles (cols, rows).
///
/// The map wraps horizontally (east/west) but not vertically. Camera
/// positioning uses the shortest wrap-around path when deciding whether the
/// focus is within the middle 70 % of the view.
fn camera_for(
    focus: Option<(usize, usize)>,
    map: (usize, usize),
    pane: (usize, usize),
    camera: (usize, usize),
) -> (usize, usize) {
    let (map_w, map_h) = map;
    let (pane_cols, pane_rows) = pane;
    let (cx, cy) = match focus {
        Some((x, y)) if pane_cols > 0 && pane_rows > 0 => {
            let margin_x = pane_cols * 15 / 100;
            let margin_y = pane_rows * 15 / 100;

            // Compute the viewport column of the focus tile using wrap-around
            // so a unit just east/west of the camera edge is treated as close.
            let view_col = (x + map_w - camera.0) % map_w;
            let view_row = y as isize - camera.1 as isize;

            let in_x = view_col < pane_cols
                && view_col >= margin_x
                && view_col < pane_cols.saturating_sub(margin_x);
            let in_y = view_row >= margin_y as isize
                && view_row < (pane_rows as isize - margin_y as isize);

            if in_x && in_y {
                return camera;
            }

            // Re-centre on the focus. The x coordinate wraps around using
            // Euclidean modulo so the camera can be positioned for the
            // shortest path.
            let new_cx =
                (x as isize - (pane_cols / 2) as isize).rem_euclid(map_w as isize) as usize;
            let new_cy = (y as isize - (pane_rows / 2) as isize).max(0);
            let new_cy = map_h.saturating_sub(pane_rows).min(new_cy as usize);
            (new_cx, new_cy)
        }
        _ => camera,
    };
    (cx, cy)
}

fn fade_progress(elapsed: Duration) -> f32 {
    let after_delay = elapsed.saturating_sub(STATUS_DELAY);
    (after_delay.as_secs_f32() / STATUS_FADE.as_secs_f32()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
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
            if ny == before.y as isize + tile_dy
                && terrain.is_land()
                && terrain.movement_cost() <= 1
            {
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
}
