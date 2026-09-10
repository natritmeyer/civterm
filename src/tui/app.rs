use std::cell::Cell;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::city_window::{self, CityWindow};
use super::civ_selector::CivSelector;
use super::competition_selector::CompetitionSelector;
use super::difficulty_selector::DifficultySelector;
use super::game_screen::{GameScreen, LEFT_COLUMN_WIDTH, TILE_WIDTH};
use super::playing_help::PlayingHelp;
use super::production_picker::{self, PickRow, ProductionPicker};
use super::research_dialog::{self, ResearchDialog};
use super::splash::SplashScreen;
use super::start_confirm::StartConfirm;
use super::status_bar::{ITEMS, StatusBar};
use super::work_picker::{self, WorkPicker};
use crate::game_engine::event::Event as GameEvent;
use crate::game_engine::{Command, Engine, GameView, Player};
use crate::model::advancements::Advancement;
use crate::model::cartography::Direction;
use crate::model::cities::{CityId, ProductionTarget};
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use crate::model::geography::TerrainImprovement;
use crate::model::units::{UnitClass, UnitId};
use strum::IntoEnumIterator;

pub const STATUS_DELAY: Duration = Duration::from_secs(2);
pub const STATUS_FADE: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
/// The number of most-recent event messages the log keeps in view.
pub const EVENT_LOG_SIZE: usize = 5;
/// How many screen cells a left-button press may travel before it becomes a
/// map drag rather than a click.
const CLICK_SLOP: i32 = 2;

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

/// The research-completion dialog's live state: the advancement just
/// discovered, the researchable advancements on offer, and the cursor/scroll
/// within that list.
struct ResearchDialogState {
    discovered: Advancement,
    choices: Vec<Advancement>,
    cursor: usize,
    scroll: usize,
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
    selected_city: Option<CityId>,
    /// Scroll offset of the open city window's improvement list.
    city_window_scroll: usize,
    /// The last-drawn city-window rectangle and its close button, for mouse
    /// hit-testing while the window is open.
    moused_window: Cell<Option<(Rect, Rect)>>,
    /// Whether the production picker floats over the city window.
    production_picker_open: bool,
    /// The picker cursor: which list (0 units, 1 improvements) and which row
    /// of that list is currently selected.
    picker_cursor_col: usize,
    picker_cursor_row: usize,
    /// Shared vertical scroll offset of the picker's two lists.
    picker_scroll: usize,
    /// The last-drawn picker panel rectangle, for mouse hit-testing.
    picker_rect: Cell<Option<Rect>>,
    /// The most recent mouse position, so hover can steer the map highlight.
    mouse_position: Cell<Option<(u16, u16)>>,
    camera: Cell<(usize, usize)>,
    /// Where the left button was pressed over the map pane, while a
    /// click-or-drag gesture is in progress.
    drag_origin: Cell<Option<(u16, u16)>>,
    /// The pointer position of the previous drag move, for per-move deltas.
    drag_last: Cell<Option<(u16, u16)>>,
    /// Leftover screen-column deltas while dragging (two columns per world
    /// tile) carried into the next move.
    drag_carry: Cell<(i32, i32)>,
    /// True once a press has travelled past `CLICK_SLOP` and is a map drag.
    drag_engaged: Cell<bool>,
    /// False once the player has dragged the map by hand, so the camera stops
    /// auto-centring on the selected unit until the player interacts again.
    camera_follow: Cell<bool>,
    /// The last-drawn map pane rectangle, for drag hit-testing and clamping.
    map_pane: Cell<Option<Rect>>,
    show_help: bool,
    show_events: bool,
    event_log: Vec<GameEvent>,
    /// The open advancement-completion dialog, if one is showing.
    research_dialog: Option<ResearchDialogState>,
    /// The last-drawn research-dialog rectangle, for mouse hit-testing.
    research_dialog_rect: Cell<Option<Rect>>,
    /// Whether the work picker floats over the map.
    work_picker_open: bool,
    /// The picker cursor: which improvement row is selected.
    work_picker_cursor: usize,
    /// Vertical scroll offset of the work picker's list.
    work_picker_scroll: usize,
    /// The last-drawn work picker panel rectangle, for mouse hit-testing.
    work_picker_rect: Cell<Option<Rect>>,
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
            selected_city: None,
            city_window_scroll: 0,
            moused_window: Cell::new(None),
            production_picker_open: false,
            picker_cursor_col: 0,
            picker_cursor_row: 0,
            picker_scroll: 0,
            picker_rect: Cell::new(None),
            mouse_position: Cell::new(None),
            camera: Cell::new((0, 0)),
            drag_origin: Cell::new(None),
            drag_last: Cell::new(None),
            drag_carry: Cell::new((0, 0)),
            drag_engaged: Cell::new(false),
            camera_follow: Cell::new(true),
            map_pane: Cell::new(None),
            show_help: false,
            show_events: false,
            event_log: Vec::new(),
            research_dialog: None,
            research_dialog_rect: Cell::new(None),
            work_picker_open: false,
            work_picker_cursor: 0,
            work_picker_scroll: 0,
            work_picker_rect: Cell::new(None),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| Self::draw(frame, self))?;
            if !event::poll(POLL_INTERVAL)? {
                continue;
            }
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press && self.handle_key(key) => {
                    return Ok(());
                }
                Event::Mouse(mouse) => self.handle_mouse(mouse),
                _ => {}
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
                    let map_pane_width = area.width.saturating_sub(LEFT_COLUMN_WIDTH);
                    let pane_cols = (map_pane_width as usize) / 2;
                    // While the player has dragged the map by hand, the camera
                    // stays where they left it; otherwise it follows the
                    // selected unit.
                    let camera = if app.camera_follow.get() {
                        camera_for(
                            focus,
                            (engine.width(), engine.height()),
                            (pane_cols, area.height as usize),
                            app.camera.get(),
                        )
                    } else {
                        app.camera.get()
                    };
                    app.camera.set(camera);
                    let map_pane = Rect {
                        x: LEFT_COLUMN_WIDTH,
                        y: area.y,
                        width: map_pane_width,
                        height: area.height,
                    };
                    app.map_pane.set(Some(map_pane));
                    let hover_target = app.hovered_move_target(engine);
                    frame.render_widget(
                        GameScreen::new(
                            engine,
                            focus,
                            camera,
                            app.selected_unit,
                            app.selected_city,
                            app.started_at.elapsed(),
                            app.show_events,
                            &app.event_log[app.event_log.len().saturating_sub(EVENT_LOG_SIZE)..],
                            hover_target,
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
                    // The city window floats above the whole screen, centred.
                    match app.selected_city {
                        Some(city_id)
                            if engine
                                .city(city_id)
                                .is_some_and(|city| city.owner() == engine.current_player_id()) =>
                        {
                            let window = city_window::window_rect(area);
                            frame.render_widget(
                                CityWindow::new(engine, city_id, app.city_window_scroll),
                                window,
                            );
                            app.moused_window
                                .set(Some((window, city_window::close_button_rect(window))));
                            // The production picker floats over the window.
                            if app.production_picker_open {
                                let panel = production_picker::picker_rect(window);
                                frame.render_widget(
                                    ProductionPicker::new(
                                        engine,
                                        city_id,
                                        app.picker_cursor_col,
                                        app.picker_cursor_row,
                                        app.picker_scroll,
                                    ),
                                    panel,
                                );
                                app.picker_rect.set(Some(panel));
                            } else {
                                app.picker_rect.set(None);
                            }
                        }
                        _ => {
                            app.moused_window.set(None);
                            app.picker_rect.set(None);
                        }
                    }
                    // The research dialog floats above everything: at the start
                    // of a turn an advancement may have completed, and the
                    // player must choose the next research target.
                    if let Some(state) = &app.research_dialog {
                        let rect = research_dialog::dialog_rect(area);
                        frame.render_widget(
                            ResearchDialog::new(
                                engine,
                                state.discovered,
                                state.choices.clone(),
                                state.cursor,
                                state.scroll,
                            ),
                            rect,
                        );
                        app.research_dialog_rect.set(Some(rect));
                    } else {
                        app.research_dialog_rect.set(None);
                    }
                    // The work picker floats over the map when the player is
                    // about to give a settler a terrain-improvement order.
                    if app.work_picker_open {
                        if let Some(unit) = app.selected_unit {
                            let panel = work_picker::work_picker_rect(area);
                            frame.render_widget(
                                WorkPicker::new(
                                    engine,
                                    unit,
                                    app.work_picker_cursor,
                                    app.work_picker_scroll,
                                ),
                                panel,
                            );
                            app.work_picker_rect.set(Some(panel));
                        } else {
                            app.work_picker_rect.set(None);
                        }
                    } else {
                        app.work_picker_rect.set(None);
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
        self.camera.set((0, 0));
        self.camera_follow.set(true);
        self.drag_origin.set(None);
        self.drag_last.set(None);
        self.drag_carry.set((0, 0));
        self.drag_engaged.set(false);
        self.phase = Phase::Playing;
        self.reset_setup();
        self.event_log.clear();
        self.selected_city = None;
        self.city_window_scroll = 0;
        self.moused_window = Cell::new(None);
        self.production_picker_open = false;
        self.picker_cursor_col = 0;
        self.picker_cursor_row = 0;
        self.picker_scroll = 0;
        self.picker_rect = Cell::new(None);
        self.research_dialog = None;
        self.research_dialog_rect = Cell::new(None);
        self.work_picker_open = false;
        self.work_picker_cursor = 0;
        self.work_picker_scroll = 0;
        self.work_picker_rect = Cell::new(None);
    }

    fn start_new_game(&mut self) {
        self.reset_setup();
        self.phase = Phase::ChoosingCiv;
    }

    fn handle_playing_key(&mut self, key: KeyEvent) -> bool {
        // While the research dialog is open it captures the keyboard: the
        // player may only pick a research target and confirm it.
        if self.research_dialog.is_some() {
            self.handle_research_dialog_key(key);
            return false;
        }
        // While the work picker is open it captures the keyboard: the player
        // may only pick an improvement and confirm it.
        if self.work_picker_open {
            match key.code {
                KeyCode::Esc => self.close_work_picker(),
                KeyCode::Enter => self.save_work_picker(),
                KeyCode::Down | KeyCode::Char('j') => self.move_work_cursor(1),
                KeyCode::Up | KeyCode::Char('k') => self.move_work_cursor(-1),
                _ => {}
            }
            return false;
        }
        // While the production picker is open it captures the keyboard.
        if self.production_picker_open {
            match key.code {
                KeyCode::Esc => self.close_production_picker(),
                KeyCode::Enter => self.save_production(),
                KeyCode::Down | KeyCode::Char('j') => self.move_picker_cursor(0, 1),
                KeyCode::Up | KeyCode::Char('k') => self.move_picker_cursor(0, -1),
                KeyCode::Right | KeyCode::Char('l') => self.move_picker_cursor(1, 0),
                KeyCode::Left | KeyCode::Char('h') => self.move_picker_cursor(-1, 0),
                _ => {}
            }
            return false;
        }
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
                if self.selected_city.is_some() {
                    self.selected_city = None;
                    self.city_window_scroll = 0;
                } else if self.show_help {
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
            KeyCode::Char('w') => {
                self.open_work_picker();
                false
            }
            KeyCode::Char('c') => {
                self.cancel_selected_unit_order();
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
        self.camera_follow.set(true);
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
        self.camera_follow.set(true);
    }

    /// Routes mouse input during play. Left presses over the map start a
    /// click-or-drag gesture: the click is deferred until the button is
    /// released without the pointer having travelled beyond `CLICK_SLOP`, so
    /// dragging the map never accidentally moves a unit or opens a city.
    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.phase != Phase::Playing {
            return;
        }
        if self.engine.is_none() {
            return;
        }
        // Remember the pointer position on every mouse event (clicks and
        // drags too) so hover can steer the map highlight.
        if mouse.column != u16::MAX && mouse.row != u16::MAX {
            self.mouse_position.set(Some((mouse.column, mouse.row)));
        }
        // The research dialog floats above everything and captures all mouse
        // input while it is open.
        if self.research_dialog_rect.get().is_some() {
            self.handle_research_dialog_mouse(mouse);
            return;
        }
        // The work picker floats over the map and captures all mouse input
        // while it is open.
        if let Some(panel) = self.work_picker_rect.get() {
            self.handle_work_picker_mouse(panel, mouse);
            return;
        }
        // The production picker floats above the city window and captures all
        // mouse input while it is open.
        if let Some(panel) = self.picker_rect.get() {
            self.handle_picker_mouse(panel, mouse);
            return;
        }
        // Wheel events scroll the open city window's improvement list.
        if self.moused_window.get().is_some() {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.city_window_scroll = self.city_window_scroll.saturating_sub(1);
                    return;
                }
                MouseEventKind::ScrollDown => {
                    self.city_window_scroll = self.city_window_scroll.saturating_add(1);
                    return;
                }
                _ => {}
            }
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.mouse_pressed(),
            MouseEventKind::Drag(MouseButton::Left) => self.mouse_dragged(),
            MouseEventKind::Up(MouseButton::Left) => self.mouse_released(),
            _ => {}
        }
    }

    /// A left press over the map pane begins a click-or-drag gesture. Presses
    /// over the city window or its buttons act immediately, and presses over
    /// the left column just clear the city selection.
    fn mouse_pressed(&mut self) {
        let Some((column, row)) = self.mouse_position.get() else {
            return;
        };
        if column == u16::MAX || row == u16::MAX {
            return;
        }
        // While the window is open, the "Change" button opens the production
        // picker, the close button dismisses the window, and presses anywhere
        // else inside it are consumed rather than reaching the map.
        if let Some((window, close)) = self.moused_window.get() {
            let change =
                city_window::change_button_rect(city_window::production_panel_rect(window));
            if change.contains((column, row).into()) {
                self.open_production_picker();
                return;
            }
            if close.contains((column, row).into()) {
                self.selected_city = None;
                self.city_window_scroll = 0;
                return;
            }
            if window.contains((column, row).into()) {
                return;
            }
        }
        if column < LEFT_COLUMN_WIDTH {
            self.selected_city = None;
            return;
        }
        // A press on the map pane starts a potential drag.
        let pane = self.map_pane.get();
        if !pane.is_some_and(|pane| pane.contains((column, row).into())) {
            return;
        }
        self.drag_origin.set(Some((column, row)));
        self.drag_last.set(Some((column, row)));
        self.drag_carry.set((0, 0));
        self.drag_engaged.set(false);
    }

    /// Follows a held left button over the map pane, panning the camera so the
    /// map moves with the hand. The first `CLICK_SLOP` cells of travel are
    /// consumed as click jitter; beyond that the gesture is a drag and the
    /// camera stops auto-following the selected unit.
    fn mouse_dragged(&mut self) {
        let Some((column, row)) = self.mouse_position.get() else {
            return;
        };
        let Some((origin_col, origin_row)) = self.drag_origin.get() else {
            return;
        };
        let pane = self.map_pane.get();
        if !pane.is_some_and(|pane| pane.contains((column, row).into())) {
            return;
        }
        if !self.drag_engaged.get() {
            let travelled = (column as i32 - origin_col as i32)
                .abs()
                .max((row as i32 - origin_row as i32).abs());
            if travelled < CLICK_SLOP {
                return;
            }
            self.drag_engaged.set(true);
            self.camera_follow.set(false);
        }
        let (last_col, last_row) = self.drag_last.get().unwrap_or((origin_col, origin_row));
        let dcol = column as i32 - last_col as i32;
        let drow = row as i32 - last_row as i32;
        self.drag_last.set(Some((column, row)));
        if dcol == 0 && drow == 0 {
            return;
        }
        let (map_w, map_h) = {
            let engine = self.engine.as_ref().expect("engine exists in play");
            (engine.width(), engine.height())
        };
        // The guard above keeps the cursor over the pane, so unwrap is safe.
        let pane = pane.expect("pane checked above");
        // Horizontal deltas accumulate over two screen columns per world tile,
        // carrying the leftover fraction into the next drag move. Rows map 1:1.
        let mut carry = self.drag_carry.get();
        carry.0 += dcol;
        let shift_x = carry.0.div_euclid(TILE_WIDTH as i32);
        carry.0 -= shift_x * TILE_WIDTH as i32;
        let (camera_x, camera_y) = self.camera.get();
        let new_x = (camera_x as i32 - shift_x).rem_euclid(map_w as i32) as usize;
        let pane_rows = pane.height as usize;
        let max_y = map_h.saturating_sub(pane_rows);
        let new_y = (camera_y as i32 - drow).clamp(0, max_y as i32) as usize;
        self.camera.set((new_x, new_y));
        self.drag_carry.set(carry);
    }

    /// Ends a left press. Releases that stayed within `CLICK_SLOP` of the
    /// press are clicks and act on the map underneath the press point;
    /// anything longer is a completed drag, which leaves the camera where it
    /// was pushed.
    fn mouse_released(&mut self) {
        let origin = self.drag_origin.get();
        self.drag_origin.set(None);
        self.drag_last.set(None);
        self.drag_carry.set((0, 0));
        let engaged = self.drag_engaged.get();
        self.drag_engaged.set(false);
        if let Some((column, row)) = origin.filter(|_| !engaged) {
            self.map_click(column, row);
        }
    }

    /// A click on the map pane: a tile one square away moves the selected
    /// unit, otherwise the click selects or clears the city selection.
    fn map_click(&mut self, column: u16, row: u16) {
        let Some(engine) = &self.engine else {
            return;
        };
        let tile_col = (column - LEFT_COLUMN_WIDTH) as usize / TILE_WIDTH;
        let (camera_x, camera_y) = self.camera.get();
        let world_x = (camera_x + tile_col) % engine.width();
        let world_y = camera_y + row as usize;
        // A click on the tile one square away in any direction moves the
        // selected unit exactly as the arrow keys would, letting the engine
        // enforce the movement rules.
        let direction = focus_coordinate(engine, self.selected_unit)
            .and_then(|from| adjacent_direction(from, (world_x, world_y), engine.width()));
        if let Some(direction) = direction {
            self.move_selected_unit(direction);
            return;
        }
        let map_h = engine.height();
        let clicked = if world_y < map_h {
            engine
                .city_at(world_x, world_y)
                .filter(|city| city.owner() == engine.current_player_id())
                .map(|city| city.id())
        } else {
            None
        };
        self.selected_city = clicked;
    }

    fn move_selected_unit(&mut self, direction: Direction) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::Move { unit, direction });
            self.record_events(events);
        }
        self.camera_follow.set(true);
    }

    /// The world tile the pointer currently hovers, when it lies one square
    /// away from the selected unit (so a click would move there); `None`
    /// otherwise, including while a modal panel floats over the map.
    fn hovered_move_target(&self, engine: &Engine) -> Option<(usize, usize)> {
        if self.research_dialog_rect.get().is_some()
            || self.work_picker_rect.get().is_some()
            || self.picker_rect.get().is_some()
            || self.moused_window.get().is_some()
            || self.drag_origin.get().is_some()
        {
            return None;
        }
        let screen = self.mouse_position.get()?;
        let camera = self.camera.get();
        let map = (engine.width(), engine.height());
        hovered_adjacent_tile(
            screen,
            camera,
            map,
            focus_coordinate(engine, self.selected_unit),
        )
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
        let human = PlayerId::new(0);
        let (events, discovered, wrapped) = if let Some(engine) = &mut self.engine {
            let advances_before = engine.player_advances(human);
            let events = engine.submit(Command::EndTurn);
            // An advancement completes at the start of the human's turn: once
            // play wraps back to player zero, their research has advanced and
            // a discovery leaves them with no research in progress. Offer the
            // research-completion dialog so they can pick the next target.
            let discovered = engine
                .player_advances(human)
                .get(advances_before.len())
                .copied();
            let wrapped = engine.current_player_id() == human;
            (events, discovered, wrapped)
        } else {
            (Vec::new(), None, false)
        };
        self.record_events(events);
        if wrapped && let Some(discovered) = discovered {
            let choices = self
                .engine
                .as_ref()
                .map(|engine| engine.researchable_advancements_for(human))
                .unwrap_or_default();
            self.research_dialog = Some(ResearchDialogState {
                discovered,
                choices,
                cursor: 0,
                scroll: 0,
            });
            // The modal floats over the map; drop any stale overlays so
            // the player isn't asked about both at once.
            self.selected_city = None;
            self.city_window_scroll = 0;
            self.moused_window = Cell::new(None);
            self.production_picker_open = false;
            self.picker_rect = Cell::new(None);
            self.work_picker_open = false;
            self.work_picker_scroll = 0;
            self.work_picker_rect = Cell::new(None);
        }
        self.select_first_unit();
    }

    fn handle_research_dialog_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_research_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_research_cursor(1),
            KeyCode::Enter | KeyCode::Char(' ') => self.research_dialog_confirm(),
            _ => {}
        }
    }

    fn handle_research_dialog_mouse(&mut self, mouse: MouseEvent) {
        let Some(dialog) = &self.research_dialog else {
            return;
        };
        let Some(panel) = self.research_dialog_rect.get() else {
            return;
        };
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.column == u16::MAX || mouse.row == u16::MAX {
                    return;
                }
                let position = (mouse.column, mouse.row).into();
                if research_dialog::ok_button_rect(panel).contains(position) {
                    self.research_dialog_confirm();
                    return;
                }
                let rows = research_dialog::list_rect(panel);
                if rows.contains(position) {
                    let row_in_view = (mouse.row as usize).saturating_sub(rows.y as usize);
                    let choice = row_in_view + dialog.scroll;
                    if choice < dialog.choices.len() {
                        self.move_research_cursor_to(choice);
                    }
                }
            }
            MouseEventKind::ScrollUp => self.move_research_scroll(-1),
            MouseEventKind::ScrollDown => self.move_research_scroll(1),
            _ => {}
        }
    }

    fn research_dialog_confirm(&mut self) {
        let Some(dialog) = self.research_dialog.take() else {
            return;
        };
        self.research_dialog_rect.set(None);
        if let Some(advancement) = dialog.choices.get(dialog.cursor).copied()
            && let Some(engine) = &mut self.engine
        {
            let events = engine.submit(Command::SetResearchTarget { advancement });
            self.record_events(events);
        }
    }

    /// Move the research dialog's cursor by `delta` rows.
    fn move_research_cursor(&mut self, delta: isize) {
        let visible = self.research_visible_rows();
        let Some(dialog) = &mut self.research_dialog else {
            return;
        };
        if dialog.choices.is_empty() {
            return;
        }
        let len = dialog.choices.len();
        dialog.cursor = if delta > 0 {
            advance(dialog.cursor, len)
        } else {
            retreat(dialog.cursor, len)
        };
        Self::clamp_research_scroll(dialog, visible);
    }

    /// Jump the research dialog's cursor to a specific row.
    fn move_research_cursor_to(&mut self, choice: usize) {
        let visible = self.research_visible_rows();
        let Some(dialog) = &mut self.research_dialog else {
            return;
        };
        if choice < dialog.choices.len() {
            dialog.cursor = choice;
        }
        Self::clamp_research_scroll(dialog, visible);
    }

    /// Scroll the research dialog's list by `delta` rows (if it overflows).
    fn move_research_scroll(&mut self, delta: isize) {
        let visible = self.research_visible_rows();
        let Some(dialog) = &mut self.research_dialog else {
            return;
        };
        let base = dialog.scroll as isize + delta;
        let max_offset = dialog.choices.len().saturating_sub(visible);
        dialog.scroll = base.clamp(0, max_offset as isize) as usize;
    }

    /// Keep the scroll so the cursor stays inside the scrolled window.
    fn clamp_research_scroll(dialog: &mut ResearchDialogState, visible: usize) {
        let max_offset = dialog.choices.len().saturating_sub(visible);
        let lo = (dialog.cursor as isize + 1 - visible as isize).max(0) as usize;
        let hi = dialog.cursor.min(max_offset);
        dialog.scroll = dialog.scroll.clamp(lo, hi);
    }

    /// The number of research-dialog list rows visible on screen right now.
    fn research_visible_rows(&self) -> usize {
        self.research_dialog_rect
            .get()
            .map(|panel| research_dialog::list_rect(panel).height as usize)
            .unwrap_or(8)
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

    /// The buildable targets the current player can pick, split into units
    /// and improvements (sorted for display by the picker module).
    fn picker_rows(&self) -> (Vec<PickRow>, Vec<PickRow>) {
        let engine = self
            .engine
            .as_ref()
            .expect("engine exists in the playing phase");
        let city = self
            .selected_city
            .expect("the picker only opens over a selected city");
        production_picker::pick_rows(engine, city)
    }

    /// The number of picker rows visible on screen right now.
    fn picker_visible_rows(&self) -> usize {
        self.picker_rect
            .get()
            .map(|panel| production_picker::rows_rect(panel).height as usize)
            .unwrap_or(production_picker::MAX_VISIBLE_ROWS as usize)
    }

    fn open_production_picker(&mut self) {
        self.production_picker_open = true;
        self.picker_cursor_col = 0;
        self.picker_cursor_row = 0;
        self.picker_scroll = 0;
    }

    fn close_production_picker(&mut self) {
        self.production_picker_open = false;
        self.picker_scroll = 0;
        self.picker_rect.set(None);
    }

    fn save_production(&mut self) {
        let target = self.current_picker_target();
        let city = self.selected_city;
        self.close_production_picker();
        let Some(city) = city else {
            return;
        };
        let Some(target) = target else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::SetProductionTarget { city, target });
            self.record_events(events);
        }
    }

    /// The item the picker cursor currently points at, if any.
    fn current_picker_target(&self) -> Option<ProductionTarget> {
        let rows = self.picker_rows();
        production_picker::target_at(&rows, self.picker_cursor_col, self.picker_cursor_row)
    }

    fn handle_picker_mouse(&mut self, panel: Rect, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.picker_scroll = self.picker_scroll.saturating_sub(1);
                return;
            }
            MouseEventKind::ScrollDown => {
                let rows = self.picker_rows();
                let max_offset =
                    production_picker::list_len(&rows).saturating_sub(self.picker_visible_rows());
                self.picker_scroll = (self.picker_scroll + 1).min(max_offset);
                return;
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return,
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        if !panel.contains(position) {
            // Clicks outside the picker are swallowed while it is open.
            return;
        }
        if production_picker::cancel_button_rect(panel).contains(position) {
            self.close_production_picker();
            return;
        }
        if production_picker::save_button_rect(panel).contains(position) {
            self.save_production();
            return;
        }
        let rows = self.picker_rows();
        let (units_col, improvements_col) = production_picker::column_rects(panel);
        if units_col.contains(position) || improvements_col.contains(position) {
            let column = if units_col.contains(position) { 0 } else { 1 };
            let row_in_view = (mouse.row as usize).saturating_sub(units_col.y as usize);
            let global_row = row_in_view + self.picker_scroll;
            if production_picker::target_at(&rows, column, global_row).is_some() {
                self.picker_cursor_col = column;
                self.picker_cursor_row = global_row;
            }
        }
    }

    /// Move the picker cursor: `dx` switches between the two lists, `dy`
    /// walks the focused list. The scroll keeps the cursor visible.
    fn move_picker_cursor(&mut self, dx: isize, dy: isize) {
        let (units_len, improvements_len) = {
            let rows = self.picker_rows();
            (rows.0.len(), rows.1.len())
        };
        if dx != 0 {
            let column = match (self.picker_cursor_col, dx) {
                (0, 1) if improvements_len > 0 => 1,
                (1, -1) if units_len > 0 => 0,
                _ => self.picker_cursor_col,
            };
            if column != self.picker_cursor_col {
                self.picker_cursor_col = column;
            }
        } else {
            let len = if self.picker_cursor_col == 0 {
                units_len
            } else {
                improvements_len
            };
            if len == 0 {
                return;
            }
            self.picker_cursor_row = if dy > 0 {
                advance(self.picker_cursor_row, len)
            } else {
                retreat(self.picker_cursor_row, len)
            };
        }
        let row = self.picker_cursor_row;
        let len = if self.picker_cursor_col == 0 {
            units_len
        } else {
            improvements_len
        };
        self.picker_cursor_row = row.min(len.saturating_sub(1));
        // Keep the cursor inside the scrolled window.
        let visible = self.picker_visible_rows();
        let max_len = units_len.max(improvements_len);
        let max_offset = max_len.saturating_sub(visible);
        let row = self.picker_cursor_row;
        let lo = (row as isize + 1 - visible as isize).max(0) as usize;
        let hi = row.min(max_offset);
        self.picker_scroll = self.picker_scroll.clamp(lo, hi);
    }

    /// The improvements the selected settler can build on its tile right now.
    fn work_rows(&self) -> Vec<TerrainImprovement> {
        let Some(unit) = self.selected_unit else {
            return Vec::new();
        };
        let Some(engine) = &self.engine else {
            return Vec::new();
        };
        engine
            .player_units()
            .into_iter()
            .find(|u| u.id() == unit)
            .map(|u| engine.tile(u.location.x as usize, u.location.y as usize))
            .map(work_picker::buildable_improvements)
            .unwrap_or_default()
    }

    /// The number of work picker rows visible on screen right now.
    fn work_visible_rows(&self) -> usize {
        self.work_picker_rect
            .get()
            .map(|panel| work_picker::rows_rect(panel).height as usize)
            .unwrap_or(work_picker::MAX_VISIBLE_ROWS as usize)
    }

    /// Open the work picker over the map, but only when the selected unit is
    /// a settler (the only unit class that builds improvements).
    fn open_work_picker(&mut self) {
        let is_settler = self
            .engine
            .as_ref()
            .and_then(|engine| {
                self.selected_unit
                    .and_then(|unit| engine.player_units().into_iter().find(|u| u.id() == unit))
            })
            .is_some_and(|u| u.unit_class == UnitClass::Settler);
        if !is_settler {
            return;
        }
        self.work_picker_open = true;
        self.work_picker_cursor = 0;
        self.work_picker_scroll = 0;
    }

    fn close_work_picker(&mut self) {
        self.work_picker_open = false;
        self.work_picker_scroll = 0;
        self.work_picker_rect.set(None);
    }

    /// Issue the selected settler's chosen improvement order, then close.
    fn save_work_picker(&mut self) {
        let unit = self.selected_unit;
        let improvement = self.work_rows().get(self.work_picker_cursor).copied();
        self.close_work_picker();
        let (Some(unit), Some(improvement)) = (unit, improvement) else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::Work { unit, improvement });
            self.record_events(events);
        }
    }

    /// Move the work picker's cursor by `delta` rows, keeping it in view.
    fn move_work_cursor(&mut self, delta: isize) {
        let len = self.work_rows().len();
        if len == 0 {
            return;
        }
        self.work_picker_cursor = if delta > 0 {
            advance(self.work_picker_cursor, len)
        } else {
            retreat(self.work_picker_cursor, len)
        };
        let visible = self.work_visible_rows();
        let max_offset = len.saturating_sub(visible);
        let row = self.work_picker_cursor;
        let lo = (row as isize + 1 - visible as isize).max(0) as usize;
        let hi = row.min(max_offset);
        self.work_picker_scroll = self.work_picker_scroll.clamp(lo, hi);
    }

    /// Cancel the selected unit's order (fortify, sentry, or improvement).
    fn cancel_selected_unit_order(&mut self) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::CancelOrder { unit });
            self.record_events(events);
        }
    }

    fn handle_work_picker_mouse(&mut self, panel: Rect, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.work_picker_scroll = self.work_picker_scroll.saturating_sub(1);
                return;
            }
            MouseEventKind::ScrollDown => {
                let rows = self.work_rows();
                let max_offset = rows.len().saturating_sub(self.work_visible_rows());
                self.work_picker_scroll = (self.work_picker_scroll + 1).min(max_offset);
                return;
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return,
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        if !panel.contains(position) {
            // Clicks outside the picker are swallowed while it is open.
            return;
        }
        if work_picker::cancel_button_rect(panel).contains(position) {
            self.close_work_picker();
            return;
        }
        if work_picker::save_button_rect(panel).contains(position) {
            self.save_work_picker();
            return;
        }
        let rows = work_picker::rows_rect(panel);
        if rows.contains(position) {
            let row_in_view = (mouse.row as usize).saturating_sub(rows.y as usize);
            let global_row = row_in_view + self.work_picker_scroll;
            if global_row < self.work_rows().len() {
                self.work_picker_cursor = global_row;
            }
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
/// the next unused name in the civilization's list of city names.
fn city_name_for(engine: &Engine, unit: UnitId) -> String {
    let unit = engine.player_units().into_iter().find(|u| u.id() == unit);
    let Some(unit) = unit else {
        return "City".to_string();
    };
    let civ = engine.civilization_of(unit.owner());
    next_city_name(civ, engine.player_cities().len())
}

/// The name for a civilization's next city, given how many cities it already
/// has. Cities are named in order of founding.
fn next_city_name(civ: Civilization, existing_cities: usize) -> String {
    match civ.city_names().get(existing_cities) {
        Some(name) => (*name).to_string(),
        None => format!("City {}", existing_cities + 1),
    }
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

/// The adjacent `Direction` from tile `from` to tile `to`, taking the map's
/// horizontal wrap into account. Returns `None` unless the two tiles are
/// orthogonally or diagonally adjacent.
fn adjacent_direction(from: (usize, usize), to: (usize, usize), map_w: usize) -> Option<Direction> {
    let (fx, fy) = from;
    let (tx, ty) = to;
    let mut dx = tx as isize - fx as isize;
    if dx > (map_w / 2) as isize {
        dx -= map_w as isize;
    } else if dx < -((map_w / 2) as isize) {
        dx += map_w as isize;
    }
    let dy = ty as isize - fy as isize;
    let direction = match (dx, dy) {
        (0, -1) => Direction::N,
        (1, -1) => Direction::NE,
        (1, 0) => Direction::E,
        (1, 1) => Direction::SE,
        (0, 1) => Direction::S,
        (-1, 1) => Direction::SW,
        (-1, 0) => Direction::W,
        (-1, -1) => Direction::NW,
        _ => return None,
    };
    Some(direction)
}

/// The world tile under the pointer, when it lies one square away from the
/// `selected` tile in any direction; `None` otherwise (including over the
/// left column or beyond the bottom map edge).
fn hovered_adjacent_tile(
    screen: (u16, u16),
    camera: (usize, usize),
    map: (usize, usize),
    selected: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let (column, row) = screen;
    if column < LEFT_COLUMN_WIDTH {
        return None;
    }
    let (map_w, map_h) = map;
    let tile_col = (column - LEFT_COLUMN_WIDTH) as usize / TILE_WIDTH;
    let world_x = (camera.0 + tile_col) % map_w;
    let world_y = camera.1 + row as usize;
    if world_y >= map_h {
        return None;
    }
    adjacent_direction(selected?, (world_x, world_y), map_w)?;
    Some((world_x, world_y))
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
    use crate::model::units::UnitOrder;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;

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
            if app.engine.as_ref().unwrap().current_player_id() == PlayerId::new(0) || presses >= 6
            {
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
}
