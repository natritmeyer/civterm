use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameView;
use crate::model::units::Unit;
use crate::tui::window_geometry;

const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;
const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

const IDEAL_WIDTH: u16 = 46;
const IDEAL_HEIGHT: u16 = 12;

const DISBAND_ALL_TEXT: &str = "Disband all";
const CANCEL_TEXT: &str = "Cancel";

fn fill_row(buf: &mut Buffer, rect: Rect, style: Style) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            cell.reset();
            cell.set_style(style);
        }
    }
}

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

pub fn dialog_rect(area: Rect) -> Rect {
    window_geometry::centered(area, IDEAL_WIDTH, IDEAL_HEIGHT)
}

pub fn inner(panel: Rect) -> Rect {
    window_geometry::inner(panel)
}

pub fn buttons_y(panel: Rect) -> u16 {
    window_geometry::button_row_y(panel)
}

pub fn rows_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.x + 1,
        y: inner.y + 2,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(4),
    }
}

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

pub fn disband_all_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let width = (DISBAND_ALL_TEXT.len() as u16 + 2).min(inner.width);
    Rect {
        x: inner.x + 1,
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

pub fn cancel_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let width = (CANCEL_TEXT.len() as u16 + 2).min(inner.width);
    Rect {
        x: inner.right().saturating_sub(width + 1),
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

pub struct DisbandDialog<'a> {
    view: &'a dyn GameView,
    ship: crate::model::units::UnitId,
}

impl<'a> DisbandDialog<'a> {
    pub fn new(view: &'a dyn GameView, ship: crate::model::units::UnitId) -> Self {
        DisbandDialog { view, ship }
    }

    fn cargo(&self) -> Vec<&'a Unit> {
        let Some(ship) = self.view.unit(self.ship) else {
            return Vec::new();
        };
        self.view
            .cargo_at(ship.location.x as usize, ship.location.y as usize)
    }
}

impl<'a> Widget for DisbandDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 6 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);
        let inner = inner(area);
        draw_text(
            buf,
            inner.x + 1,
            inner.y,
            inner.right(),
            "Disband ship?",
            BOLD,
        );
        let cargo = self.cargo();
        if cargo.is_empty() {
            draw_text(
                buf,
                inner.x + 1,
                inner.y + 2,
                inner.right(),
                "(no cargo)",
                TEXT,
            );
        } else {
            draw_text(
                buf,
                inner.x + 1,
                inner.y + 1,
                inner.right(),
                "Cargo aboard:",
                TEXT,
            );
            for (index, unit) in cargo.iter().enumerate() {
                if let Some(row) = row_rect(area, index) {
                    if row.y >= inner.bottom().saturating_sub(2) {
                        break;
                    }
                    draw_text(
                        buf,
                        row.x,
                        row.y,
                        row.right(),
                        &format!("{:?}", unit.unit_class),
                        TEXT,
                    );
                }
            }
        }
        let all = disband_all_button_rect(area);
        fill_row(buf, all, Style::new().fg(BLACK).bg(Color::White));
        draw_text(buf, all.x + 1, all.y, all.right(), DISBAND_ALL_TEXT, LINK);
        let cancel = cancel_button_rect(area);
        fill_row(buf, cancel, Style::new().fg(BLACK).bg(Color::White));
        draw_text(
            buf,
            cancel.x + 1,
            cancel.y,
            cancel.right(),
            CANCEL_TEXT,
            LINK,
        );
    }
}
