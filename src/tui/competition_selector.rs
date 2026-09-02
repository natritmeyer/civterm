use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::splash::SPACE_BG;
use super::theme::{ACCENT, DIM, HIGHLIGHT_BG, draw_text};
use crate::model::competition::Competition;

fn color_of(rivals: u8) -> Color {
    match rivals {
        1 => Color::Rgb(110, 200, 120),
        2 => Color::Rgb(150, 200, 100),
        3 => ACCENT,
        4 => Color::Rgb(220, 190, 90),
        5 => Color::Rgb(230, 160, 80),
        6 => Color::Rgb(235, 120, 70),
        7 => Color::Rgb(220, 90, 90),
        _ => ACCENT,
    }
}

fn tagline(rivals: u8) -> &'static str {
    match rivals {
        1 => "One rival empire on the map",
        2 => "Two rivals share the continents",
        3 => "Three rivals vie for dominance",
        4 => "Four rivals spread across the land",
        5 => "Five rivals crowd the frontier",
        6 => "Six rivals squeeze every corner",
        7 => "Seven rivals, a crowded earth",
        _ => "A world of rivals",
    }
}

const HEADER: &str = "LEVEL OF COMPETITION";
const SUBTITLE: &str = "How many other empires will rise against you?";
const FOOTER: &str = "↑ ↓ choose a level   ·   Enter select   ·   Esc back";
const LIST_X_OFFSET: u16 = 6;
const LIST_TOP: u16 = 4;

#[derive(Clone, Copy)]
pub struct CompetitionSelector {
    selected: usize,
    chosen: Option<Competition>,
}

impl CompetitionSelector {
    pub fn new(selected: usize, chosen: Option<Competition>) -> Self {
        Self { selected, chosen }
    }
}

fn list_x(area: Rect) -> u16 {
    area.x + LIST_X_OFFSET
}

fn list_top(area: Rect) -> u16 {
    area.y + LIST_TOP
}

impl Widget for CompetitionSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let roots = Competition::MAX - Competition::MIN + 1;
        let selected = self.selected.min(roots as usize - 1);

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
        for (i, rivals) in (Competition::MIN..=Competition::MAX).enumerate() {
            let y = top + i as u16;
            if y >= area.bottom() {
                break;
            }
            let tagline = tagline(rivals);
            let is_selected = i == selected;
            let mut number_style = Style::default().fg(color_of(rivals));
            if is_selected {
                number_style = number_style
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED);
                let text_end = list_x(area) + 5 + tagline.len() as u16;
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
            draw_text(
                buf,
                area.right(),
                list_x(area) + 2,
                y,
                &rivals.to_string(),
                number_style,
            );
            draw_text(
                buf,
                area.right(),
                list_x(area) + 4,
                y,
                tagline,
                Style::default().fg(DIM),
            );
        }

        if let Some(chosen) = self.chosen {
            let note = format!("✦ [{} rivals] selected", chosen.rivals());
            draw_text(
                buf,
                area.right(),
                list_x(area),
                top + 9,
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

    fn render(selected: usize, chosen: Option<Competition>) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(CompetitionSelector::new(selected, chosen), frame.area())
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn area() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn all_competition_levels_are_listed() {
        let buf = render(0, None);
        for (i, rivals) in (Competition::MIN..=Competition::MAX).enumerate() {
            let y = list_top(area()) + i as u16;
            assert_eq!(
                buf.cell((list_x(area()) + 2, y)).unwrap().symbol(),
                rivals.to_string()
            );
        }
    }

    #[test]
    fn each_level_shows_its_tagline() {
        let buf = render(0, None);
        for (i, rivals) in (Competition::MIN..=Competition::MAX).enumerate() {
            let y = list_top(area()) + i as u16;
            let start = list_x(area()) + 4;
            for (j, ch) in tagline(rivals).chars().enumerate() {
                assert_eq!(
                    buf.cell((start + j as u16, y)).unwrap().symbol(),
                    ch.to_string()
                );
            }
        }
    }

    #[test]
    fn selected_row_is_marked() {
        let buf = render(3, None);
        let marker = buf.cell((list_x(area()), list_top(area()) + 3)).unwrap();
        assert_eq!(marker.symbol(), "▸");
        assert_eq!(marker.style().fg, Some(ACCENT));
        let unselected = buf.cell((list_x(area()), list_top(area()))).unwrap();
        assert_eq!(unselected.symbol(), " ");
    }

    #[test]
    fn the_highlight_band_covers_the_whole_row_text() {
        let buf = render(4, None);
        let y = list_top(area()) + 4;
        let tagline = tagline(5);
        let first = list_x(area()) + 2;
        let last = first + 3 + tagline.len() as u16;
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
    fn chosen_competition_is_marked() {
        let buf = render(3, Some(Competition::new(4)));
        let note = "✦ [4 rivals] selected";
        for (j, ch) in note.chars().enumerate() {
            assert_eq!(
                buf.cell((list_x(area()) + j as u16, list_top(area()) + 9))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }
}
