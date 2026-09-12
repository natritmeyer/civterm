use super::*;
use crate::game_engine::Player;
use crate::tui::status_bar::ITEMS;
use crossterm::event::KeyCode;
use strum::IntoEnumIterator;

impl App {
    pub(super) fn handle_menu_key(&mut self, key: KeyEvent) -> bool {
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

    pub(super) fn handle_civ_key(&mut self, key: KeyEvent) -> bool {
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

    pub(super) fn handle_competition_key(&mut self, key: KeyEvent) -> bool {
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

    pub(super) fn handle_difficulty_key(&mut self, key: KeyEvent) -> bool {
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

    pub(super) fn handle_start_key(&mut self, key: KeyEvent) -> bool {
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

    pub(super) fn start_game(&mut self) {
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
        self.diplomacy = None;
        self.diplomacy_rect = Cell::new(None);
        self.work_picker_open = false;
        self.work_picker_cursor = 0;
        self.work_picker_scroll = 0;
        self.work_picker_rect = Cell::new(None);
    }

    pub(super) fn start_new_game(&mut self) {
        self.reset_setup();
        self.phase = Phase::ChoosingCiv;
    }

    pub(super) fn reset_setup(&mut self) {
        self.civ_index = 0;
        self.chosen_civ = None;
        self.competition_index = 0;
        self.chosen_competition = None;
        self.difficulty_index = 0;
        self.chosen_difficulty = None;
        self.start_choice = StartChoice::Start;
    }
}
