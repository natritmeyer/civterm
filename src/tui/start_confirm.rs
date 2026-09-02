use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::splash::SPACE_BG;
use super::theme::{ACCENT, DIM, draw_text};
use crate::model::civilizations::Civilization;
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;

const TITLE: &str = "READY TO BEGIN";
const SUBTITLE: &str = "Review your selections and confirm your path, emperor.";
const FOOTER: &str = "(S)tart game   ·   (Q)uit   ·   Esc back";
const START_STYLE: Style = Style::new()
    .fg(Color::Rgb(120, 200, 120))
    .add_modifier(Modifier::BOLD);
const QUIT_STYLE: Style = Style::new().fg(ACCENT);
const LABEL_STYLE: Style = Style::new().fg(DIM);
const VALUE_STYLE: Style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);

#[derive(Clone, Copy)]
pub struct StartConfirm {
    civilization: Civilization,
    competition: Competition,
    difficulty: Difficulty,
}

impl StartConfirm {
    pub fn new(
        civilization: Civilization,
        competition: Competition,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            civilization,
            competition,
            difficulty,
        }
    }
}

impl Widget for StartConfirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(SPACE_BG);
                }
            }
        }

        let cx = area.x + area.width / 2;
        let top = area.y + area.height / 2 - 5;

        draw_text(
            buf,
            area.right(),
            cx.saturating_sub(TITLE.len() as u16 / 2),
            top,
            TITLE,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        );
        draw_text(
            buf,
            area.right(),
            cx.saturating_sub(SUBTITLE.len() as u16 / 2),
            top + 1,
            SUBTITLE,
            Style::default().fg(DIM),
        );

        let panel_x = cx.saturating_sub(SUBTITLE.len() as u16 / 2);
        let panel_top = top + 3;
        draw_row(
            buf,
            area.right(),
            panel_x,
            panel_top,
            "Civilization",
            self.civilization.display_name(),
        );
        draw_row(
            buf,
            area.right(),
            panel_x,
            panel_top + 1,
            "Competition",
            &format!("{} rivals", self.competition.rivals()),
        );
        draw_row(
            buf,
            area.right(),
            panel_x,
            panel_top + 2,
            "Difficulty",
            self.difficulty.display_name(),
        );

        draw_text(
            buf,
            area.right(),
            cx.saturating_sub(2),
            top + 7,
            "▸ Start",
            START_STYLE,
        );
        draw_text(
            buf,
            area.right(),
            cx.saturating_sub(2),
            top + 8,
            "▸ Quit",
            QUIT_STYLE,
        );

        let footer_y = area.bottom().saturating_sub(1);
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

fn draw_row(buf: &mut Buffer, right: u16, x: u16, y: u16, label: &'static str, value: &str) {
    draw_text(buf, right, x, y, label, LABEL_STYLE);
    let value_x = x + 16;
    draw_text(buf, right, value_x, y, value, VALUE_STYLE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render() -> Buffer {
        render_with(
            Civilization::American,
            Competition::new(3),
            Difficulty::Normal,
        )
    }

    fn render_with(
        civilization: Civilization,
        competition: Competition,
        difficulty: Difficulty,
    ) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    StartConfirm::new(civilization, competition, difficulty),
                    frame.area(),
                )
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn title_is_centered() {
        let buf = render();
        let cx = 50u16;
        let title_y = 20;
        for (i, ch) in TITLE.chars().enumerate() {
            assert_eq!(
                buf.cell((cx - TITLE.len() as u16 / 2 + i as u16, title_y))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }

    #[test]
    fn subtitle_sits_under_the_title() {
        let buf = render();
        let cx = 50u16;
        let y = 21;
        for (i, ch) in SUBTITLE.chars().enumerate() {
            assert_eq!(
                buf.cell((cx - SUBTITLE.len() as u16 / 2 + i as u16, y))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }

    #[test]
    fn the_selections_are_reviewed() {
        let buf = render_with(
            Civilization::Babylonian,
            Competition::new(5),
            Difficulty::Hard,
        );
        let cx = 50u16;
        let panel_x = cx - SUBTITLE.len() as u16 / 2;
        let panel_top = 23u16;
        let start = panel_x + 16;
        for (i, ch) in "Babylonian".chars().enumerate() {
            assert_eq!(
                buf.cell((start + i as u16, panel_top)).unwrap().symbol(),
                ch.to_string()
            );
        }
        for (i, ch) in "5 rivals".chars().enumerate() {
            assert_eq!(
                buf.cell((start + i as u16, panel_top + 1))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
        for (i, ch) in "Hard".chars().enumerate() {
            assert_eq!(
                buf.cell((start + i as u16, panel_top + 2))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }

    #[test]
    fn the_two_options_are_stacked_with_clean_columns() {
        let buf = render();
        let start_row: String = (0..100)
            .filter_map(|x| buf.cell((x, 27)).map(|c| c.symbol().to_string()))
            .collect();
        let quit_row: String = (0..100)
            .filter_map(|x| buf.cell((x, 28)).map(|c| c.symbol().to_string()))
            .collect();
        assert_eq!(start_row.trim(), "▸ Start");
        assert_eq!(quit_row.trim(), "▸ Quit");
    }

    #[test]
    fn footer_shows_key_hints() {
        let buf = render();
        let cx = 50u16;
        let footer_y = 49;
        for (i, ch) in FOOTER.chars().enumerate() {
            assert_eq!(
                buf.cell((cx - FOOTER.len() as u16 / 2 + i as u16, footer_y))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }
}
