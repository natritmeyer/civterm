use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::model::cities::ProductionTarget;

/// The vanilla-yellow backdrop shared with the other civ-style windows.
const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Black;

const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);

/// The dialog's ideal dimensions before clamping to the screen.
const IDEAL_WIDTH: u16 = 46;
const IDEAL_HEIGHT: u16 = 11;

const OK_TEXT: &str = "OK";
const NEXT_ORDER_TEXT: &str = "Next Order";

/// A modal window reporting that one of the player's cities finished building
/// a unit or an improvement. OK moves on to the next city that finished;
/// "Next Order" opens that city's window instead, so the player can set what it
/// builds next without hunting for it on the map.
pub struct BuildCompleteDialog {
    city_name: String,
    target: ProductionTarget,
}

impl BuildCompleteDialog {
    pub fn new(city_name: String, target: ProductionTarget) -> Self {
        BuildCompleteDialog { city_name, target }
    }

    /// The finished target as a player-facing noun: improvements carry a
    /// display name of their own, while a unit class reads well as-is.
    fn target_label(&self) -> String {
        match self.target {
            ProductionTarget::Unit(unit_class) => format!("{unit_class:?}"),
            ProductionTarget::Improvement(improvement) => improvement.name().to_string(),
        }
    }
}

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

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

fn draw_border(buf: &mut Buffer, rect: Rect) {
    let x0 = rect.x;
    let x1 = rect.right() - 1;
    let y0 = rect.y;
    let y1 = rect.bottom() - 1;
    for x in x0..=x1 {
        set_cell(buf, x, y0, "─", TEXT);
        set_cell(buf, x, y1, "─", TEXT);
    }
    for y in y0..=y1 {
        set_cell(buf, x0, y, "│", TEXT);
        set_cell(buf, x1, y, "│", TEXT);
    }
    set_cell(buf, x0, y0, "┌", TEXT);
    set_cell(buf, x1, y0, "┐", TEXT);
    set_cell(buf, x0, y1, "└", TEXT);
    set_cell(buf, x1, y1, "┘", TEXT);
}

/// The rectangle the dialog occupies over `area`, centred across it.
pub fn dialog_rect(area: Rect) -> Rect {
    let width = (IDEAL_WIDTH.min(area.width.saturating_sub(2)).max(2)) & !1;
    let height = (IDEAL_HEIGHT.min(area.height.saturating_sub(2)).max(2)) & !1;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// The interior of the dialog, inside its border.
fn inner(panel: Rect) -> Rect {
    Rect {
        x: panel.x + 1,
        y: panel.y + 1,
        width: panel.width - 2,
        height: panel.height - 2,
    }
}

/// The y row the buttons sit on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    panel.y + panel.height - 2
}

/// A button's rectangle: `width` cells wide, starting at `x` on the button row.
fn button_rect(panel: Rect, x: u16, width: u16) -> Rect {
    Rect {
        x,
        y: buttons_y(panel),
        width,
        height: 1,
    }
}

/// The rectangle of the OK button, left-aligned on the button row.
pub fn ok_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    button_rect(
        panel,
        inner.x + 1,
        (OK_TEXT.len() as u16 + 2).min(inner.width),
    )
}

/// The rectangle of the Next Order button, right-aligned on the button row.
pub fn next_order_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let width = (NEXT_ORDER_TEXT.len() as u16 + 2).min(inner.width);
    button_rect(panel, inner.right().saturating_sub(width + 1), width)
}

impl Widget for BuildCompleteDialog {
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
            "Build complete",
            BOLD,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 2,
            inner.right(),
            &format!("{} finished building", self.city_name),
            TEXT,
        );
        draw_text(
            buf,
            inner.x + 1,
            inner.y + 3,
            inner.right(),
            &format!("a {}", self.target_label()),
            BOLD,
        );
        // Both buttons are live at once, so neither is drawn as the focused
        // choice the way the quit dialog highlights its cursor.
        for (rect, label) in [
            (ok_button_rect(area), OK_TEXT),
            (next_order_button_rect(area), NEXT_ORDER_TEXT),
        ] {
            fill_row(buf, rect, Style::new().fg(BLACK).bg(Color::White));
            draw_text(buf, rect.x + 1, rect.y, rect.right(), label, LINK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cities::CityImprovement;
    use crate::model::units::UnitClass;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(city_name: &str, target: ProductionTarget) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    BuildCompleteDialog::new(city_name.to_string(), target),
                    panel(),
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn panel() -> Rect {
        dialog_rect(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        })
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn dialog_centres_the_window_in_the_area() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };
        let panel = dialog_rect(area);
        assert_eq!(panel.width, IDEAL_WIDTH);
        // `dialog_rect` rounds both dimensions down to an even number, so the
        // centring is derived from the clamped rect rather than the ideal one.
        assert_eq!(panel.height, IDEAL_HEIGHT & !1);
        assert_eq!(panel.x, area.x + (area.width - panel.width) / 2);
        assert_eq!(panel.y, area.y + (area.height - panel.height) / 2);
    }

    #[test]
    fn the_dialog_names_the_city_and_the_finished_unit() {
        let buf = render("Ravenna", ProductionTarget::Unit(UnitClass::Legion));
        let panel = panel();
        let body = (0..panel.height)
            .map(|i| row_text(&buf, panel.x + 1, panel.y + i, panel.width - 2))
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(body.contains("Ravenna"), "city: {body}");
        assert!(body.contains("Legion"), "unit: {body}");
    }

    #[test]
    fn an_improvement_is_named_by_its_display_name() {
        let buf = render(
            "Ravenna",
            ProductionTarget::Improvement(CityImprovement::CityWalls),
        );
        let panel = panel();
        let body = (0..panel.height)
            .map(|i| row_text(&buf, panel.x + 1, panel.y + i, panel.width - 2))
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(body.contains("City Walls"), "improvement: {body}");
    }

    #[test]
    fn both_buttons_are_drawn_and_highlighted() {
        let buf = render("Ravenna", ProductionTarget::Unit(UnitClass::Legion));
        let panel = panel();
        for (rect, label) in [
            (ok_button_rect(panel), OK_TEXT),
            (next_order_button_rect(panel), NEXT_ORDER_TEXT),
        ] {
            assert_eq!(
                row_text(&buf, rect.x + 1, rect.y, label.len() as u16),
                label,
                "{label} is drawn"
            );
            assert_eq!(
                buf.cell((rect.x, rect.y)).unwrap().bg,
                Color::White,
                "{label} is highlighted"
            );
        }
    }

    /// The two buttons must not overlap: a click aimed at one would otherwise
    /// be swallowed by the other, depending on hit-test order.
    #[test]
    fn the_two_buttons_do_not_overlap() {
        let panel = panel();
        assert!(
            !ok_button_rect(panel).intersects(next_order_button_rect(panel)),
            "OK at {:?} overlaps Next Order at {:?}",
            ok_button_rect(panel),
            next_order_button_rect(panel)
        );
    }
}
