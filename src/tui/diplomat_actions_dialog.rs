use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::{DiplomatAction, DiplomatOption};
use crate::tui::window_geometry;

const VANILLA_BG: Color = Color::Rgb(216, 182, 78);
const BLACK: Color = Color::Rgb(0, 0, 0);
const TEXT: Style = Style::new().fg(BLACK).bg(VANILLA_BG);
const BOLD: Style = TEXT.add_modifier(Modifier::BOLD);
const LINK: Style = TEXT.add_modifier(Modifier::UNDERLINED);
/// A row the player cannot take is shown, but dimmed: the point of the window
/// is to show what a diplomat *could* do here, and a row that is simply absent
/// teaches nothing about why it is absent.
const DIMMED: Style = TEXT.add_modifier(Modifier::DIM);
const SELECTED: Style = Style::new().fg(BLACK).bg(Color::White);

const IDEAL_WIDTH: u16 = 52;
const IDEAL_HEIGHT: u16 = 12;
const OK_TEXT: &str = "[ OK ]";
/// The button's white body: the label with one square of white on each side of
/// it, so both ends of the pill look the same. The label is drawn one cell in
/// from the left edge of the rect, which is what makes the pad a pad — at the
/// old label width the label ran one cell past the fill, and the `]` sat on
/// bare terrain with no white square to match the one on its left.
const OK_WIDTH: u16 = OK_TEXT.len() as u16 + 2;
/// Rows spent above the list: a blank line under the heading.
const HEADER_ROWS: u16 = 2;

/// The window a player's diplomat is given when he walks into a rival city: the
/// five things he came for, what each costs, and an OK button. Choosing a row
/// and confirming it is the whole interaction — there is no separate "apply",
/// because the action is not a setting but a thing that happens.
pub struct DiplomatActionsDialog<'a> {
    city_name: &'a str,
    options: &'a [DiplomatOption],
    cursor: usize,
}

impl<'a> DiplomatActionsDialog<'a> {
    pub fn new(city_name: &'a str, options: &'a [DiplomatOption], cursor: usize) -> Self {
        DiplomatActionsDialog {
            city_name,
            options,
            cursor,
        }
    }
}

/// The rectangle the dialog occupies over `area`, centred across it.
///
/// Every step saturates and the result is clamped inside `area`, because
/// `area` is the terminal: a window the player drags down to a column or two
/// reports a degenerate area, and `App::draw` asks for this rect on every
/// frame the dialog is up — including the frames drawn while it is too small to
/// paint. A plain `area.width - width` overflows there and takes the game down
/// with the window still on screen.
pub fn dialog_rect(area: Rect) -> Rect {
    let width = (IDEAL_WIDTH.min(area.width.saturating_sub(2)).max(2) & !1).min(area.width & !1);
    let height =
        (IDEAL_HEIGHT.min(area.height.saturating_sub(2)).max(2) & !1).min(area.height & !1);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// The interior of the dialog, inside its border. Saturating, because the
/// mouse handler works from whatever rect the last frame recorded — including a
/// degenerate one drawn on a terminal too small to paint the dialog.
fn inner(panel: Rect) -> Rect {
    window_geometry::inner(panel)
}

/// The y row the button sits on (one above the bottom border).
fn buttons_y(panel: Rect) -> u16 {
    window_geometry::button_row_y(panel)
}

/// The rectangle of the OK button, right-aligned on the button row so its white
/// body ends at the interior's last cell — the label therefore sits one cell
/// left of the border, with its matching white square after it — never left of
/// the panel's own edge.
pub fn ok_button_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    Rect {
        x: inner.right().saturating_sub(OK_WIDTH).max(panel.x),
        y: buttons_y(panel),
        width: OK_WIDTH,
        height: 1,
    }
}

/// The rows region holding the five actions, one to a line.
pub fn list_rect(panel: Rect) -> Rect {
    let inner = inner(panel);
    let top = inner.y + HEADER_ROWS;
    // The list stops two rows short of the button row: the row beneath it is
    // the refusal line, and the one after that is blank air.
    let height = buttons_y(panel)
        .saturating_sub(2)
        .saturating_sub(top)
        .min(DiplomatAction::ALL.len() as u16);
    Rect {
        x: inner.x + 1,
        y: top,
        width: inner.width.saturating_sub(2),
        height,
    }
}

/// The rectangle of the action on list row `index`, or `None` when that row is
/// off the bottom of the window. A click inside it selects that action.
pub fn row_rect(panel: Rect, index: usize) -> Option<Rect> {
    let rows = list_rect(panel);
    if index as u16 >= rows.height {
        return None;
    }
    Some(Rect {
        x: rows.x,
        y: rows.y + index as u16,
        width: rows.width,
        height: 1,
    })
}

/// The reason the row under the cursor cannot be taken, drawn on the line
/// under the list. Showing it here rather than on the row itself is what lets
/// the refusals be sentences: a row is too narrow to hold "You need units of
/// your own in the city" beside a price.
pub fn reason_rect(panel: Rect) -> Rect {
    let rows = list_rect(panel);
    Rect {
        x: rows.x,
        y: rows.bottom(),
        width: rows.width,
        height: 1,
    }
}

impl Widget for DiplomatActionsDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 8 {
            return;
        }
        fill_rect(buf, inner(area));
        draw_border(buf, area);

        let inner = inner(area);
        draw_text(
            buf,
            inner.x,
            inner.y,
            &format!("Inside {}", self.city_name),
            BOLD,
        );

        let rows = list_rect(area);
        for index in 0..rows.height as usize {
            let row = rows.y + index as u16;
            let Some(option) = self.options.get(index) else {
                break;
            };
            let selected = index == self.cursor;
            if selected {
                fill_row(
                    buf,
                    Rect {
                        x: rows.x,
                        y: row,
                        width: rows.width,
                        height: 1,
                    },
                    SELECTED,
                );
            }
            // A blocked row is dimmed even when it is the one on the cursor:
            // the highlight says "this is the row you would choose", and the
            // dimming says "and you cannot". The dimming is added to the
            // highlight rather than replacing it, or the row's own background
            // would come back and the two messages would cancel out.
            let blocked = option.blocked.is_some();
            let style = match (selected, blocked) {
                (true, true) => SELECTED.add_modifier(Modifier::DIM),
                (_, true) => DIMMED,
                (true, false) => SELECTED,
                (false, false) => TEXT,
            };
            let label = format!("{:<22}{:>5}g", option.action.label(), option.cost);
            draw_text(buf, rows.x + 1, row, &label, style);
        }

        if let Some(blocked) = self
            .options
            .get(self.cursor)
            .and_then(|o| o.blocked.as_deref())
        {
            let reason = reason_rect(area);
            if reason.y < buttons_y(area) {
                draw_text(buf, reason.x + 1, reason.y, blocked, DIMMED);
            }
        }

        let ok = ok_button_rect(area);
        fill_row(buf, ok, Style::new().fg(BLACK).bg(Color::White));
        draw_text(buf, ok.x + 1, ok.y, OK_TEXT, LINK);
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

fn cell_width(ch: char) -> u16 {
    if ch == '\u{1F4A5}' { 2 } else { 1 }
}

fn draw_text(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    let mut cx = x;
    for ch in text.chars() {
        let width = cell_width(ch);
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
            if width == 2
                && let Some(next) = buf.cell_mut((cx.saturating_add(1), y))
            {
                next.set_diff_option(ratatui::buffer::CellDiffOption::Skip);
            }
        }
        cx = cx.saturating_add(width);
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

fn draw_border(buf: &mut Buffer, rect: Rect) {
    window_geometry::draw_border(buf, rect, TEXT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_engine::DiplomatAction;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn options() -> Vec<DiplomatOption> {
        DiplomatAction::ALL
            .into_iter()
            .map(|action| DiplomatOption {
                action,
                cost: action.cost(None),
                blocked: None,
            })
            .collect()
    }

    fn render(options: &[DiplomatOption], cursor: usize) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    DiplomatActionsDialog::new("Umgungundlovu", options, cursor),
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
        let rect = dialog_rect(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        });
        assert_eq!(rect.width, IDEAL_WIDTH);
        assert_eq!(rect.height, IDEAL_HEIGHT);
        assert_eq!(rect.x, (100 - rect.width) / 2);
        assert_eq!(rect.y, (40 - rect.height) / 2);
    }

    #[test]
    fn the_window_names_the_city_the_diplomat_is_inside() {
        let buf = render(&options(), 0);
        let panel = panel();
        assert!(row_text(&buf, panel.x + 2, panel.y + 1, 22).contains("Umgungundlovu"));
    }

    /// Every action gets a line, and the line carries its price — the player is
    /// choosing between five things that cost different amounts.
    #[test]
    fn every_action_is_listed_with_its_price() {
        let buf = render(&options(), 0);
        let rows = list_rect(panel());
        for (index, option) in options().iter().enumerate() {
            let text = row_text(&buf, rows.x + 1, rows.y + index as u16, rows.width - 1);
            assert!(
                text.contains(option.action.label()),
                "row {index} should name {:?}, drew {text:?}",
                option.action
            );
            assert!(
                text.contains(&format!("{}g", option.cost)),
                "row {index} should show {} gold, drew {text:?}",
                option.cost
            );
        }
    }

    #[test]
    fn the_selected_row_is_highlighted() {
        let buf = render(&options(), 2);
        let rows = list_rect(panel());
        assert_eq!(
            buf.cell((rows.x + 1, rows.y + 2)).unwrap().bg,
            Color::White,
            "the cursor's row is lit"
        );
        assert_ne!(
            buf.cell((rows.x + 1, rows.y)).unwrap().bg,
            Color::White,
            "and the others are not"
        );
    }

    /// A row the player cannot take is still drawn, and the reason why is spelled
    /// out under the list for the row on the cursor.
    #[test]
    fn a_blocked_row_is_shown_with_its_reason() {
        let mut blocked = options();
        blocked[3].blocked = Some("The city holds no units to incite".to_string());
        let buf = render(&blocked, 3);
        let rows = list_rect(panel());
        assert!(
            row_text(&buf, rows.x + 1, rows.y + 3, rows.width - 1).contains("Incite a Revolt"),
            "the row is still listed"
        );
        let reason = reason_rect(panel());
        assert!(
            row_text(&buf, reason.x + 1, reason.y, rows.width - 1).contains("no units to incite"),
            "and the refusal is spelled out for the row on the cursor"
        );
    }

    #[test]
    fn the_reason_line_follows_the_cursor() {
        let mut blocked = options();
        blocked[3].blocked = Some("the third one".to_string());
        blocked[1].blocked = Some("the second one".to_string());
        let buf = render(&blocked, 1);
        let reason = reason_rect(panel());
        let text = row_text(&buf, reason.x + 1, reason.y, list_rect(panel()).width - 1);
        assert!(text.contains("the second one"), "drew {text:?}");
        assert!(!text.contains("the third one"));
    }

    #[test]
    fn the_ok_button_is_drawn_and_highlighted() {
        let buf = render(&options(), 0);
        let panel = panel();
        let ok = ok_button_rect(panel);
        assert_eq!(
            row_text(&buf, ok.x + 1, ok.y, OK_TEXT.len() as u16),
            OK_TEXT
        );
        assert_eq!(buf.cell((ok.x, ok.y)).unwrap().bg, Color::White);
        // The label is drawn with the ordinary vanilla-backed link style, so
        // the white squares that make it a button are the cells on either side
        // of it: one in front of the `[`, one after the `]`. The old fill was
        // the label's width, so the square after the label was never painted
        // and the button had no matching right-hand end.
        assert_eq!(
            buf.cell((ok.x + OK_WIDTH - 1, ok.y)).unwrap().bg,
            Color::White,
            "the square after the label matches the one before it"
        );
        // The label is an underlined link, like every other button in the game:
        // the underline is what tells the player this is the thing to press.
        let label = buf.cell((ok.x + 1, ok.y)).unwrap();
        assert!(
            label.modifier.contains(Modifier::UNDERLINED),
            "the OK label is underlined"
        );
        // The button's geometry is also where the mouse looks for it, so a
        // button placed outside its own window would be both unpainted and
        // unclickable — it has to lie inside the panel it is drawn into.
        assert!(
            ok.x >= panel.x
                && ok.right() <= panel.right()
                && ok.y >= panel.y
                && ok.bottom() <= panel.bottom(),
            "the button {:?} lies inside the window {:?}",
            ok,
            panel
        );
    }

    /// A row the player cannot take is dimmed, live and blocked alike under the
    /// cursor: the highlight says which row would be chosen, the dimming says
    /// that it cannot be.
    #[test]
    fn a_blocked_row_is_dimmed_even_when_it_is_on_the_cursor() {
        let mut blocked = options();
        blocked[0].blocked = Some("You need 25 gold".to_string());
        blocked[2].blocked = Some("Nothing to destroy".to_string());
        let buf = render(&blocked, 0);
        let rows = list_rect(panel());
        assert!(
            buf.cell((rows.x + 1, rows.y + 2))
                .unwrap()
                .modifier
                .contains(Modifier::DIM),
            "a blocked row off the cursor is dimmed too"
        );
        let blocked_cell = buf.cell((rows.x + 1, rows.y)).unwrap();
        assert!(
            blocked_cell.modifier.contains(Modifier::DIM),
            "the blocked row under the cursor is dimmed"
        );
        assert_eq!(
            blocked_cell.bg,
            Color::White,
            "and still lit, because the cursor is on it"
        );
        let live_cell = buf.cell((rows.x + 1, rows.y + 1)).unwrap();
        assert!(
            !live_cell.modifier.contains(Modifier::DIM),
            "a row the player can take is not dimmed"
        );
    }

    /// Every row is clickable, and the geometry agrees with what was painted:
    /// a click is only going to be matched against these rectangles.
    #[test]
    fn every_row_has_a_click_target_that_was_painted() {
        let buf = render(&options(), 0);
        let panel = panel();
        for index in 0..options().len() {
            let row = row_rect(panel, index).expect("each action has a row");
            assert!(
                row_text(&buf, row.x + 1, row.y, row.width - 1)
                    .contains(options()[index].action.label()),
                "row {index} is drawn where it is clickable"
            );
        }
        assert!(
            row_rect(panel, options().len()).is_none(),
            "and there is no row below the list"
        );
    }
}

/// The widget itself must be safe on a terminal too small to paint it, since
/// `App::draw` still hands it a rectangle on every frame the window is up. The
/// shared geometry's own degenerate-size test covers the hit-testing this file
/// delegates; this covers the painting.
#[cfg(test)]
mod degenerate_terminals {
    use super::*;
    use crate::game_engine::{DiplomatAction, DiplomatOption};
    use crate::tui::window_geometry::DEGENERATE_SIZES;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn drawing_at_any_size_never_panics() {
        let options: Vec<DiplomatOption> = DiplomatAction::ALL
            .into_iter()
            .map(|action| DiplomatOption {
                action,
                cost: action.cost(None),
                blocked: None,
            })
            .collect();
        for (width, height) in
            DEGENERATE_SIZES
                .iter()
                .copied()
                .chain([(10, 5), (5, 3), (2, 1), (1, 1)])
        {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| {
                    frame.render_widget(
                        DiplomatActionsDialog::new("Umgungundlovu", &options, 0),
                        dialog_rect(frame.area()),
                    )
                })
                .unwrap();
        }
    }
}
