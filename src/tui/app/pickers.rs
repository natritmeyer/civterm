use super::production_picker::PickRow;
use super::*;
use crate::game_engine::Command;
use crate::model::cities::ProductionTarget;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

impl App {
    /// The buildable targets the current player can pick, split into units
    /// and improvements (sorted for display by the picker module).
    pub(super) fn picker_rows(&self) -> (Vec<PickRow>, Vec<PickRow>) {
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
    pub(super) fn picker_visible_rows(&self) -> usize {
        self.picker_rect
            .get()
            .map(|panel| production_picker::rows_rect(panel).height as usize)
            .unwrap_or(production_picker::MAX_VISIBLE_ROWS as usize)
    }

    pub(super) fn open_production_picker(&mut self) {
        self.production_picker_open = true;
        self.picker_cursor_col = 0;
        self.picker_cursor_row = 0;
        self.picker_scroll = 0;
    }

    pub(super) fn close_production_picker(&mut self) {
        self.production_picker_open = false;
        self.picker_scroll = 0;
        self.picker_rect.set(None);
    }

    pub(super) fn save_production(&mut self) {
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
    pub(super) fn current_picker_target(&self) -> Option<ProductionTarget> {
        let rows = self.picker_rows();
        production_picker::target_at(&rows, self.picker_cursor_col, self.picker_cursor_row)
    }

    pub(super) fn handle_picker_mouse(&mut self, panel: Rect, mouse: MouseEvent) {
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
    pub(super) fn move_picker_cursor(&mut self, dx: isize, dy: isize) {
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

    /// The commands the selected unit can take on its tile right now.
    pub(super) fn command_rows(&self) -> Vec<command_picker::CommandChoice> {
        let Some(unit) = self.selected_unit else {
            return Vec::new();
        };
        let Some(engine) = &self.engine else {
            return Vec::new();
        };
        command_picker::available_commands(engine, unit)
    }

    /// The number of command picker rows visible on screen right now.
    pub(super) fn command_visible_rows(&self) -> usize {
        self.command_picker_rect
            .get()
            .map(|panel| command_picker::rows_rect(panel).height as usize)
            .unwrap_or(command_picker::MAX_VISIBLE_ROWS as usize)
    }

    /// Open the command picker over the map for the selected unit. Any unit
    /// on the map can be commanded; a selection that no longer resolves to a
    /// living unit opens nothing.
    pub(super) fn open_command_picker(&mut self) {
        let is_living = self
            .engine
            .as_ref()
            .and_then(|engine| {
                self.selected_unit
                    .and_then(|unit| engine.player_units().into_iter().find(|u| u.id() == unit))
            })
            .is_some();
        if !is_living {
            return;
        }
        self.command_picker_open = true;
        self.command_picker_cursor = 0;
        self.command_picker_scroll = 0;
    }

    pub(super) fn close_command_picker(&mut self) {
        self.command_picker_open = false;
        self.command_picker_scroll = 0;
        self.command_picker_rect.set(None);
    }

    /// Issue the selected unit's chosen command, then close. Fortify, sentry,
    /// work and their cancellations spend the turn; unfortify and unsentry
    /// are free and step the unit straight back into the command loop.
    pub(super) fn save_command_picker(&mut self) {
        let unit = self.selected_unit;
        let command = self.command_rows().get(self.command_picker_cursor).copied();
        self.close_command_picker();
        let (Some(unit), Some(command)) = (unit, command) else {
            return;
        };
        if let Some(engine) = &mut self.engine {
            let command = match command {
                command_picker::CommandChoice::Fortify => Command::Fortify { unit },
                command_picker::CommandChoice::Sentry => Command::Sentry { unit },
                command_picker::CommandChoice::Unfortify => Command::Unfortify { unit },
                command_picker::CommandChoice::Unsentry => Command::Unsentry { unit },
                command_picker::CommandChoice::Work(improvement) => {
                    Command::Work { unit, improvement }
                }
                command_picker::CommandChoice::CancelOrder(_) => Command::CancelOrder { unit },
            };
            let events = engine.submit(command);
            self.record_events(events);
            // A spending command leaves the focus spent: let it move on; a
            // free one restores the focus's agency and clears any arm.
            self.schedule_unit_advance_if_spent();
        }
    }

    /// Move the command picker's cursor by `delta` rows, keeping it in view.
    pub(super) fn move_command_cursor(&mut self, delta: isize) {
        let len = self.command_rows().len();
        if len == 0 {
            return;
        }
        self.command_picker_cursor = if delta > 0 {
            advance(self.command_picker_cursor, len)
        } else {
            retreat(self.command_picker_cursor, len)
        };
        let visible = self.command_visible_rows();
        let max_offset = len.saturating_sub(visible);
        let row = self.command_picker_cursor;
        let lo = (row as isize + 1 - visible as isize).max(0) as usize;
        let hi = row.min(max_offset);
        self.command_picker_scroll = self.command_picker_scroll.clamp(lo, hi);
    }

    pub(super) fn handle_command_picker_mouse(&mut self, panel: Rect, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.command_picker_scroll = self.command_picker_scroll.saturating_sub(1);
                return;
            }
            MouseEventKind::ScrollDown => {
                let rows = self.command_rows();
                let max_offset = rows.len().saturating_sub(self.command_visible_rows());
                self.command_picker_scroll = (self.command_picker_scroll + 1).min(max_offset);
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
        if command_picker::cancel_button_rect(panel).contains(position) {
            self.close_command_picker();
            return;
        }
        if command_picker::save_button_rect(panel).contains(position) {
            self.save_command_picker();
            return;
        }
        let rows = command_picker::rows_rect(panel);
        if rows.contains(position) {
            let row_in_view = (mouse.row as usize).saturating_sub(rows.y as usize);
            let global_row = row_in_view + self.command_picker_scroll;
            if global_row < self.command_rows().len() {
                self.command_picker_cursor = global_row;
            }
        }
    }
}
