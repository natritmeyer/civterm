use std::cell::Cell;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::city_window::{self, CityWindow};
use super::civ_selector::CivSelector;
use super::competition_selector::CompetitionSelector;
use super::difficulty_selector::DifficultySelector;
use super::diplomacy_dialog::{self, DiplomacyChoice, DiplomacyDialog, DiplomacyOrigin};
use super::game_screen::{
    BATTLE_FLASH_DURATION, BattleAnimation, GameScreen, LEFT_COLUMN_WIDTH, TILE_WIDTH,
};
use super::playing_help::PlayingHelp;
use super::production_picker::{self, ProductionPicker};
use super::research_dialog::{self, ResearchDialog};
use super::splash::SplashScreen;
use super::start_confirm::StartConfirm;
use super::status_bar::StatusBar;
use super::work_picker::{self, WorkPicker};
use crate::game_engine::event::Event as GameEvent;
use crate::game_engine::{Engine, GameView};
use crate::model::advancements::Advancement;
use crate::model::cartography::Direction;
use crate::model::cities::CityId;
use crate::model::civilizations::{Civilization, PlayerId};
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use crate::model::units::{UnitClass, UnitId};

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

/// The war-or-peace dialog's live state: who it is about, why it opened, and
/// the answer currently on the cursor. A blocked move keeps the unit and
/// direction it would retake once war is declared.
#[derive(Clone, Copy)]
struct DiplomacyState {
    opponent: PlayerId,
    origin: DiplomacyOrigin,
    choice: DiplomacyChoice,
    pending: Option<(UnitId, Direction)>,
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
    /// The open war-or-peace dialog, if one is showing.
    diplomacy: Option<DiplomacyState>,
    /// The last-drawn diplomacy-dialog rectangle, for mouse hit-testing.
    diplomacy_rect: Cell<Option<Rect>>,
    /// Whether the work picker floats over the map.
    work_picker_open: bool,
    /// The picker cursor: which improvement row is selected.
    work_picker_cursor: usize,
    /// Vertical scroll offset of the work picker's list.
    work_picker_scroll: usize,
    /// The last-drawn work picker panel rectangle, for mouse hit-testing.
    work_picker_rect: Cell<Option<Rect>>,
    /// The in-flight battle flash on the defender's tile, if a combat has just
    /// resolved; `None` once the animation has run its course.
    battle_animation: Option<BattleAnimation>,
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
            diplomacy: None,
            diplomacy_rect: Cell::new(None),
            work_picker_open: false,
            work_picker_cursor: 0,
            work_picker_scroll: 0,
            work_picker_rect: Cell::new(None),
            battle_animation: None,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| Self::draw(frame, self))?;
            self.clear_expired_battle_animation(self.started_at.elapsed());
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

    /// The battle flash lasts `BATTLE_FLASH_DURATION`; once it has run that
    /// long, drop the state so later draws no longer reserve the defender's
    /// tile. `now` is the current value of the app clock.
    fn clear_expired_battle_animation(&mut self, now: Duration) {
        if self
            .battle_animation
            .is_some_and(|animation| now.saturating_sub(animation.start) >= BATTLE_FLASH_DURATION)
        {
            self.battle_animation = None;
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
                        )
                        .with_battle_animation(app.battle_animation),
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
                    // The diplomacy window floats above the map when a new
                    // civilization is met or a unit tries to cross a peaceful
                    // border, asking whether to declare war or stay at peace.
                    if let Some(state) = &app.diplomacy {
                        let rect = diplomacy_dialog::dialog_rect(area);
                        frame.render_widget(
                            DiplomacyDialog::new(
                                engine.civilization_of(state.opponent),
                                state.origin,
                                state.choice,
                            ),
                            rect,
                        );
                        app.diplomacy_rect.set(Some(rect));
                    } else {
                        app.diplomacy_rect.set(None);
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

mod dialogs;
mod mouse;
mod pickers;
mod playing;
mod setup;

#[cfg(test)]
mod tests;
