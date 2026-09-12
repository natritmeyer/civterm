use super::*;
use crate::game_engine::Command;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

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
}
