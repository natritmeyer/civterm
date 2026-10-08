use super::*;
use crate::game_engine::{
    Command, DiplomatAction, DiplomatAudience, StealOutcome, load_game, save_game,
};
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use crate::tui::diplomat_actions_dialog;
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
        // A research choice can share a round with one or more finished builds.
        // Those windows were parked behind this dialog, so they come back now
        // rather than being dropped: the player still gets told about every
        // city that finished.
        self.resume_build_notice();
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
        // The window suppressed the jump to the next unit with budget while it
        // was up (`modal_open`), so a move that spent the focus and opened it
        // never armed the advance. Back in the ordinary run loop now, arm it
        // again: the next unit that can take an order keeps its flash.
        self.schedule_unit_advance_if_spent();
    }

    /// Close the diplomacy window without declaring war; a pending move is
    /// abandoned and the unit stays put.
    pub(super) fn diplomacy_cancel(&mut self) {
        self.diplomacy = None;
        self.diplomacy_rect = Cell::new(None);
        // The window held the auto-advance off (`modal_open`); re-arm it now
        // that the run loop owns the screen again, so a unit spent before the
        // dialog opened still hands the flash on to the next one.
        self.schedule_unit_advance_if_spent();
    }

    /// Acknowledge the war-declaration window currently showing: drop the
    /// announced rival and close the window once no declaration remains.
    pub(super) fn war_notice_confirm(&mut self) {
        let Some(notice) = &mut self.war_notice else {
            return;
        };
        notice.queue.pop_front();
        if notice.queue.is_empty() {
            self.war_notice = None;
            self.war_notice_rect.set(None);
        }
    }

    /// Acknowledge the technology-stolen window currently showing: the prize
    /// has been read, and the window closes.
    pub(super) fn steal_notice_confirm(&mut self) {
        self.steal_notice = None;
        self.steal_notice_rect.set(None);
    }

    /// Acknowledge the sabotage-report window currently showing: the damage
    /// has been read, and the window closes.
    pub(super) fn sabotage_notice_confirm(&mut self) {
        self.sabotage_notice = None;
        self.sabotage_notice_rect.set(None);
    }

    /// Queue the build-completion windows for a finished round: one per city
    /// of the player's that finished a unit or improvement. Each is announced
    /// in turn, so a player with three cities building gets three windows
    /// rather than one that silently swallows two.
    pub(super) fn open_build_notices(&mut self, builds: Vec<BuildComplete>) {
        if builds.is_empty() {
            return;
        }
        self.build_notice = Some(BuildNoticeState {
            queue: builds.into(),
            suspended: false,
        });
        // The window floats over the map; drop any stale overlay so the player
        // is never asked about a build and a map selection at once.
        self.selected_city = None;
        self.city_window_scroll = 0;
        self.moused_window = Cell::new(None);
        self.command_picker_open = false;
        self.command_picker_rect = Cell::new(None);
        self.production_picker_open = false;
        self.picker_rect = Cell::new(None);
    }

    /// Acknowledge the build-completion window showing: drop the city it
    /// announced and close the window once no city is left waiting.
    pub(super) fn build_notice_confirm(&mut self) {
        let Some(notice) = &mut self.build_notice else {
            return;
        };
        notice.queue.pop_front();
        if notice.queue.is_empty() {
            self.build_notice = None;
            self.build_notice_rect.set(None);
        }
    }

    /// Open the announced city's own window so the player can set what it
    /// builds next, and park the notice until that window closes. If the city
    /// is gone — it could have been lost in the same round that finished its
    /// building — the notice simply moves on rather than opening nothing.
    pub(super) fn build_notice_next_order(&mut self) {
        let Some(notice) = &mut self.build_notice else {
            return;
        };
        let Some(done) = notice.queue.pop_front() else {
            return;
        };
        let city = done.city;
        let opens = self
            .engine
            .as_ref()
            .is_some_and(|engine| engine.city(city).is_some());
        if !opens {
            self.close_build_notice_if_empty();
            return;
        }
        notice.suspended = true;
        self.build_notice_rect.set(None);
        self.open_city_window(city);
    }

    /// Close the notice outright when its queue has run out.
    fn close_build_notice_if_empty(&mut self) {
        let empty = self
            .build_notice
            .as_ref()
            .is_none_or(|notice| notice.queue.is_empty());
        if empty {
            self.build_notice = None;
            self.build_notice_rect.set(None);
        }
    }

    /// The city the build-completion window is currently announcing, if the
    /// window is on screen. A suspended notice is deliberately excluded: the
    /// city window it opened owns the screen instead.
    pub(super) fn current_build_notice(&self) -> Option<&BuildComplete> {
        let notice = self.build_notice.as_ref()?;
        if notice.suspended {
            return None;
        }
        notice.queue.front()
    }

    /// A click on the build-completion window: OK moves on to the next city,
    /// "Next Order" opens this city's window instead.
    pub(super) fn handle_build_notice_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.build_notice_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        if build_complete_dialog::ok_button_rect(panel).contains(position) {
            self.build_notice_confirm();
        } else if build_complete_dialog::next_order_button_rect(panel).contains(position) {
            self.build_notice_next_order();
        }
    }

    /// Resume a build-completion notice parked by "Next Order": the city window
    /// it opened has closed, so the loop carries on with the next city. The
    /// notice is cleared outright once its queue has emptied, since a park with
    /// nothing behind it has nothing left to return to.
    pub(super) fn resume_build_notice(&mut self) {
        let Some(notice) = &mut self.build_notice else {
            return;
        };
        if !notice.suspended {
            return;
        }
        notice.suspended = false;
        self.close_build_notice_if_empty();
    }

    /// A click on the war-declaration window's OK button acknowledges it.
    pub(super) fn handle_war_notice_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.war_notice_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        if war_dialog::ok_button_rect(panel).contains((mouse.column, mouse.row).into()) {
            self.war_notice_confirm();
        }
    }

    /// A click on the technology-stolen window's OK button acknowledges it.
    pub(super) fn handle_steal_notice_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.steal_notice_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        if steal_dialog::ok_button_rect(panel).contains((mouse.column, mouse.row).into()) {
            self.steal_notice_confirm();
        }
    }

    /// A click on the sabotage-report window's OK button acknowledges it.
    pub(super) fn handle_sabotage_notice_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.sabotage_notice_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        if sabotage_dialog::ok_button_rect(panel).contains((mouse.column, mouse.row).into()) {
            self.sabotage_notice_confirm();
        }
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

    /// Offer the player's diplomat the five things he came into the city for.
    /// The record has already been drained from the engine, so this is the one
    /// place the window is opened from.
    pub(super) fn open_diplomat_actions(&mut self, audience: DiplomatAudience) {
        let Some(engine) = &self.engine else {
            return;
        };
        let options = engine.diplomat_options(audience.unit);
        self.diplomat_actions = Some(DiplomatActionsState {
            unit: audience.unit,
            city_name: audience.city_name,
            options,
            cursor: 0,
            from: audience.from,
            moves_before: audience.moves_before,
        });
    }
    pub(super) fn handle_diplomat_actions_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_diplomat_actions_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_diplomat_actions_cursor(1),
            // Enter confirms the row on the cursor, which is where the action
            // happens; Esc sends the diplomat on his way without spending him,
            // restoring him to the tile he entered from with his move budget
            // intact.
            KeyCode::Enter | KeyCode::Char(' ') => self.diplomat_actions_confirm(),
            KeyCode::Esc => self.close_diplomat_actions(),
            _ => {}
        }
    }
    /// Move the diplomat window's cursor by `delta` rows. The five actions are
    /// a fixed list, so the cursor wraps rather than stopping at the ends.
    pub(super) fn move_diplomat_actions_cursor(&mut self, delta: isize) {
        let Some(state) = &mut self.diplomat_actions else {
            return;
        };
        if state.options.is_empty() {
            return;
        }
        let total = state.options.len();
        state.cursor = if delta > 0 {
            advance(state.cursor, total)
        } else {
            retreat(state.cursor, total)
        };
    }
    /// Confirm the action on the cursor: it happens, and the window closes.
    ///
    /// A row the window greys out is not a choice, so confirming it does nothing
    /// at all — not even close the window. Closing on a refused offer would
    /// leave the player standing in a rival city with a diplomat they have not
    /// spent and no way to spend him, which is worse than the situation the
    /// window was opened to solve.
    pub(super) fn diplomat_actions_confirm(&mut self) {
        let Some(state) = &self.diplomat_actions else {
            return;
        };
        let Some(option) = state.options.get(state.cursor).cloned() else {
            return;
        };
        if option.blocked.is_some() {
            return;
        }
        let state = self
            .diplomat_actions
            .take()
            .expect("the window was showing a moment ago");
        let action = option.action;
        self.diplomat_actions_rect.set(None);
        let (investigated, stolen, sabotage) = if let Some(engine) = &mut self.engine {
            // The engine re-checks the rule rather than trusting the snapshot
            // this window drew: the window shows the choice as it stood when
            // the diplomat arrived, and the rules are what is enforced.
            let events = engine.submit(Command::DiplomatAction {
                unit: state.unit,
                action,
            });
            let investigated = if action == DiplomatAction::InvestigateCity {
                engine.drain_investigation()
            } else {
                None
            };
            let stolen = if action == DiplomatAction::StealTechnology {
                engine.drain_steal_outcome()
            } else {
                None
            };
            let sabotage = if action == DiplomatAction::IndustrialSabotage {
                engine.drain_sabotage_notice()
            } else {
                None
            };
            self.record_events(events);
            (investigated, stolen, sabotage)
        } else {
            (None, None, None)
        };
        // What the investigation bought is a look at the city: open its window
        // once, over the map, as a read-only report. The window is the moment
        // of the purchase and is not openable again — the evidence lives in the
        // `investigated` flag and the event log, not in a reopenable door.
        if let Some(city) = investigated {
            self.open_city_window(city);
        }
        // A theft that came away empty-handed is no purchase: the rival had no
        // advance the player could take, so nothing was spent and the diplomat
        // walks back out the way he came in — the same undo a dismissal performs.
        if matches!(&stolen, Some(StealOutcome::NothingToSteal { .. }))
            && let Some(engine) = &mut self.engine
        {
            let events = engine.submit(Command::WithdrawDiplomat {
                unit: state.unit,
                from: state.from,
                moves_before: state.moves_before,
            });
            self.record_events(events);
        }
        // What the theft bought is the advance itself; what it failed to buy is
        // the refusal, and both are announced in a window of their own.
        if let Some(stolen_outcome) = stolen {
            self.steal_notice = Some(stolen_outcome);
        }
        // What the sabotage bought is the damage: announce it in a window of
        // its own — which improvement came down, and in which city.
        if let Some(sabotaged) = sabotage {
            self.sabotage_notice = Some(sabotaged);
        }
    }
    /// Dismiss the window without acting. The diplomat stays standing in the
    /// city, unspent.
    pub(super) fn close_diplomat_actions(&mut self) {
        if let Some(state) = self.diplomat_actions.take()
            && let Some(engine) = &mut self.engine
        {
            let events = engine.submit(Command::WithdrawDiplomat {
                unit: state.unit,
                from: state.from,
                moves_before: state.moves_before,
            });
            self.record_events(events);
        }
        self.diplomat_actions_rect.set(None);
    }
    /// A click on the diplomat window: a row moves the cursor onto it, and OK
    /// confirms whatever row the cursor is then on.
    pub(super) fn handle_diplomat_actions_mouse(&mut self, mouse: MouseEvent) {
        let Some(panel) = self.diplomat_actions_rect.get() else {
            return;
        };
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let position = (mouse.column, mouse.row).into();
        if diplomat_actions_dialog::ok_button_rect(panel).contains(position) {
            self.diplomat_actions_confirm();
            return;
        }
        let total = self
            .diplomat_actions
            .as_ref()
            .map_or(0, |state| state.options.len());
        for index in 0..total {
            if diplomat_actions_dialog::row_rect(panel, index)
                .is_some_and(|row| row.contains(position))
            {
                let Some(state) = &mut self.diplomat_actions else {
                    return;
                };
                state.cursor = index;
                return;
            }
        }
    }
}
