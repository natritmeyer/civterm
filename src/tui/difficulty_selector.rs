use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use strum::IntoEnumIterator;

use super::splash::SPACE_BG;
use super::theme::{ACCENT, DIM, HIGHLIGHT_BG, draw_text};
use crate::model::difficulty::Difficulty;

fn color_of(difficulty: Difficulty) -> Color {
    match difficulty {
        Difficulty::Easy => Color::Rgb(110, 200, 120),
        Difficulty::Normal => ACCENT,
        Difficulty::Hard => Color::Rgb(220, 90, 90),
    }
}

fn tagline(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => "Settled soil and gentle tribes",
        Difficulty::Normal => "The classic balance of empire",
        Difficulty::Hard => "Ruthless rivals at every step",
    }
}

const HEADER: &str = "CHOOSE DIFFICULTY";
const SUBTITLE: &str = "How harsh should your world be?";
const FOOTER: &str = "↑ ↓ choose a difficulty   ·   Enter select   ·   Esc back";
const LIST_X_OFFSET: u16 = 6;
const LIST_TOP: u16 = 4;

#[derive(Clone, Copy)]
pub struct DifficultySelector {
    selected: usize,
    chosen: Option<Difficulty>,
}

impl DifficultySelector {
    pub fn new(selected: usize, chosen: Option<Difficulty>) -> Self {
        Self { selected, chosen }
    }
}

fn list_x(area: Rect) -> u16 {
    area.x + LIST_X_OFFSET
}

fn list_top(area: Rect) -> u16 {
    area.y + LIST_TOP
}

impl Widget for DifficultySelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected = self.selected.min(Difficulty::iter().count() - 1);

        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(SPACE_BG);
                }
            }
        }

        let cx = area.x + area.width / 2;
        let header_y = area.y + 1;
        if header_y < area.bottom() {
            draw_text(
                buf,
                area.right(),
                cx.saturating_sub(HEADER.len() as u16 / 2),
                header_y,
                HEADER,
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            );
            draw_text(
                buf,
                area.right(),
                cx.saturating_sub(SUBTITLE.len() as u16 / 2),
                header_y + 1,
                SUBTITLE,
                Style::default().fg(DIM),
            );
        }

        let top = list_top(area);
        for (i, difficulty) in Difficulty::iter().enumerate() {
            let y = top + i as u16;
            if y >= area.bottom() {
                break;
            }
            let name = difficulty.display_name();
            let tagline = tagline(difficulty);
            let is_selected = i == selected;
            let mut name_style = Style::default().fg(color_of(difficulty));
            if is_selected {
                name_style = name_style
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED);
                let text_end = list_x(area) + 2 + name.len() as u16 + 2 + tagline.len() as u16;
                let band_end = text_end.min(area.right());
                for x in list_x(area) + 2..band_end {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(" ");
                        cell.set_bg(HIGHLIGHT_BG);
                    }
                }
            }
            let marker = if is_selected { "▸" } else { " " };
            draw_text(
                buf,
                area.right(),
                list_x(area),
                y,
                marker,
                Style::default().fg(ACCENT),
            );
            draw_text(buf, area.right(), list_x(area) + 2, y, name, name_style);
            draw_text(
                buf,
                area.right(),
                list_x(area) + 4 + name.len() as u16,
                y,
                tagline,
                Style::default().fg(DIM),
            );
        }

        if let Some(chosen) = self.chosen {
            let note = format!("✦ [{}] selected", chosen.display_name());
            draw_text(
                buf,
                area.right(),
                list_x(area),
                top + 4,
                &note,
                Style::default().fg(ACCENT),
            );
        }

        let footer_y = area.bottom().saturating_sub(1);
        if footer_y >= area.y {
            draw_text(
                buf,
                area.right(),
                cx.saturating_sub(FOOTER.len() as u16 / 2),
                footer_y,
                FOOTER,
                Style::default().fg(DIM),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(selected: usize, chosen: Option<Difficulty>) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(DifficultySelector::new(selected, chosen), frame.area())
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn area() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn all_difficulties_are_listed() {
        let buf = render(0, None);
        assert_eq!(Difficulty::iter().count(), 3);
        for (i, difficulty) in Difficulty::iter().enumerate() {
            let y = list_top(area()) + i as u16;
            let start = list_x(area()) + 2;
            for (j, ch) in difficulty.display_name().chars().enumerate() {
                assert_eq!(
                    buf.cell((start + j as u16, y)).unwrap().symbol(),
                    ch.to_string()
                );
            }
        }
    }

    #[test]
    fn each_difficulty_shows_its_tagline() {
        let buf = render(0, None);
        for (i, difficulty) in Difficulty::iter().enumerate() {
            let y = list_top(area()) + i as u16;
            let start = list_x(area()) + 4 + difficulty.display_name().len() as u16;
            for (j, ch) in tagline(difficulty).chars().enumerate() {
                assert_eq!(
                    buf.cell((start + j as u16, y)).unwrap().symbol(),
                    ch.to_string()
                );
            }
        }
    }

    #[test]
    fn selected_row_is_marked() {
        let buf = render(1, None);
        let marker = buf.cell((list_x(area()), list_top(area()) + 1)).unwrap();
        assert_eq!(marker.symbol(), "▸");
        assert_eq!(marker.style().fg, Some(ACCENT));
        let unselected = buf.cell((list_x(area()), list_top(area()))).unwrap();
        assert_eq!(unselected.symbol(), " ");
    }

    #[test]
    fn the_highlight_band_covers_the_whole_row_text() {
        let buf = render(1, None);
        let y = list_top(area()) + 1;
        let name = Difficulty::Normal.display_name();
        let tagline = tagline(Difficulty::Normal);
        let first = list_x(area()) + 2;
        let last = first + name.len() as u16 + 2 + tagline.len() as u16;
        for x in first..last {
            assert_eq!(
                buf.cell((x, y)).unwrap().style().bg,
                Some(HIGHLIGHT_BG),
                "column {x} should be highlighted"
            );
        }
        assert_ne!(
            buf.cell((last, y)).unwrap().style().bg,
            Some(HIGHLIGHT_BG),
            "highlight should stop at the end of the text"
        );
    }

    #[test]
    fn chosen_difficulty_is_marked() {
        let buf = render(1, Some(Difficulty::Normal));
        let note = "✦ [Normal] selected";
        for (j, ch) in note.chars().enumerate() {
            assert_eq!(
                buf.cell((list_x(area()) + j as u16, list_top(area()) + 4))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }
}
