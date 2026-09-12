use super::*;
use crate::game_engine::Command;
use crate::model::cartography::Location;
use crossterm::event::KeyCode;
use strum::IntoEnumIterator;

impl App {
    pub(super) fn handle_playing_key(&mut self, key: KeyEvent) -> bool {
        // While the research dialog is open it captures the keyboard: the
        // player may only pick a research target and confirm it.
        if self.research_dialog.is_some() {
            self.handle_research_dialog_key(key);
            return false;
        }
        // While the diplomacy window is open it captures the keyboard: the
        // player may only choose to declare war or remain at peace.
        if self.diplomacy.is_some() {
            self.handle_diplomacy_key(key);
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

    pub(super) fn select_first_unit(&mut self) {
        if let Some(engine) = &self.engine {
            self.selected_unit = engine.player_units().first().map(|unit| unit.id());
        } else {
            self.selected_unit = None;
        }
        self.camera_follow.set(true);
    }

    pub(super) fn cycle_unit_selection(&mut self) {
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

    pub(super) fn move_selected_unit(&mut self, direction: Direction) {
        if let Some(unit) = self.selected_unit {
            self.move_unit(unit, direction);
        }
    }

    pub(super) fn move_unit(&mut self, unit: UnitId, direction: Direction) {
        // The tile the move aims at, computed before the move: combat always
        // happens on the destination square, so that is where the flash goes.
        // The step wraps east/west like the map does.
        let destination = self.engine.as_ref().and_then(|engine| {
            engine
                .player_units()
                .iter()
                .find(|u| u.id() == unit)
                .and_then(|u| {
                    let (width, height) = (engine.width(), engine.height());
                    let (dx, dy) = direction.delta();
                    let x = (u.location.x as isize + dx).rem_euclid(width as isize);
                    let y = u.location.y as isize + dy;
                    if y >= 0 && y < height as isize {
                        Some(Location::new(x as u16, y as u16))
                    } else {
                        None
                    }
                })
        });
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::Move { unit, direction });
            if events
                .iter()
                .any(|event| event.message().contains("attacks"))
            {
                if let Some(location) = destination {
                    self.battle_animation = Some(BattleAnimation {
                        location,
                        start: self.started_at.elapsed(),
                    });
                }
                // The battle may have cost the player their acting unit (a
                // repelled attacker is removed). Drop the stale selection so
                // the map stops flashing a ghost.
                if engine.player_units().iter().all(|u| u.id() != unit) {
                    self.selected_unit = None;
                }
            }
            // A freshly met rival presents the war-or-peace choice.
            if let Some(opponent) = events
                .iter()
                .find_map(|event| self.contact_opponent(event.message()))
            {
                self.diplomacy = Some(DiplomacyState {
                    opponent,
                    origin: DiplomacyOrigin::Contact,
                    choice: DiplomacyChoice::Peace,
                    pending: None,
                });
            }
            // A step onto a peaceful foreign tile offers the same choice, and
            // keeps the move pending so declaring war retakes it as an attack.
            if self.diplomacy.is_none()
                && events.iter().any(|event| {
                    event
                        .message()
                        .contains("cannot move onto a tile occupied by a civilization at peace")
                })
                && let Some(opponent) =
                    destination.and_then(|location| self.occupant_owner(location))
            {
                self.diplomacy = Some(DiplomacyState {
                    opponent,
                    origin: DiplomacyOrigin::Movement,
                    choice: DiplomacyChoice::Peace,
                    pending: Some((unit, direction)),
                });
            }
            self.record_events(events);
        }
        self.camera_follow.set(true);
    }

    /// The rival just met in `message` ("{Civ} and {Civ} meet for the first
    /// time"), stripped down to its player id. Only the human's moves trigger
    /// meetings, so one of the two names is always the human's civilization.
    pub(super) fn contact_opponent(&self, message: &str) -> Option<PlayerId> {
        if !message.contains(" meet for the first time") {
            return None;
        }
        let engine = self.engine.as_ref()?;
        let human = PlayerId::new(0);
        let human_civilization = engine.civilization_of(human);
        let other = Civilization::iter()
            .find(|civ| *civ != human_civilization && message.contains(civ.display_name()))?;
        engine.player_id_of(other)
    }

    /// The foreign owner of the city or unit standing on `location`, if any.
    pub(super) fn occupant_owner(&self, location: Location) -> Option<PlayerId> {
        let engine = self.engine.as_ref()?;
        let human = PlayerId::new(0);
        let (x, y) = (location.x as usize, location.y as usize);
        engine
            .units_at(x, y)
            .into_iter()
            .map(|unit| unit.owner())
            .find(|owner| *owner != human)
            .or_else(|| {
                engine
                    .city_at(x, y)
                    .map(|city| city.owner())
                    .filter(|owner| *owner != human)
            })
    }

    /// The world tile the pointer currently hovers, when it lies one square
    /// away from the selected unit (so a click would move there); `None`
    /// otherwise, including while a modal panel floats over the map.
    pub(super) fn hovered_move_target(&self, engine: &Engine) -> Option<(usize, usize)> {
        if self.research_dialog_rect.get().is_some()
            || self.diplomacy_rect.get().is_some()
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

    pub(super) fn found_selected_city(&mut self) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let name = city_name_for(engine, unit);
            let events = engine.submit(Command::FoundCity { unit, name });
            self.record_events(events);
        }
    }

    pub(super) fn end_turn(&mut self) {
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

    pub(super) fn record_events(&mut self, events: Vec<GameEvent>) {
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

    /// Cancel the selected unit's order (fortify, sentry, or improvement).
    pub(super) fn cancel_selected_unit_order(&mut self) {
        let Some(unit) = self.selected_unit else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let events = engine.submit(Command::CancelOrder { unit });
            self.record_events(events);
        }
    }
}
