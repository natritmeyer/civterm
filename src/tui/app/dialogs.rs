use super::*;
use crate::game_engine::{Command, load_game, save_game};
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

const DEFAULT_SAVE_PATH: &str = "civterm.civ";

impl App {
    pub(super) fn handle_research_dialog_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_research_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_research_cursor(1),
            KeyCode::Enter | KeyCode::Char(' ') => self.research_dialog_confirm(),
            _ => {}
        }
    }

    pub(super) fn handle_research_dialog_mouse(&mut self, mouse: MouseEvent) {
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

    pub(super) fn research_dialog_confirm(&mut self) {
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

    pub(super) fn handle_diplomacy_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => {
                self.diplomacy_choice(DiplomacyChoice::War)
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
                self.diplomacy_choice(DiplomacyChoice::Peace)
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.diplomacy_confirm(),
            KeyCode::Esc => self.diplomacy_cancel(),
            _ => {}
        }
    }

    pub(super) fn handle_diplomacy_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.diplomacy_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        if diplomacy_dialog::war_button_rect(panel).contains(position) {
            self.diplomacy_confirm_choice(DiplomacyChoice::War);
        } else if diplomacy_dialog::peace_button_rect(panel).contains(position) {
            self.diplomacy_confirm_choice(DiplomacyChoice::Peace);
        }
    }

    /// Steer the diplomacy cursor to `choice`.
    pub(super) fn diplomacy_choice(&mut self, choice: DiplomacyChoice) {
        if let Some(state) = &mut self.diplomacy {
            state.choice = choice;
        }
    }

    /// Confirm the answer currently on the diplomacy cursor.
    pub(super) fn diplomacy_confirm(&mut self) {
        let choice = self.diplomacy.map(|state| state.choice);
        if let Some(choice) = choice {
            self.diplomacy_confirm_choice(choice);
        }
    }

    /// Close the diplomacy window and act on the given answer.
    pub(super) fn diplomacy_confirm_choice(&mut self, choice: DiplomacyChoice) {
        let Some(state) = self.diplomacy.take() else {
            return;
        };
        self.diplomacy_rect.set(None);
        match choice {
            DiplomacyChoice::War => {
                if let Some(engine) = &mut self.engine {
                    let events = engine.submit(Command::DeclareWar {
                        opponent: state.opponent,
                    });
                    self.record_events(events);
                }
                // The blocked step becomes the first attack of the war.
                if let Some((unit, direction)) = state.pending {
                    self.move_unit(unit, direction);
                }
            }
            DiplomacyChoice::Peace => {
                if let Some(engine) = &mut self.engine {
                    let events = engine.submit(Command::MakePeace {
                        opponent: state.opponent,
                    });
                    self.record_events(events);
                }
            }
        }
    }

    /// Close the diplomacy window without declaring war; a pending move is
    /// abandoned and the unit stays put.
    pub(super) fn diplomacy_cancel(&mut self) {
        self.diplomacy = None;
        self.diplomacy_rect = Cell::new(None);
    }

    /// Move the research dialog's cursor by `delta` rows.
    pub(super) fn move_research_cursor(&mut self, delta: isize) {
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
    pub(super) fn move_research_cursor_to(&mut self, choice: usize) {
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
    pub(super) fn move_research_scroll(&mut self, delta: isize) {
        let visible = self.research_visible_rows();
        let Some(dialog) = &mut self.research_dialog else {
            return;
        };
        let base = dialog.scroll as isize + delta;
        let max_offset = dialog.choices.len().saturating_sub(visible);
        dialog.scroll = base.clamp(0, max_offset as isize) as usize;
    }

    /// Keep the scroll so the cursor stays inside the scrolled window.
    pub(super) fn clamp_research_scroll(dialog: &mut ResearchDialogState, visible: usize) {
        let max_offset = dialog.choices.len().saturating_sub(visible);
        let lo = (dialog.cursor as isize + 1 - visible as isize).max(0) as usize;
        let hi = dialog.cursor.min(max_offset);
        dialog.scroll = dialog.scroll.clamp(lo, hi);
    }

    /// The number of research-dialog list rows visible on screen right now.
    pub(super) fn research_visible_rows(&self) -> usize {
        self.research_dialog_rect
            .get()
            .map(|panel| research_dialog::list_rect(panel).height as usize)
            .unwrap_or(8)
    }

    /// Open the save prompt in play. The path starts on a predictable default
    /// so a plain Enter saves without typing.
    pub(super) fn open_save_prompt(&mut self) {
        self.save_prompt = Some(SaveLoadState {
            kind: SaveLoadKind::Save,
            input: DEFAULT_SAVE_PATH.to_string(),
            error: None,
        });
    }

    /// Open the load prompt from the menu's "Load Saved Game" item.
    pub(super) fn open_load_prompt(&mut self) {
        self.selected = 1;
        self.save_prompt = Some(SaveLoadState {
            kind: SaveLoadKind::Load,
            input: String::new(),
            error: None,
        });
    }

    /// Handle a keystroke while the save/load prompt is open: the prompt
    /// captures everything except its own controls.
    pub(super) fn handle_save_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c)
                if !c.is_control()
                    && (key.modifiers.is_empty()
                        || key.modifiers.contains(KeyModifiers::SHIFT)) =>
            {
                let Some(state) = &mut self.save_prompt else {
                    return;
                };
                state.error = None;
                if state.input.chars().count() < 200 {
                    state.input.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(state) = &mut self.save_prompt {
                    state.input.pop();
                }
            }
            KeyCode::Enter => self.save_prompt_confirm(),
            KeyCode::Esc => self.close_save_prompt(),
            _ => {}
        }
    }

    /// Close the prompt without acting on the typed path.
    pub(super) fn close_save_prompt(&mut self) {
        self.save_prompt = None;
        self.save_prompt_rect.set(None);
    }

    /// Handle a keystroke while the quit dialog is open: cycle between
    /// continue, save and quit, confirm with Enter, back out with Esc.
    /// Confirming quit returns `true`, ending the run loop.
    pub(super) fn handle_quit_dialog_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => {
                self.quit_choice_rotate(-1)
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
                self.quit_choice_rotate(1)
            }
            KeyCode::Enter | KeyCode::Char(' ') => return self.quit_dialog_confirm(),
            KeyCode::Esc => self.close_quit_dialog(),
            _ => {}
        }
        false
    }

    /// Steer the quit dialog's cursor one step through continue → save →
    /// quit, wrapping both ways.
    pub(super) fn quit_choice_rotate(&mut self, delta: isize) {
        let Some(choice) = &mut self.quit_dialog else {
            return;
        };
        *choice = if delta > 0 {
            choice.next()
        } else {
            choice.prev()
        };
    }

    /// Confirm the answer currently on the quit-dialog cursor.
    pub(super) fn quit_dialog_confirm(&mut self) -> bool {
        let Some(choice) = self.quit_dialog else {
            return false;
        };
        self.quit_dialog_confirm_choice(choice)
    }

    /// Close the quit dialog and act on the given answer: continue leaves the
    /// game untouched, save opens the save prompt, and quit ends the process.
    /// Returns `true` when the process should end.
    pub(super) fn quit_dialog_confirm_choice(&mut self, choice: QuitChoice) -> bool {
        if self.quit_dialog.is_none() {
            return false;
        }
        self.quit_dialog = None;
        self.quit_dialog_rect.set(None);
        match choice {
            QuitChoice::Continue => {}
            QuitChoice::Save => self.open_save_prompt(),
            QuitChoice::Quit => return true,
        }
        false
    }

    /// Close the quit dialog without acting on it; play resumes underneath.
    pub(super) fn close_quit_dialog(&mut self) {
        self.quit_dialog = None;
        self.quit_dialog_rect.set(None);
    }

    /// A click on one of the quit dialog's buttons acts as if that answer had
    /// been confirmed with the keyboard. A clicked QUIT sets the exit flag:
    /// mouse input has no return channel, so the run loop leaves on its next
    /// tick.
    pub(super) fn handle_quit_dialog_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.quit_dialog_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        let choice = if quit_dialog::continue_button_rect(panel).contains(position) {
            Some(QuitChoice::Continue)
        } else if quit_dialog::save_button_rect(panel).contains(position) {
            Some(QuitChoice::Save)
        } else if quit_dialog::quit_button_rect(panel).contains(position) {
            Some(QuitChoice::Quit)
        } else {
            None
        };
        if let Some(choice) = choice
            && self.quit_dialog_confirm_choice(choice)
        {
            self.exit_requested = true;
        }
    }

    /// Act on the typed path: save or load as the prompt kind demands. An
    /// empty path or a failed operation keeps the prompt open showing why.
    pub(super) fn save_prompt_confirm(&mut self) {
        let Some(state) = self.save_prompt.take() else {
            return;
        };
        self.save_prompt_rect.set(None);
        let path = state.input.trim().to_string();
        if path.is_empty() {
            self.save_prompt = Some(SaveLoadState {
                kind: state.kind,
                input: state.input,
                error: Some("Enter a path".to_string()),
            });
            return;
        }
        match state.kind {
            SaveLoadKind::Save => self.perform_save(&path),
            SaveLoadKind::Load => self.perform_load(&path),
        }
    }

    /// Write the current game to `path`.
    fn perform_save(&mut self, path: &str) {
        let Some(engine) = self.engine.take() else {
            return;
        };
        let competition = self
            .game_competition
            .unwrap_or(Competition::new(Competition::MIN));
        let difficulty = self.game_difficulty.unwrap_or(Difficulty::Normal);
        let result = save_game(path, &engine, competition, difficulty);
        self.engine = Some(engine);
        match result {
            Ok(()) => self.record_events(vec![GameEvent::new(format!("Game saved to {path}"))]),
            Err(err) => {
                self.save_prompt = Some(SaveLoadState {
                    kind: SaveLoadKind::Save,
                    input: path.to_string(),
                    error: Some(err.to_string()),
                });
            }
        }
    }

    /// Load the game at `path`, replacing the current one if successful.
    fn perform_load(&mut self, path: &str) {
        match load_game(path) {
            Ok(loaded) => {
                self.engine = Some(loaded.engine);
                self.game_competition = Some(loaded.competition);
                self.game_difficulty = Some(loaded.difficulty);
                self.event_log.clear();
                self.enter_playing();
                self.record_events(vec![GameEvent::new(format!("Loaded game from {path}"))]);
            }
            Err(err) => {
                self.save_prompt = Some(SaveLoadState {
                    kind: SaveLoadKind::Load,
                    input: path.to_string(),
                    error: Some(err.to_string()),
                });
            }
        }
    }
}
