//! The geometry every floating window shares: sizing and centring it over the
//! terminal, finding its interior, and painting its border.
//!
//! These live in one place because every window used to carry its own copy, and
//! every copy had the same defect. `area` here is the terminal, so it can be
//! *smaller than the window*: a terminal dragged down to a column or two reports
//! a degenerate rect, and a window that is only painted when it fits is still
//! **laid out** when it does not — `App::draw` asks for the rect before the
//! widget gets a chance to bail, and the mouse guards recompute their hit
//! targets from whatever the last frame recorded. A plain `area.width - width`
//! or `panel.width - 2` in that arithmetic overflows and takes the game down
//! mid-decision, so every step here saturates and the result is clamped to stay
//! inside `area`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// Size a floating window to `ideal_width` x `ideal_height`, centred over
/// `area`.
///
/// The window is inset one cell from `area` where it can be, rounded down to an
/// even size (a dialog that came out one cell odd showed a seam down the
/// middle), and clamped to `area` afterwards — the clamp is what makes the
/// result total for an `area` narrower than the window's own minimum.
pub fn centered(area: Rect, ideal_width: u16, ideal_height: u16) -> Rect {
    sized(area, ideal_width, ideal_height, 2, 2)
}

/// As [`centered`], for a window that insists on a floor of its own — the
/// command picker, for instance, is never a sliver two cells wide because its
/// layout is built around a two-column grid.
///
/// The floor is a wish, not a promise: the clamp to `area` still has the last
/// word, so a terminal narrower than the floor gets a smaller window rather than
/// an overflow.
pub fn sized(
    area: Rect,
    ideal_width: u16,
    ideal_height: u16,
    min_width: u16,
    min_height: u16,
) -> Rect {
    let width =
        (ideal_width.min(area.width.saturating_sub(2)).max(min_width) & !1).min(area.width & !1);
    let height = (ideal_height
        .min(area.height.saturating_sub(2))
        .max(min_height)
        & !1)
        .min(area.height & !1);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// The interior of a window, inside its border. Saturating, because the mouse
/// guards work from whatever rect the last frame recorded — including a
/// degenerate one drawn on a terminal too small to paint the window at all.
pub fn inner(panel: Rect) -> Rect {
    Rect {
        x: panel.x.saturating_add(1),
        y: panel.y.saturating_add(1),
        width: panel.width.saturating_sub(2),
        height: panel.height.saturating_sub(2),
    }
}

/// The row a window's buttons sit on: one above the bottom border.
pub fn button_row_y(panel: Rect) -> u16 {
    panel.y.saturating_add(panel.height).saturating_sub(2)
}

/// Paint a window's border and corners in `style`. A window with no room is
/// skipped rather than drawn at a wrapped-around edge.
pub fn draw_border(buf: &mut Buffer, rect: Rect, style: Style) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    let x0 = rect.x;
    let x1 = rect.right() - 1;
    let y0 = rect.y;
    let y1 = rect.bottom() - 1;
    for x in x0..=x1 {
        set_cell(buf, x, y0, "─", style);
        set_cell(buf, x, y1, "─", style);
    }
    for y in y0..=y1 {
        set_cell(buf, x0, y, "│", style);
        set_cell(buf, x1, y, "│", style);
    }
    set_cell(buf, x0, y0, "┌", style);
    set_cell(buf, x1, y0, "┐", style);
    set_cell(buf, x0, y1, "└", style);
    set_cell(buf, x1, y1, "┘", style);
}

/// Write one cell, ignoring coordinates outside the buffer.
pub fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

/// The degenerate sizes a player reaches by dragging a window down, plus the
/// sizes either side of a typical window's own minimum.
#[cfg(test)]
pub(crate) const DEGENERATE_SIZES: [(u16, u16); 14] = [
    (240, 70),
    (200, 60),
    (120, 40),
    (80, 24),
    (72, 20),
    (60, 20),
    (54, 14),
    (53, 13),
    (52, 12),
    (44, 12),
    (40, 10),
    (30, 10),
    (20, 8),
    (12, 6),
];

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn every_helper_is_total_over_any_terminal_size() {
        for (width, height) in
            DEGENERATE_SIZES
                .iter()
                .copied()
                .chain([(10, 5), (5, 3), (2, 1), (1, 1)])
        {
            let area = Rect {
                x: 3,
                y: 2,
                width,
                height,
            };
            let panel = centered(area, 52, 12);
            assert!(
                panel.x >= area.x && panel.y >= area.y,
                "{panel:?} escapes {area:?}"
            );
            assert!(
                panel.right() <= area.right() && panel.bottom() <= area.bottom(),
                "{panel:?} escapes {area:?}"
            );
            // The rest is what the mouse guards recompute from the last frame's
            // rect, on a panel too small to have been painted.
            let _ = inner(panel);
            let _ = button_row_y(panel);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| draw_border(frame.buffer_mut(), panel, Style::default()))
                .unwrap();
        }
    }

    #[test]
    fn a_full_size_window_is_inset_one_cell_and_centred() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 40,
        };
        let panel = centered(area, 52, 12);
        assert_eq!(
            panel,
            Rect {
                x: 34,
                y: 14,
                width: 52,
                height: 12
            }
        );
    }

    #[test]
    fn a_window_is_never_odd_sized() {
        for (width, height) in DEGENERATE_SIZES {
            let panel = centered(
                Rect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                52,
                12,
            );
            assert_eq!(panel.width % 2, 0, "width is even at {width}x{height}");
            assert_eq!(panel.height % 2, 0, "height is even at {width}x{height}");
        }
    }
}

/// Every floating window's geometry is laid out again by its mouse guard, from
/// whatever the last frame recorded — including a frame drawn on a terminal too
/// small to paint the window. This walks each window's own hit-test helpers at
/// every degenerate size, so one window's arithmetic cannot reintroduce the
/// overflow the shared helpers were written to remove.
#[cfg(test)]
mod every_window {
    use super::*;
    use crate::tui::{
        build_complete_dialog, city_window, command_picker, diplomacy_dialog,
        diplomat_actions_dialog, production_picker, quit_dialog, research_dialog, sabotage_dialog,
        save_load_prompt, starvation_dialog, steal_dialog, war_dialog,
    };

    /// Every terminal a player can produce by dragging a window down, and then
    /// some: the sizes either side of each window's own minimum, plus the
    /// degenerate ones.
    const SIZES: [(u16, u16); 20] = [
        (240, 70),
        (200, 60),
        (120, 40),
        (80, 24),
        (72, 20),
        (60, 20),
        (54, 14),
        (53, 13),
        (52, 12),
        (44, 12),
        (40, 10),
        (30, 10),
        (20, 8),
        (16, 8),
        (12, 6),
        (10, 5),
        (5, 3),
        (3, 2),
        (2, 1),
        (1, 1),
    ];

    #[test]
    fn no_window_panics_when_hit_tested_at_any_size() {
        for (width, height) in SIZES {
            let area = Rect {
                x: 5,
                y: 3,
                width,
                height,
            };
            // A dialog's own rect, and then the same rect as a frame recorded
            // one: a degenerate one is what a too-small terminal leaves behind.
            for panel in [
                save_load_prompt::dialog_rect(area),
                research_dialog::dialog_rect(area),
                quit_dialog::dialog_rect(area),
                war_dialog::dialog_rect(area),
                diplomacy_dialog::dialog_rect(area),
                build_complete_dialog::dialog_rect(area),
                diplomat_actions_dialog::dialog_rect(area),
                steal_dialog::dialog_rect(area),
                sabotage_dialog::dialog_rect(area),
                starvation_dialog::dialog_rect(area),
                command_picker::command_picker_rect(area),
                city_window::window_rect(area),
                production_picker::picker_rect(city_window::window_rect(area)),
            ] {
                let _ = research_dialog::ok_button_rect(panel);
                let _ = quit_dialog::quit_button_rect(panel);
                let _ = quit_dialog::save_button_rect(panel);
                let _ = quit_dialog::continue_button_rect(panel);
                let _ = war_dialog::ok_button_rect(panel);
                let _ = diplomacy_dialog::peace_button_rect(panel);
                let _ = diplomacy_dialog::war_button_rect(panel);
                let _ = build_complete_dialog::ok_button_rect(panel);
                let _ = build_complete_dialog::next_order_button_rect(panel);
                let _ = diplomat_actions_dialog::ok_button_rect(panel);
                let _ = diplomat_actions_dialog::reason_rect(panel);
                for index in 0..8 {
                    let _ = diplomat_actions_dialog::row_rect(panel, index);
                }
                let _ = steal_dialog::ok_button_rect(panel);
                let _ = sabotage_dialog::ok_button_rect(panel);
                let _ = starvation_dialog::ok_button_rect(panel);
                let _ = command_picker::rows_rect(panel);
                let _ = command_picker::cancel_button_rect(panel);
                let _ = command_picker::save_button_rect(panel);
                let _ = production_picker::rows_rect(panel);
                let _ = production_picker::cancel_button_rect(panel);
                let _ = production_picker::save_button_rect(panel);
                let _ = city_window::close_button_rect(panel);
                let _ = city_window::production_panel_rect(panel);
                let _ = city_window::bottom_panels(panel);
                let _ = city_window::change_button_rect(city_window::production_panel_rect(panel));
            }
        }
    }
}
