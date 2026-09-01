use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::splash::SPACE_BG;

pub(crate) const ITEMS: [&str; 3] = ["(N)ew Game", "(L)oad Saved Game", "(Q)uit"];
const ITEM_GAP: usize = 4;
const BAR_BG_END: Color = Color::Rgb(24, 24, 56);
const TEXT_DIM: Color = Color::Rgb(45, 54, 100);
const TEXT_BRIGHT: Color = Color::Rgb(235, 230, 215);
const TEXT_SELECT: Color = Color::Yellow;

#[derive(Clone, Copy)]
pub struct StatusBar {
    progress: f32,
    selected: usize,
}

impl StatusBar {
    pub fn new(progress: f32, selected: usize) -> Self {
        Self { progress, selected }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn fade_color(start: Color, end: Color, t: f32) -> Color {
    if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (start, end) {
        Color::Rgb(lerp(r1, r2, t), lerp(g1, g2, t), lerp(b1, b2, t))
    } else {
        end
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let bg = fade_color(SPACE_BG, BAR_BG_END, self.progress);
        for x in area.x..area.right() {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.reset();
                cell.set_fg(bg);
                cell.set_bg(bg);
            }
        }

        let selected = self.selected.min(ITEMS.len() - 1);
        let total = ITEMS.iter().map(|s| s.len()).sum::<usize>() + ITEM_GAP * (ITEMS.len() - 1);
        let mut x = area.x + area.width.saturating_sub(total as u16) / 2;
        for (i, item) in ITEMS.iter().enumerate() {
            for (j, ch) in item.chars().enumerate() {
                if x >= area.right() {
                    return;
                }
                if let Some(cell) = buf.cell_mut((x, area.y)) {
                    cell.reset();
                    cell.set_symbol(&ch.to_string());
                    let is_selected = i == selected;
                    let bright = if is_selected {
                        TEXT_SELECT
                    } else {
                        TEXT_BRIGHT
                    };
                    let mut style =
                        Style::default().fg(fade_color(TEXT_DIM, bright, self.progress));
                    if is_selected {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if j == 1 {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    cell.set_style(style);
                    cell.set_bg(bg);
                }
                x += 1;
            }
            if i + 1 < ITEMS.len() {
                for _ in 0..ITEM_GAP {
                    if x >= area.right() {
                        return;
                    }
                    if let Some(cell) = buf.cell_mut((x, area.y)) {
                        cell.reset();
                        cell.set_symbol(" ");
                        cell.set_fg(bg);
                        cell.set_bg(bg);
                    }
                    x += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_status(progress: f32, selected: usize) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(80, 1)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(StatusBar::new(progress, selected), frame.area());
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn item_offset(item: usize) -> u16 {
        let mut offset = 0usize;
        for (i, text) in ITEMS.iter().enumerate() {
            if i == item {
                break;
            }
            offset += text.len() + ITEM_GAP;
        }
        (80u16 - ITEMS.iter().map(|s| s.len() + ITEM_GAP).sum::<usize>() as u16 + ITEM_GAP as u16)
            / 2
            + offset as u16
    }

    #[test]
    fn text_is_centred_when_fully_visible() {
        let buf = render_status(1.0, 0);
        let text_x = item_offset(0);
        let first = buf.cell((text_x, 0)).unwrap();
        assert_eq!(first.symbol(), "(");
        let mnemonic = buf.cell((text_x + 1, 0)).unwrap();
        assert_eq!(mnemonic.symbol(), "N");
        assert_eq!(mnemonic.style().fg, Some(TEXT_SELECT));
        assert!(mnemonic.style().add_modifier.contains(Modifier::UNDERLINED));
        assert!(mnemonic.style().add_modifier.contains(Modifier::BOLD));
        assert_eq!(first.style().bg, Some(BAR_BG_END));
    }

    #[test]
    fn selected_item_is_highlighted() {
        let buf = render_status(1.0, 1);
        let selected_start = item_offset(1);
        let selected = buf.cell((selected_start + 1, 0)).unwrap();
        assert_eq!(selected.symbol(), "L");
        assert_eq!(selected.style().fg, Some(TEXT_SELECT));
        assert!(selected.style().add_modifier.contains(Modifier::UNDERLINED));
        let unselected = buf.cell((item_offset(0) + 1, 0)).unwrap();
        assert_eq!(unselected.symbol(), "N");
        assert_eq!(unselected.style().fg, Some(TEXT_BRIGHT));
        assert!(
            !unselected
                .style()
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(unselected.style().add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn bar_starts_dark_and_dim() {
        let buf = render_status(0.0, 0);
        let first = buf.cell((40, 0)).unwrap();
        assert_eq!(first.style().fg, Some(TEXT_DIM));
        assert_eq!(first.style().bg, Some(SPACE_BG));
    }

    #[test]
    fn fade_midway_blends_colours() {
        let buf = render_status(0.5, 0);
        let first = buf.cell((40, 0)).unwrap();
        assert_eq!(
            first.style().fg,
            Some(Color::Rgb(
                lerp(45, 235, 0.5),
                lerp(54, 230, 0.5),
                lerp(100, 215, 0.5)
            ))
        );
    }

    #[test]
    fn all_three_options_appear() {
        let buf = render_status(1.0, 2);
        for (item, text) in ITEMS.iter().enumerate() {
            let start = item_offset(item);
            for (i, ch) in text.chars().enumerate() {
                assert_eq!(
                    buf.cell((start + i as u16, 0)).unwrap().symbol(),
                    ch.to_string()
                );
            }
        }
    }
}
