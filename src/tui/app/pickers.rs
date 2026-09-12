use super::production_picker::PickRow;
use super::*;
use crate::game_engine::Command;
use crate::model::cities::ProductionTarget;
use crate::model::geography::TerrainImprovement;
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

    /// The improvements the selected settler can build on its tile right now.
    pub(super) fn work_rows(&self) -> Vec<TerrainImprovement> {
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
    pub(super) fn work_visible_rows(&self) -> usize {
        self.work_picker_rect
            .get()
            .map(|panel| work_picker::rows_rect(panel).height as usize)
            .unwrap_or(work_picker::MAX_VISIBLE_ROWS as usize)
    }

    /// Open the work picker over the map, but only when the selected unit is
    /// a settler (the only unit class that builds improvements).
    pub(super) fn open_work_picker(&mut self) {
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

    pub(super) fn close_work_picker(&mut self) {
        self.work_picker_open = false;
        self.work_picker_scroll = 0;
        self.work_picker_rect.set(None);
    }

    /// Issue the selected settler's chosen improvement order, then close.
    pub(super) fn save_work_picker(&mut self) {
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
    pub(super) fn move_work_cursor(&mut self, delta: isize) {
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

    pub(super) fn handle_work_picker_mouse(&mut self, panel: Rect, mouse: MouseEvent) {
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
}
