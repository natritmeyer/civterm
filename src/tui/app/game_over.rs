use super::*;
use crate::game_engine::GameOutcome;
use crate::game_engine::GameView;
use crate::model::civilizations::PlayerId;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

impl App {
    /// Refresh the end-of-match overlay from the engine's own state. Called
    /// after every command submission (via `record_events`), so whichever
    /// command lands the killing blow — the fall of the human's last city or
    /// unit, or the capture that finishes the last rival — immediately raises
    /// the curtain. It also catches `EndTurn`, where the rivals take their
    /// turns and the board may be cleared, and a freshly loaded game that was
    /// already over.
    pub(super) fn check_game_over(&mut self) {
        if self.game_over.is_some() {
            return;
        }
        let Some(engine) = &self.engine else {
            return;
        };
        let Some(outcome) = engine.game_outcome() else {
            return;
        };
        let civ_name = engine
            .civilization_of(PlayerId::new(0))
            .display_name()
            .to_string();
        // Every floating panel is dropped so the overlay owns the screen.
        self.selected_city = None;
        self.city_window_scroll = 0;
        self.moused_window = Cell::new(None);
        self.production_picker_open = false;
        self.picker_rect = Cell::new(None);
        self.research_dialog = None;
        self.research_dialog_rect = Cell::new(None);
        self.diplomacy = None;
        self.diplomacy_rect = Cell::new(None);
        self.command_picker_open = false;
        self.command_picker_cursor = 0;
        self.command_picker_scroll = 0;
        self.command_picker_rect = Cell::new(None);
        self.quit_dialog = None;
        self.quit_dialog_rect = Cell::new(None);
        self.save_prompt = None;
        self.save_prompt_rect = Cell::new(None);
        // The battle flash is a transient map overlay, never a panel: it may
        // linger (hidden) beneath the end screen, and clearing it here would
        // erase the very flash the killing blow just started.
        self.rival_animation = None;
        self.game_over = Some(GameOverState {
            outcome,
            civ_name,
            first_selected: true,
        });
    }

    /// Keys while the end-of-match overlay shows: the cursor moves between the
    /// two buttons with the arrow keys (or tab), and Enter or Space confirms.
    pub(super) fn handle_game_over_key(&mut self, key: KeyEvent) {
        let Some(state) = &mut self.game_over else {
            return;
        };
        match key.code {
            KeyCode::Left => state.first_selected = true,
            KeyCode::Right => state.first_selected = false,
            KeyCode::BackTab | KeyCode::Tab => {
                state.first_selected = !state.first_selected;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let outcome = state.outcome;
                let first = state.first_selected;
                self.confirm_game_over(outcome, first);
            }
            _ => {}
        }
    }

    /// A click on one of the two end-of-match buttons activates it, exactly as
    /// the keyboard's Enter would.
    pub(super) fn handle_game_over_mouse(&mut self, mouse: MouseEvent) {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if mouse.column == u16::MAX || mouse.row == u16::MAX {
            return;
        }
        let Some((first_rect, second_rect)) = self.game_over_buttons.get() else {
            return;
        };
        let Some(state) = &self.game_over else {
            return;
        };
        let position = (mouse.column, mouse.row).into();
        let first = if first_rect.contains(position) {
            Some(true)
        } else if second_rect.contains(position) {
            Some(false)
        } else {
            None
        };
        if let Some(first) = first {
            let outcome = state.outcome;
            self.confirm_game_over(outcome, first);
        }
    }

    /// Act on the chosen button of the end-of-match overlay. The second button
    /// is always "Quit", ending the process; the defeat screen's "Start again"
    /// returns to the starting window, and the victory screen's "Continue
    /// playing" lifts the overlay and keeps the game going.
    fn confirm_game_over(&mut self, outcome: GameOutcome, first: bool) {
        if !first {
            self.exit_requested = true;
            return;
        }
        self.game_over = None;
        self.game_over_buttons.set(None);
        if outcome == GameOutcome::Defeat {
            self.phase = Phase::Menu;
        }
    }
}
