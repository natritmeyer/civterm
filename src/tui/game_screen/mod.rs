use std::collections::HashSet;
use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::theme::{ACCENT, DARK_GREY, DIM, draw_text};
use crate::game_engine::{Event, GameView};
use crate::model::cartography::Location;
use crate::model::cities::CityId;
use crate::model::units::{UnitId, UnitOrder};

/// Fixed width of the left-hand information column.
pub const LEFT_COLUMN_WIDTH: u16 = 36;

/// Width of the event-log overlay when visible.
const EVENT_LOG_WIDTH: u16 = 44;

pub const TILE_WIDTH: usize = 2;

/// How long a battle's explosion flash stays on screen: a 💥 that sits on the
/// defended tile for a second, then disappears.
pub(crate) const BATTLE_FLASH_DURATION: Duration = Duration::from_millis(1000);

/// A single battle's on-screen flash. After a move attacks into an enemy
/// square the defender's tile shows a 💥 for `BATTLE_FLASH_DURATION`, then
/// reverts to its normal display. The explosion is anchored in the tile's left
/// column, where the two-cell-wide glyph exactly covers the tile: anchoring it
/// in the right column would spill it one cell into the neighbour to the east.
/// `start` is the value of `GameScreen.now` when combat happened, so the
/// animation is a pure function of the clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BattleAnimation {
    /// The defended world tile (already wrapped horizontally) that flashes.
    pub(crate) location: Location,
    /// The clock value when the battle was resolved.
    pub(crate) start: Duration,
}

impl BattleAnimation {
    /// The explosion glyph for `now`, or `None` once the flash has run
    /// `BATTLE_FLASH_DURATION`. Always anchored at the tile's left column so
    /// the two-cell-wide 💥 covers exactly that tile.
    fn glyph(&self, now: Duration) -> Option<&'static str> {
        let elapsed = now.saturating_sub(self.start);
        (elapsed < BATTLE_FLASH_DURATION).then_some("💥")
    }
}

/// Duration of each sub-phase inside a rival-move frame: 300ms on the starting
/// tile, 300ms on the ending tile, 600ms total per step.
pub(crate) const RIVAL_MOVE_PHASE: Duration = Duration::from_millis(300);

/// One movement step a rival unit took during its AI turn: the unit and the
/// tiles it left and entered. Frames are replayed in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RivalMoveFrame {
    pub(crate) unit_id: UnitId,
    pub(crate) from: Location,
    pub(crate) to: Location,
}

/// The full rival-move animation for one round: a sequence of per-step frames
/// played sequentially. `start` is the clock value when the round resolved.
#[derive(Clone, Debug)]
pub(crate) struct RivalMoveAnimation {
    pub(crate) start: Duration,
    pub(crate) frames: Vec<RivalMoveFrame>,
}

impl RivalMoveAnimation {
    /// Whether the animation has run to completion at the given clock.
    pub(crate) fn is_complete(&self, now: Duration) -> bool {
        let elapsed = now.saturating_sub(self.start);
        let total = self.frames.len() as u32 * RIVAL_MOVE_PHASE.as_millis() as u32 * 2;
        elapsed.as_millis() as u32 >= total
    }

    /// The unit IDs that should be hidden from their game-state position: all
    /// units whose frames have not yet fully played out.
    pub(crate) fn hidden_units(&self, now: Duration) -> HashSet<UnitId> {
        let elapsed = now.saturating_sub(self.start);
        let frame_ms = RIVAL_MOVE_PHASE.as_millis() as u32 * 2;
        let active_idx = (elapsed.as_millis() as u32 / frame_ms) as usize;
        self.frames
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= active_idx)
            .map(|(_, frame)| frame.unit_id)
            .collect()
    }

    /// The active frame and whether we are in the "from" sub-phase (first
    /// half) or "to" sub-phase (second half). `None` once the animation has
    /// expired.
    pub(crate) fn active_frame(&self, now: Duration) -> Option<(&RivalMoveFrame, bool)> {
        let elapsed = now.saturating_sub(self.start);
        let frame_ms = RIVAL_MOVE_PHASE.as_millis() as u32 * 2;
        let idx = (elapsed.as_millis() as u32 / frame_ms) as usize;
        let frame = self.frames.get(idx)?;
        let in_from_phase =
            (elapsed.as_millis() as u32 % frame_ms) < RIVAL_MOVE_PHASE.as_millis() as u32;
        Some((frame, in_from_phase))
    }
}

pub struct GameScreen<'a> {
    view: &'a dyn GameView,
    focus: Option<(usize, usize)>,
    camera: (usize, usize),
    selected_unit: Option<UnitId>,
    selected_city: Option<CityId>,
    /// An instant pushed forward every frame; drives the idle-unit flash.
    now: Duration,
    /// Whether the idle-unit flash may animate: the app pins it off while a
    /// floating window or dialog covers the board, so the blinking does not
    /// compete with the panel for attention.
    flash_enabled: bool,
    /// Whether the event log overlays the map pane's top-right corner.
    show_events: bool,
    /// The most recent event messages, oldest first.
    events: &'a [Event],
    /// The world tile (wrapped horizontally) hatched with `▓` to show where a
    /// click would move the selected unit; `None` while nothing is hovered.
    hover_target: Option<(usize, usize)>,
    /// The world tile currently under the pointer (wrapped horizontally),
    /// whose terrain, improvements and units the focus panel lists; `None`
    /// while nothing is hovered. The panel only draws the block for tiles the
    /// player has discovered.
    hovered_tile: Option<(usize, usize)>,
    /// The in-flight battle explosion on the defender's tile, if any.
    battle_animation: Option<BattleAnimation>,
    /// The rival-movement replay for the round just resolved, if any.
    rival_animation: Option<&'a RivalMoveAnimation>,
}

impl<'a> GameScreen<'a> {
    /// `camera` is the world-tile coordinate at the top-left of the map pane.
    /// `selected_unit` (if any) is the unit whose tile flashes while it awaits
    /// instruction; `now` is a monotonic clock used to time that flash.
    /// `show_events` toggles the event log overlay, which shows `events`
    /// (most recent messages, oldest first) in the top-right of the map pane.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view: &'a dyn GameView,
        focus: Option<(usize, usize)>,
        camera: (usize, usize),
        selected_unit: Option<UnitId>,
        selected_city: Option<CityId>,
        now: Duration,
        show_events: bool,
        events: &'a [Event],
        hover_target: Option<(usize, usize)>,
    ) -> Self {
        GameScreen {
            view,
            focus,
            camera,
            selected_unit,
            selected_city,
            now,
            show_events,
            events,
            hover_target,
            hovered_tile: None,
            battle_animation: None,
            rival_animation: None,
            flash_enabled: true,
        }
    }

    /// Show a battle explosion on the defender's tile while the flash is
    /// running.
    pub(crate) fn with_battle_animation(mut self, animation: Option<BattleAnimation>) -> Self {
        self.battle_animation = animation;
        self
    }

    /// The world tile whose details the focus panel lists beneath the focus
    /// block: whatever the pointer currently hovers over.
    pub(crate) fn with_hovered_tile(mut self, tile: Option<(usize, usize)>) -> Self {
        self.hovered_tile = tile;
        self
    }

    /// Replay the round's rival movements over the map while they animate.
    pub(crate) fn with_rival_animation(
        mut self,
        animation: Option<&'a RivalMoveAnimation>,
    ) -> Self {
        self.rival_animation = animation;
        self
    }

    /// Pin the idle-unit flash on or off: the app disables it while a window
    /// or dialog covers the board.
    pub(crate) fn with_flash_enabled(mut self, enabled: bool) -> Self {
        self.flash_enabled = enabled;
        self
    }

    /// Whether the selected idle unit's tile is currently showing the
    /// civilization flash colour: on for half a second, off for half a second,
    /// repeating once per second — unless the flash is disabled, in which case
    /// the tile is pinned to the off phase.
    fn flash_phase(&self) -> bool {
        self.flash_enabled && self.now.as_millis() % 1000 < 500
    }

    fn draw_main_map(&self, area: Rect, buf: &mut Buffer) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let cell_cols = area.width as usize / TILE_WIDTH;
        let cell_rows = area.height as usize;
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }

        let flashing = self.flash_phase();
        // City labels are drawn in a second pass, after every tile row has been
        // painted, so a label on one row's tiles is not overwritten by the next
        // map row below it.
        let mut city_labels: Vec<(u16, u16, String)> = Vec::new();

        // The battle flash is painted in a final pass after every tile has
        // been drawn, so it sits on top of the whole map — tiles, units,
        // markers and city labels. When the animation is running, this holds
        // the painted tile's left column; the two-column-wide 💥 covers the
        // tile from there.
        let mut flash_cell: Option<(u16, u16)> = None;

        // Rival units replaying their round are lifted off their game-state
        // squares so the overlay, not the tile pass, places them.
        let hidden_units = self
            .rival_animation
            .map(|animation| animation.hidden_units(self.now))
            .unwrap_or_default();

        // Tiles are shown at 1:1; `camera` is the top-left world tile. The map
        // wraps horizontally (east/west) but not vertically — tiles beyond the
        // north/south edge render as void.
        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let src_x = self.camera.0 + col;
                let src_y = self.camera.1 + row;
                let cx = area.x + (col * TILE_WIDTH) as u16;
                let cy = area.y + row as u16;

                if let Some(name) = paint_tile(
                    buf,
                    cx,
                    cy,
                    self.view,
                    src_x,
                    src_y,
                    map_w,
                    map_h,
                    self.selected_city,
                    self.selected_unit,
                    flashing,
                    self.hover_target,
                    &hidden_units,
                ) {
                    city_labels.push((cx, cy + 1, name));
                }

                if let Some(animation) = self.battle_animation
                    && src_x % map_w == animation.location.x as usize
                    && src_y == animation.location.y as usize
                    && animation.glyph(self.now).is_some()
                {
                    flash_cell = Some((cx, cy));
                }
            }
        }

        // Second pass: city name labels sit on top of the map.
        for (tile_cx, row_y, name) in city_labels {
            draw_city_label(buf, tile_cx, row_y, &name);
        }

        // Third and final pass: the battle explosion, anchored in the tile's
        // left column so the wide glyph covers exactly the defended tile, with
        // the tile's right column blanked beneath it. Painted last, it is at
        // the top of the z-order and nothing the map draws can cover it.
        if let Some((cx, cy)) = flash_cell
            && let Some(animation) = self.battle_animation
            && let Some(glyph) = animation.glyph(self.now)
        {
            if let Some(cell) = buf.cell_mut((cx, cy)) {
                cell.set_symbol(glyph);
            }
            if TILE_WIDTH > 1
                && let Some(cell) = buf.cell_mut((cx + 1, cy))
            {
                cell.set_symbol(" ");
            }
        }

        // The rival-movement replay draws the actively moving unit on top of
        // everything the map paints, anchored in the movement step's current
        // tile (the two-cell tile's left column), in the owner's colour.
        if let Some(animation) = self.rival_animation
            && let Some((frame, in_from_phase)) = animation.active_frame(self.now)
        {
            let tile = if in_from_phase { frame.from } else { frame.to };
            let screen_col = (tile.x as usize + map_w - self.camera.0) % map_w;
            let screen_row = tile.y as isize - self.camera.1 as isize;
            if screen_col < cell_cols
                && (0..cell_rows as isize).contains(&screen_row)
                && let Some(unit) = self.view.unit(frame.unit_id)
            {
                let cx = area.x + (screen_col * TILE_WIDTH) as u16;
                let cy = area.y + screen_row as u16;
                let style = Style::default()
                    .fg(civilization_color(self.view.civilization_of(unit.owner())))
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED);
                if let Some(cell) = buf.cell_mut((cx, cy)) {
                    cell.set_symbol(&first_letter(unit.unit_class).to_string());
                    cell.set_style(style);
                }
            }
        }
    }

    /// Overlays the event log on the top-right corner of the map pane. The box
    /// shows up to five most-recent messages, newest at the bottom, or a
    /// placeholder when none have been recorded yet.
    fn draw_event_log(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_events {
            return;
        }
        let rows = self.events.len().clamp(1, 5) as u16 + 2;
        let width = area.width.min(EVENT_LOG_WIDTH);
        if width < 4 || rows > area.height {
            return;
        }
        let left = area.right() - width;
        let top = area.y;
        let right = area.right() - 1;
        let bottom = top + rows - 1;

        let bg = Color::Rgb(12, 12, 40);
        let border = Style::default().fg(DIM).bg(bg);
        let message_style = Style::default().fg(Color::Rgb(220, 220, 235)).bg(bg);
        let title_style = Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD)
            .bg(bg);
        let inner_width = (width - 2) as usize;

        // Fill the box so the map underneath does not show through.
        for y in top..=bottom {
            for x in left..=right {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_symbol(" ");
                    cell.set_bg(bg);
                }
            }
        }

        // Top border with the title centred.
        fill_row(buf, left, right, top, "─", border);
        set_cell(buf, left, top, "┌", border);
        set_cell(buf, right, top, "┐", border);
        let title = " EVENTS ";
        let title_x = left + (width - title.chars().count() as u16) / 2;
        draw_text(buf, right, title_x, top, title, title_style);

        // Message rows, newest at the bottom. `events` holds at most five in
        // oldest-first order, so the last one lands on the final message row.
        // With no events yet, show a placeholder so the box is still visible.
        let start = self.events.len().saturating_sub(5);
        for offset in 0..rows.saturating_sub(2) as usize {
            let y = top + 1 + offset as u16;
            let visible = match self.events.get(start + offset) {
                Some(event) => event
                    .message()
                    .chars()
                    .take(inner_width)
                    .collect::<String>(),
                None => "(no events yet)"
                    .chars()
                    .take(inner_width)
                    .collect::<String>(),
            };
            draw_text(buf, right, left + 1, y, &visible, message_style);
        }

        // Bottom border.
        fill_row(buf, left, right, bottom, "─", border);
        set_cell(buf, left, bottom, "└", border);
        set_cell(buf, right, bottom, "┘", border);
    }

    fn draw_minimap(&self, area: Rect, buf: &mut Buffer) {
        let map_w = self.view.width().max(1);
        let map_h = self.view.height().max(1);
        let cell_cols = area.width as usize;
        let cell_rows = area.height as usize;
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }
        let x_stride = (map_w as f64 / cell_cols as f64).ceil().max(1.0) as usize;
        let y_stride = (map_h as f64 / cell_rows as f64).ceil().max(1.0) as usize;

        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let src_x = (col * x_stride).min(map_w - 1);
                let src_y = (row * y_stride).min(map_h - 1);
                if !self.view.explored(src_x, src_y) {
                    continue;
                }
                let terrain = self.view.tile(src_x, src_y).terrain.as_char();
                let color = if terrain == '~' {
                    Color::Rgb(40, 90, 150)
                } else {
                    Color::Rgb(90, 160, 70)
                };
                if let Some(cell) = buf.cell_mut((area.x + col as u16, area.y + row as u16)) {
                    cell.set_symbol(" ");
                    cell.set_bg(color);
                }
            }
        }
    }

    fn draw_player_stats(&self, area: Rect, buf: &mut Buffer) {
        let year = self.view.year();
        let gold = self.view.gold();
        let civ = self.view.current_player();
        let x = area.x + 1;

        draw_text(
            buf,
            area.right(),
            x,
            area.y,
            &format!("Civilization: {}", civ.display_name()),
            Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        draw_text(
            buf,
            area.right(),
            x,
            area.y + 1,
            &format!("Year: {}", format_year(year)),
            Style::default().fg(Color::Black),
        );
        draw_text(
            buf,
            area.right(),
            x,
            area.y + 2,
            &format!("Gold  {}", gold),
            Style::default().fg(Color::Black),
        );

        let target = self.view.advancement_in_progress();
        let progress = self.view.research_progress();
        let cost = self.view.research_cost();
        let income = self.view.research_income();
        let header_y = area.y + 4;
        let label = if let Some(t) = target {
            format!("Researching: {:?}", t)
        } else {
            "Researching: --".to_string()
        };
        draw_text(
            buf,
            area.right(),
            x,
            header_y,
            &label,
            Style::default().fg(DARK_GREY),
        );
        if let Some(cost) = cost {
            let pct = if cost == 0 {
                0
            } else {
                (progress as f64 / cost as f64 * 10.0) as usize
            };
            let bar: String = "█".repeat(pct) + &"░".repeat(10 - pct);
            draw_text(
                buf,
                area.right(),
                x,
                header_y + 1,
                &bar,
                Style::default().fg(Color::Rgb(80, 160, 255)),
            );
            draw_text(
                buf,
                area.right(),
                x + 22,
                header_y + 1,
                &format!("{progress}/{cost} +{income}"),
                Style::default().fg(DIM),
            );
        }
    }

    fn draw_focus(&self, area: Rect, buf: &mut Buffer) {
        let civ = self.view.current_player();
        let x = area.x + 1;
        draw_text(
            buf,
            area.right(),
            x,
            area.y,
            &format!("In turn: {}", civ.display_name()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        );

        let mut row = area.y + 2;
        match self.focus {
            Some((fx, fy)) => {
                let unit = self.view.units_at(fx, fy).first().copied();
                if let Some(unit) = unit {
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        &format!("Unit: {:?} mv {}", unit.unit_class, unit.moves_remaining()),
                        Style::default().fg(Color::Rgb(230, 200, 120)),
                    );
                    row += 1;
                    if let UnitOrder::Improving(improvement) = unit.order() {
                        draw_text(
                            buf,
                            area.right(),
                            x,
                            row,
                            &format!(
                                "Building {} {} of {} turns",
                                improvement.name().to_lowercase(),
                                unit.work_progress(),
                                improvement.work_turns()
                            ),
                            Style::default().fg(Color::Rgb(150, 220, 160)),
                        );
                        row += 1;
                    }
                } else {
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        "(no unit here)",
                        Style::default().fg(DIM),
                    );
                    row += 1;
                }
                let tile = self.view.tile(fx, fy);
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!("Terrain: {:?}", tile.terrain),
                    Style::default().fg(DIM),
                );
                row += 1;
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!(
                        "Food {}  Prod {}  Trade {}",
                        tile.yields_food(),
                        tile.yields_resources(),
                        tile.yields_trade()
                    ),
                    Style::default().fg(DIM),
                );
                if tile.has_road() || tile.is_mined() || tile.is_irrigated() {
                    row += 1;
                    let improvements = [
                        tile.is_irrigated().then_some("≈ irrigation"),
                        tile.is_mined().then_some("⛏ mine"),
                        tile.has_road().then_some("+ road"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(", ");
                    draw_text(
                        buf,
                        area.right(),
                        x,
                        row,
                        &improvements,
                        Style::default().fg(DIM),
                    );
                }
            }
            None => {
                draw_text(
                    buf,
                    area.right(),
                    x,
                    area.y + 1,
                    "(move a unit to inspect it)",
                    Style::default().fg(DIM),
                );
            }
        }

        // The hovered-tile block: everything the pointer is over, drawn only
        // for tiles the player has already discovered — the fog keeps terrain,
        // improvements and units alike out of sight.
        if let Some((hx, hy)) = self.hovered_tile
            && self.view.explored(hx, hy)
        {
            row += 2;
            let tile = self.view.tile(hx, hy);
            draw_text(
                buf,
                area.right(),
                x,
                row,
                &format!("Hovering: {:?} ({}, {})", tile.terrain, hx, hy),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            );
            row += 1;
            draw_text(
                buf,
                area.right(),
                x,
                row,
                &format!(
                    "Move {}   Food {}  Prod {}  Trade {}",
                    tile.terrain.movement_cost(),
                    tile.yields_food(),
                    tile.yields_resources(),
                    tile.yields_trade()
                ),
                Style::default().fg(DIM),
            );
            row += 1;
            if tile.has_road() || tile.is_mined() || tile.is_irrigated() {
                let improvements = [
                    tile.is_irrigated().then_some("≈ irrigation"),
                    tile.is_mined().then_some("⛏ mine"),
                    tile.has_road().then_some("+ road"),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &improvements,
                    Style::default().fg(Color::Rgb(150, 220, 160)),
                );
                row += 1;
            }
            if let Some(resource) = tile.resource() {
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!("Resource: {:?}", resource),
                    Style::default().fg(DIM),
                );
                row += 1;
            }
            if let Some(city) = self.view.city_at(hx, hy) {
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &format!("City: {} (pop {})", city.name, city.population()),
                    Style::default().fg(DIM),
                );
                row += 1;
            }
            let units = self.view.units_at(hx, hy);
            if units.is_empty() {
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    "No units here",
                    Style::default().fg(DIM),
                );
            } else {
                let listing = units
                    .iter()
                    .map(|unit| {
                        let owner = self.view.civilization_of(unit.owner()).display_name();
                        format!("{:?} ({owner})", unit.unit_class)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                draw_text(
                    buf,
                    area.right(),
                    x,
                    row,
                    &listing,
                    Style::default().fg(Color::Rgb(230, 200, 120)),
                );
            }
        }
    }
}

impl<'a> Widget for GameScreen<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut left = area;
        left.width = LEFT_COLUMN_WIDTH.min(area.width);
        let mut right = area;
        right.x += LEFT_COLUMN_WIDTH;
        right.width = area.width.saturating_sub(LEFT_COLUMN_WIDTH);

        // Fill the backgrounds: the left column is grey, the map pane is dark.
        let left_bg = Color::DarkGray;
        let right_bg = Color::Rgb(12, 12, 40);
        for y in area.y..area.bottom() {
            for x in area.x..left.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(left_bg);
                }
            }
            for x in left.right().max(area.x)..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(right_bg);
                }
            }
        }

        let left_mid = left.height / 3;
        let left_remaining = left.height - left_mid;
        let stats_height = (left_remaining / 2).saturating_sub(5);
        let mini_height = left_mid;
        let focus_height = left_remaining - stats_height;

        let minimap_area = Rect::new(left.x, left.y, left.width, mini_height);
        let stats_area = Rect::new(left.x, left.y + mini_height, left.width, stats_height);
        let focus_area = Rect::new(
            left.x,
            left.y + mini_height + stats_height,
            left.width,
            focus_height,
        );

        // The middle (player stats) panel gets a lighter grey background.
        for y in stats_area.y..stats_area.bottom() {
            for x in stats_area.x..stats_area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(Color::Gray);
                }
            }
        }

        self.draw_minimap(minimap_area, buf);
        self.draw_player_stats(stats_area, buf);
        self.draw_focus(focus_area, buf);
        self.draw_main_map(right, buf);
        self.draw_event_log(right, buf);
    }
}

#[cfg(test)]
pub(crate) use tiles::{CITY_LABEL_FG, tile_style};
pub(crate) use tiles::{
    civilization_color, draw_city_label, fill_row, first_letter, format_year, paint_tile, set_cell,
};

mod tiles;

#[cfg(test)]
mod tests;
