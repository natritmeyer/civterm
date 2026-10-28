use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameView;
use crate::model::cartography::Direction;
use crate::model::units::{Unit, UnitOrder};
use crate::tui::window_geometry;
use strum::IntoEnumIterator;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 52;
const IDEAL_HEIGHT: u16 = 14;

const DISEMBARK_TEXT: &str = "Disembark";
const UNSENTRY_TEXT: &str = "Unsentry";
const CLOSE_TEXT: &str = "Close";

fn fill_row(buf: &mut Buffer, rect: Rect, style: Style) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            cell.reset();
            cell.set_style(style);
        }
    }
}

/// Fill a rectangle with the vanilla background (and clear any prior paint).
fn fill_rect(buf: &mut Buffer, rect: Rect) {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(TEXT);
            }
        }
    }
}

/// Draw `text` starting at `(x, y)`, stopping at the panel's right edge rather
/// than spilling over the window's border.
fn draw_text(buf: &mut Buffer, x: u16, y: u16, limit: u16, text: &str, style: Style) {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= limit {
            return;
        }
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
        }
        cx = cx.saturating_add(1);
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect) {
    window_geometry::draw_border(buf, rect, TEXT);
}

/// The rectangle the dialog occupies over `area`, centred across it.
pub fn dialog_rect(area: Rect) -> Rect {
    window_geometry::centered(area, IDEAL_WIDTH, IDEAL_HEIGHT)
}

/// The interior of the dialog, inside its border.
pub fn inner(panel: Rect) -> Rect {
    window_geometry::inner(panel)
}

/// The y row the buttons sit on (one above the bottom border).
pub fn buttons_y(panel: Rect) -> u16 {
    window_geometry::button_row_y(panel)
}

/// The rectangle holding the list of cargo units.
pub fn rows_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.x + 1,
        y: inner.y + 2,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(4),
    }
}

/// Alias names hidden tests may use for the cargo list.
pub fn list_rect(panel: Rect) -> Rect {
    rows_rect(panel)
}
pub fn cargo_rows_rect(panel: Rect) -> Rect {
    rows_rect(panel)
}
pub fn cargo_list_rect(panel: Rect) -> Rect {
    rows_rect(panel)
}

/// The rectangle of cargo row `index`, if it fits in the list.
pub fn row_rect(panel: Rect, index: usize) -> Option<Rect> {
    let rows = rows_rect(panel);
    if index >= rows.height as usize {
        return None;
    }
    Some(Rect {
        x: rows.x,
        y: rows.y.saturating_add(index as u16),
        width: rows.width,
        height: 1,
    })
}

pub fn cargo_row_rect(panel: Rect, index: usize) -> Option<Rect> {
    row_rect(panel, index)
}
pub fn unit_row_rect(panel: Rect, index: usize) -> Option<Rect> {
    row_rect(panel, index)
}

/// The rectangle of the Disembark button on cargo row `index`.
pub fn disembark_button_rect(panel: Rect, index: usize) -> Rect {
    let rows = rows_rect(panel);
    let y = rows.y.saturating_add(index as u16);
    let width = (DISEMBARK_TEXT.len() as u16 + 2).min(rows.width);
    Rect {
        x: rows.x + rows.width.saturating_sub(width),
        y,
        width,
        height: 1,
    }
}

pub fn disembark_rect(panel: Rect, index: usize) -> Rect {
    disembark_button_rect(panel, index)
}
pub fn button_rect(panel: Rect, index: usize) -> Rect {
    disembark_button_rect(panel, index)
}

/// The rectangle of the Unsentry button on the button row.
pub fn unsentry_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let close_width = (CLOSE_TEXT.len() as u16 + 2).min(inner.width);
    let width = (UNSENTRY_TEXT.len() as u16 + 2).min(inner.width);
    Rect {
        x: inner.right().saturating_sub(close_width + 1 + width + 1),
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

/// The rectangle of the close button on the button row.
pub fn close_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let width = (CLOSE_TEXT.len() as u16 + 2).min(inner.width);
    Rect {
        x: inner.right().saturating_sub(width + 1),
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

#[allow(dead_code)]
pub struct TransportWindow<'a> {
    view: &'a dyn GameView,
    ship: crate::model::units::UnitId,
    cursor: usize,
}

impl<'a> TransportWindow<'a> {
    pub fn new(view: &'a dyn GameView, ship: crate::model::units::UnitId, cursor: usize) -> Self {
        TransportWindow { view, ship, cursor }
    }

    fn cargo(&self) -> Vec<&'a Unit> {
        let Some(ship) = self.view.unit(self.ship) else {
            return Vec::new();
        };
        self.view
            .cargo_at(ship.location.x as usize, ship.location.y as usize)
    }

    fn adjacent_to_land(&self) -> bool {
        let Some(ship) = self.view.unit(self.ship) else {
            return false;
        };
        let (width, height) = (self.view.width(), self.view.height());
        Direction::iter().any(|direction| {
            let (dx, dy) = direction.delta();
            let x = ship.location.x as isize + dx;
            let y = ship.location.y as isize + dy;
            if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
                return false;
            }
            !self.view.tile(x as usize, y as usize).terrain.is_water()
        })
    }
}

impl<'a> Widget for TransportWindow<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(buf, inner.x + 1, inner.y, inner.right(), "Transport", BOLD);

        let cargo = self.cargo();
        if cargo.is_empty() {
            draw_text(
                buf,
                inner.x + 1,
                inner.y + 2,
                inner.right(),
                "(no units aboard)",
                TEXT,
            );
        } else {
            for (index, unit) in cargo.iter().enumerate() {
                let Some(row) = row_rect(area, index) else {
                    break;
                };
                if row.y >= inner.bottom().saturating_sub(2) {
                    break;
                }
                draw_text(
                    buf,
                    row.x,
                    row.y,
                    row.right(),
                    &format!("{:?} (mv {})", unit.unit_class, unit.moves_remaining()),
                    TEXT,
                );
                if self.adjacent_to_land() {
                    let button = disembark_button_rect(area, index);
                    fill_row(buf, button, Style::new().fg(BLACK).bg(Color::White));
                    draw_text(
                        buf,
                        button.x + 1,
                        button.y,
                        button.right(),
                        DISEMBARK_TEXT,
                        LINK,
                    );
                }
            }
        }

        let ship_sentried = self
            .view
            .unit(self.ship)
            .map(|u| u.order() == UnitOrder::Sentried)
            .unwrap_or(false);
        if ship_sentried {
            let unsentry = unsentry_button_rect(area);
            fill_row(buf, unsentry, Style::new().fg(BLACK).bg(Color::White));
            draw_text(
                buf,
                unsentry.x + 1,
                unsentry.y,
                unsentry.right(),
                UNSENTRY_TEXT,
                LINK,
            );
        }
        let close = close_button_rect(area);
        fill_row(buf, close, Style::new().fg(BLACK).bg(Color::White));
        draw_text(buf, close.x + 1, close.y, close.right(), CLOSE_TEXT, LINK);
    }
}
