use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

impl App {
    /// Routes mouse input during play. Left presses over the map start a
    /// click-or-drag gesture: the click is deferred until the button is
    /// released without the pointer having travelled beyond `CLICK_SLOP`, so
    /// dragging the map never accidentally moves a unit or opens a city.
    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) {
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
        // The diplomacy window floats above everything and captures all mouse
        // input while it is open.
        if self.diplomacy_rect.get().is_some() {
            self.handle_diplomacy_mouse(mouse);
            return;
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
    pub(super) fn mouse_pressed(&mut self) {
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
    pub(super) fn mouse_dragged(&mut self) {
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
    pub(super) fn mouse_released(&mut self) {
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
    pub(super) fn map_click(&mut self, column: u16, row: u16) {
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
}
