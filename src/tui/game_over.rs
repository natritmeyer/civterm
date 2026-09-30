use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::game_engine::GameOutcome;

/// The colour of the stone tablet's fill and the raised lettering on it.
const STONE_FILL: Color = Color::Rgb(104, 108, 118);
const STONE_SHADE: Color = Color::Rgb(64, 66, 74);
const STONE_INK: Color = Color::Rgb(240, 240, 246);
/// The muted engraving tone of the tablet's "R I P" inscription.
const INSCRIPTION: Color = Color::Rgb(150, 152, 160);
/// The night sky behind the victory banner, and its celebratory palette.
const NIGHT: Color = Color::Rgb(24, 18, 46);
const BANNER_FILL: Color = Color::Rgb(44, 34, 80);
const BANNER_BORDER: Color = Color::Rgb(150, 122, 44);
const PALE: Color = Color::Rgb(236, 236, 242);
const GOLD: Color = Color::Rgb(242, 210, 96);
/// The dim backdrop shared by both end screens.
const SLATE: Color = Color::Rgb(20, 20, 26);
/// The resting tone of the unselected button label on its backdrop.
const MUTED: Color = Color::Rgb(196, 196, 206);
const BLACK: Color = Color::Black;
const SELECTED: Color = Color::White;

/// The tablet/banner keeps a fixed width so the button row underneath can be
/// placed identically whichever civ name graces the headline.
const PANEL_WIDTH: u16 = 68;
/// The tablet/banner occupies seven rows: border, inscription, blank, the
/// (wrapped) headline, blank, border.
const PANEL_HEIGHT: u16 = 7;
/// Rows between the panel's bottom border and the button row.
const PANEL_BUTTON_GAP: u16 = 2;
/// How much of the panel's height is shared with the buttons below it, when
/// the whole block is centred on the screen.
const BLOCK_HEIGHT: u16 = PANEL_HEIGHT + PANEL_BUTTON_GAP + 1;

const RIP: &str = "R I P";
const VICTORY_HEADER: &str = "✦  VICTORY  ✦";

/// The end-of-match overlay: a full-screen backdrop with the tombstone of the
/// fallen civilization (defeat) or a celebratory banner (victory), and the
/// two side-by-side buttons beneath it.
pub struct GameOver {
    outcome: GameOutcome,
    civ_name: String,
    first_selected: bool,
}

impl GameOver {
    pub fn new(outcome: GameOutcome, civ_name: &str, first_selected: bool) -> Self {
        GameOver {
            outcome,
            civ_name: civ_name.to_string(),
            first_selected,
        }
    }
}

/// The pair of button labels for an outcome: the first button continues the
/// story (continue playing, or start again), the second always quits.
pub fn button_labels(outcome: GameOutcome) -> (&'static str, &'static str) {
    match outcome {
        GameOutcome::Victory => ("Continue playing", "Quit"),
        GameOutcome::Defeat => ("Start again", "Quit"),
    }
}

/// The y row the two buttons sit on, centred beneath the panel with it.
pub fn buttons_row(area: Rect) -> u16 {
    area.y + area.height.saturating_sub(BLOCK_HEIGHT) / 2 + PANEL_HEIGHT + PANEL_BUTTON_GAP
}

/// The rectangles of the two buttons, side by side beneath the panel and
/// centred across the screen. The app hit-tests these rectangles with the
/// mouse exactly as the widget paints them.
pub fn button_rects(area: Rect, outcome: GameOutcome) -> (Rect, Rect) {
    let (first_label, second_label) = button_labels(outcome);
    let first_width = first_label.len() as u16 + 2;
    let second_width = second_label.len() as u16 + 2;
    let pair_width = first_width + 2 + second_width;
    let x = area.x + area.width.saturating_sub(pair_width) / 2;
    let y = buttons_row(area);
    (
        Rect {
            x,
            y,
            width: first_width,
            height: 1,
        },
        Rect {
            x: x + first_width + 2,
            y,
            width: second_width,
            height: 1,
        },
    )
}

/// The headline for an outcome, carrying the fallen or conquering civ's name.
fn headline(outcome: GameOutcome, civ_name: &str) -> String {
    match outcome {
        GameOutcome::Victory => {
            format!("The {civ_name} civilization has conquered the whole planet!")
        }
        GameOutcome::Defeat => format!("The {civ_name} civilization is no more!"),
    }
}

/// The tablet's width, clamped to survive narrow screens.
fn panel_rect(area: Rect) -> Rect {
    let width = PANEL_WIDTH.min(area.width.saturating_sub(2)).max(2);
    let x = area.x + area.width.saturating_sub(width) / 2;
    Rect {
        x,
        y: area.y + area.height.saturating_sub(BLOCK_HEIGHT) / 2,
        width,
        height: PANEL_HEIGHT,
    }
}

/// Split `text` into lines of at most `width` cells, breaking on word spaces
/// (and hard-chopping any single word longer than `width`).
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        if current.is_empty() {
            current.push_str(&word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(&word);
        } else {
            lines.push(std::mem::take(&mut current));
            if word.len() > width {
                loop {
                    let chop = width.min(word.len());
                    lines.push(word[..chop].to_string());
                    word.drain(..chop);
                    if word.is_empty() {
                        break;
                    }
                }
            } else {
                current.push_str(&word);
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn cell_width(ch: char) -> u16 {
    if ch as u32 >= 0x1_0000 { 2 } else { 1 }
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

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

/// Fill a rectangle with the given style, wiping any prior paint.
fn fill_rect(buf: &mut Buffer, rect: Rect, style: Style) {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }
}

/// The x that centres `text` within `rect`.
fn centred_x(rect: Rect, text_len: u16) -> u16 {
    rect.x + rect.width.saturating_sub(text_len) / 2
}

/// Paint the border of `rect` with the given edge and corner glyphs.
fn draw_border(
    buf: &mut Buffer,
    rect: Rect,
    horizontal: &str,
    vertical: &str,
    corners: (char, char, char, char),
    style: Style,
) {
    let (top_left, top_right, bottom_left, bottom_right) = corners;
    let x0 = rect.x;
    let x1 = rect.right().saturating_sub(1);
    let y0 = rect.y;
    let y1 = rect.bottom().saturating_sub(1);
    for x in x0..=x1 {
        set_cell(buf, x, y0, horizontal, style);
        set_cell(buf, x, y1, horizontal, style);
    }
    for y in y0..=y1 {
        set_cell(buf, x0, y, vertical, style);
        set_cell(buf, x1, y, vertical, style);
    }
    set_cell(buf, x0, y0, &top_left.to_string(), style);
    set_cell(buf, x1, y0, &top_right.to_string(), style);
    set_cell(buf, x0, y1, &bottom_left.to_string(), style);
    set_cell(buf, x1, y1, &bottom_right.to_string(), style);
}

/// The two buttons, the selected one picked out white, the pair centred
/// beneath the panel.
fn draw_buttons(
    buf: &mut Buffer,
    area: Rect,
    outcome: GameOutcome,
    first_selected: bool,
    backdrop: Color,
) {
    let (first, second) = button_rects(area, outcome);
    let (first_label, second_label) = button_labels(outcome);
    let (selected_rect, unselected_rect) = if first_selected {
        (first, second)
    } else {
        (second, first)
    };
    fill_rect(buf, selected_rect, Style::new().bg(SELECTED));
    fill_rect(buf, unselected_rect, Style::new().bg(backdrop));
    let first_style = if first_selected {
        Style::new()
            .fg(BLACK)
            .bg(SELECTED)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::new()
            .fg(MUTED)
            .bg(backdrop)
            .add_modifier(Modifier::UNDERLINED)
    };
    let second_style = if first_selected {
        Style::new()
            .fg(MUTED)
            .bg(backdrop)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::new()
            .fg(BLACK)
            .bg(SELECTED)
            .add_modifier(Modifier::UNDERLINED)
    };
    draw_text(buf, first.x + 1, first.y, first_label, first_style);
    draw_text(buf, second.x + 1, second.y, second_label, second_style);
}

impl Widget for GameOver {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.outcome {
            GameOutcome::Victory => self.render_celebration(area, buf),
            GameOutcome::Defeat => self.render_tombstone(area, buf),
        }
    }
}

impl GameOver {
    /// The fallen civilization's headstone against the dark: a grey stone
    /// slab, its "R I P" and the civilization's story cut into it.
    fn render_tombstone(&self, area: Rect, buf: &mut Buffer) {
        let backdrop = Style::new().bg(SLATE);
        fill_rect(buf, area, backdrop);
        let panel = panel_rect(area);
        fill_rect(buf, panel, Style::new().bg(STONE_FILL));
        draw_border(
            buf,
            panel,
            "─",
            "│",
            ('╭', '╮', '╰', '╯'),
            Style::new().fg(STONE_SHADE).bg(STONE_FILL),
        );
        let inscription = Style::new()
            .fg(INSCRIPTION)
            .bg(STONE_FILL)
            .add_modifier(Modifier::BOLD);
        draw_text(
            buf,
            centred_x(panel, RIP.len() as u16),
            panel.y + 1,
            RIP,
            inscription,
        );
        let headline = headline(GameOutcome::Defeat, &self.civ_name);
        let header_style = Style::new().fg(STONE_INK).bg(STONE_FILL);
        for (i, line) in wrap(&headline, panel.width.saturating_sub(4) as usize)
            .into_iter()
            .take(2)
            .enumerate()
        {
            draw_text(
                buf,
                centred_x(panel, line.len() as u16),
                panel.y + 3 + i as u16,
                &line,
                header_style,
            );
        }
        draw_buttons(buf, area, self.outcome, self.first_selected, SLATE);
    }

    /// The conquered planet's celebration: a double-lined banner over the
    /// night sky, gold confetti around the screen.
    fn render_celebration(&self, area: Rect, buf: &mut Buffer) {
        let backdrop = Style::new().bg(NIGHT);
        fill_rect(buf, area, backdrop);
        let confetti = Style::new().fg(GOLD).bg(NIGHT);
        for (x, y) in [
            (area.x + 3, area.y + 2),
            (area.right().saturating_sub(4), area.y + 2),
            (area.x + 3, area.y + area.height.saturating_sub(3)),
            (
                area.right().saturating_sub(4),
                area.y + area.height.saturating_sub(3),
            ),
        ] {
            set_cell(buf, x, y, "✦", confetti);
        }
        let panel = panel_rect(area);
        fill_rect(buf, panel, Style::new().bg(BANNER_FILL));
        draw_border(
            buf,
            panel,
            "═",
            "║",
            ('╔', '╗', '╚', '╝'),
            Style::new().fg(BANNER_BORDER).bg(BANNER_FILL),
        );
        let header_style = Style::new()
            .fg(GOLD)
            .bg(BANNER_FILL)
            .add_modifier(Modifier::BOLD);
        draw_text(
            buf,
            centred_x(panel, VICTORY_HEADER.len() as u16),
            panel.y + 1,
            VICTORY_HEADER,
            header_style,
        );
        let headline = headline(GameOutcome::Victory, &self.civ_name);
        let body_style = Style::new()
            .fg(PALE)
            .bg(BANNER_FILL)
            .add_modifier(Modifier::BOLD);
        for (i, line) in wrap(&headline, panel.width.saturating_sub(4) as usize)
            .into_iter()
            .take(2)
            .enumerate()
        {
            draw_text(
                buf,
                centred_x(panel, line.len() as u16),
                panel.y + 3 + i as u16,
                &line,
                body_style,
            );
        }
        draw_buttons(buf, area, self.outcome, self.first_selected, NIGHT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 40,
    };

    fn render(outcome: GameOutcome, civ: &str, first_selected: bool) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(GameOver::new(outcome, civ, first_selected), AREA))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn row_text(buf: &Buffer, x: u16, y: u16, len: u16) -> String {
        (0..len)
            .map(|i| buf.cell((x + i, y)).unwrap().symbol())
            .collect()
    }

    /// Whether `y` holds `needle` anywhere across the screen.
    fn row_contains(buf: &Buffer, y: u16, needle: &str) -> bool {
        let text: String = (0..AREA.width)
            .map(|x| buf.cell((x, y)).unwrap().symbol())
            .collect();
        text.contains(needle)
    }

    fn panel() -> Rect {
        panel_rect(AREA)
    }

    #[test]
    fn each_outcome_offers_the_pair_of_buttons() {
        assert_eq!(
            button_labels(GameOutcome::Victory),
            ("Continue playing", "Quit")
        );
        assert_eq!(button_labels(GameOutcome::Defeat), ("Start again", "Quit"));
    }

    #[test]
    fn the_buttons_sit_below_the_panel_side_by_side() {
        for outcome in [GameOutcome::Victory, GameOutcome::Defeat] {
            let (first, second) = button_rects(AREA, outcome);
            assert_eq!(buttons_row(AREA), 24, "centred below the panel");
            assert_eq!(first.y, second.y);
            assert!(first.x < second.x);
            assert_eq!(first.height, 1);
            assert_eq!(second.height, 1);
            // The pair never overlaps the panel that floats above them.
            assert!(first.y > panel().bottom(), "buttons below the panel");
        }
    }

    #[test]
    fn the_panel_is_centred_and_fits_with_the_buttons() {
        let panel = panel();
        assert_eq!(panel.width, PANEL_WIDTH);
        assert_eq!(panel.height, PANEL_HEIGHT);
        assert_eq!(panel.y, (40 - (PANEL_HEIGHT + 3)) / 2, "vertically centred");
        assert!(buttons_row(AREA) < AREA.height, "buttons stay on screen");
    }

    #[test]
    fn defeat_erects_a_tombstone_for_the_fallen_civilization() {
        let buf = render(GameOutcome::Defeat, "American", true);
        // The dim graveyard backdrop, the rounded headstone slab.
        assert_eq!(buf.cell((0, 0)).unwrap().bg, SLATE);
        let panel = panel_rect(AREA);
        assert_eq!(buf.cell((panel.x, panel.y)).unwrap().symbol(), "╭");
        assert_eq!(
            buf.cell((panel.x, panel.bottom() - 1)).unwrap().symbol(),
            "╰"
        );
        assert_eq!(
            buf.cell((panel.x + 1, panel.y)).unwrap().bg,
            STONE_FILL,
            "the slab is stone"
        );
        // The inscription and the civilization's story are cut into it.
        assert!(row_contains(&buf, panel.y + 1, "R I P"));
        assert!(row_contains(
            &buf,
            panel.y + 3,
            "The American civilization is no more!"
        ));
        // The first button is picked out white.
        let (first, _) = button_rects(AREA, GameOutcome::Defeat);
        assert_eq!(buf.cell((first.x, first.y)).unwrap().bg, SELECTED);
    }

    #[test]
    fn victory_raises_a_celebration_for_the_conquest() {
        let buf = render(GameOutcome::Victory, "American", false);
        // The night sky with its four gold confetti stars.
        assert_eq!(buf.cell((0, 0)).unwrap().bg, NIGHT);
        assert_eq!(buf.cell((3, 2)).unwrap().symbol(), "✦");
        assert_eq!(buf.cell((3, 2)).unwrap().fg, GOLD);
        // The double-lined banner carries the conquest.
        let panel = panel_rect(AREA);
        assert_eq!(buf.cell((panel.x, panel.y)).unwrap().symbol(), "╔");
        assert_eq!(
            buf.cell((panel.x, panel.bottom() - 1)).unwrap().symbol(),
            "╚"
        );
        assert!(row_contains(&buf, panel.y + 1, "VICTORY"));
        assert!(row_contains(
            &buf,
            panel.y + 3,
            "The American civilization has conquered the whole planet!"
        ));
        // The second button is the one picked out white.
        let (_, second) = button_rects(AREA, GameOutcome::Victory);
        assert_eq!(buf.cell((second.x, second.y)).unwrap().bg, SELECTED);
        let (first, _) = button_rects(AREA, GameOutcome::Victory);
        assert_eq!(buf.cell((first.x, first.y)).unwrap().bg, NIGHT);
    }

    #[test]
    fn a_wide_headline_wraps_inside_the_tombstone() {
        // "Babylonian" is the longest civ name; the headline stays on the
        // tombstone's single wrapped line rather than overflowing its sides.
        let buf = render(GameOutcome::Defeat, "Babylonian", true);
        let panel = panel_rect(AREA);
        assert!(row_contains(
            &buf,
            panel.y + 3,
            "The Babylonian civilization is no more!"
        ));
        // The tombstone's right border is untouched.
        assert_eq!(
            buf.cell((panel.right() - 1, panel.y + 3)).unwrap().symbol(),
            "│"
        );
    }

    #[test]
    fn the_button_labels_sit_inside_their_buttons() {
        let buf = render(GameOutcome::Victory, "American", true);
        let (first, second) = button_rects(AREA, GameOutcome::Victory);
        // The label starts one cell into the button, after its padding.
        assert_eq!(row_text(&buf, first.x, first.y, 1), " ");
        assert_eq!(
            row_text(&buf, first.x + 1, first.y, "Continue playing".len() as u16),
            "Continue playing"
        );
        assert_eq!(row_text(&buf, second.x + 1, second.y, 4), "Quit");
    }

    #[test]
    fn wrapping_splits_on_words_and_never_orphans_a_word() {
        let lines = wrap("one two three", 9);
        assert_eq!(lines, vec!["one two", "three"]);
        let lines = wrap("The Babylonian civilization is no more!", 24);
        assert_eq!(lines.join(" "), "The Babylonian civilization is no more!");
        assert!(lines.iter().all(|line| line.len() <= 24));
    }
}
