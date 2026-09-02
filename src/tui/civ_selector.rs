use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use strum::IntoEnumIterator;

use super::splash::SPACE_BG;
use crate::model::civilizations::Civilization;

fn color_of(civ: Civilization) -> Color {
    match civ {
        Civilization::American => Color::Rgb(120, 160, 220),
        Civilization::Aztec => Color::Rgb(200, 120, 70),
        Civilization::Babylonian => Color::Rgb(110, 130, 230),
        Civilization::Chinese => Color::Rgb(235, 200, 90),
        Civilization::Egyptian => Color::Rgb(218, 179, 60),
        Civilization::English => Color::Rgb(210, 90, 90),
        Civilization::French => Color::Rgb(120, 90, 210),
        Civilization::German => Color::Rgb(140, 140, 150),
        Civilization::Greek => Color::Rgb(120, 190, 255),
        Civilization::Indian => Color::Rgb(90, 180, 90),
        Civilization::Mongol => Color::Rgb(170, 150, 100),
        Civilization::Roman => Color::Rgb(200, 70, 70),
        Civilization::Russian => Color::Rgb(215, 120, 60),
        Civilization::Zulu => Color::Rgb(170, 110, 60),
    }
}

const HEADER: &str = "CHOOSE YOUR CIVILIZATION";
const SUBTITLE: &str = "Build an empire that will stand the test of time";
const FOOTER: &str = "↑ ↓ choose a civilization   ·   Enter select   ·   Esc back";
const ACCENT: Color = Color::Rgb(212, 175, 55);
const DIM: Color = Color::Rgb(120, 120, 160);
const HIGHLIGHT_BG: Color = Color::Rgb(45, 45, 95);
const LIST_X_OFFSET: u16 = 6;
const LIST_TOP: u16 = 4;
const BAND_WIDTH: u16 = 20;

#[derive(Clone, Copy)]
pub struct CivSelector {
    selected: usize,
    chosen: Option<Civilization>,
}

impl CivSelector {
    pub fn new(selected: usize, chosen: Option<Civilization>) -> Self {
        Self { selected, chosen }
    }
}

fn list_x(area: Rect) -> u16 {
    area.x + LIST_X_OFFSET
}

fn list_top(area: Rect) -> u16 {
    area.y + LIST_TOP
}

fn divider_x(area: Rect) -> u16 {
    area.x + area.width * 5 / 12
}

fn detail_x(area: Rect) -> u16 {
    divider_x(area) + 3
}

fn draw_text(buf: &mut Buffer, area_right: u16, x: u16, y: u16, text: &str, style: Style) {
    for (offset, ch) in (x..).zip(text.chars()) {
        if offset >= area_right {
            return;
        }
        if let Some(cell) = buf.cell_mut((offset, y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
        }
    }
}

impl Widget for CivSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected = self.selected.min(Civilization::iter().count() - 1);

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
        let count = Civilization::iter().count() as u16;
        let bottom = top.saturating_add(count);
        for (i, civ) in Civilization::iter().enumerate() {
            let y = top + i as u16;
            if y >= area.bottom() {
                break;
            }
            let is_selected = i == selected;
            let mut name_style = Style::default().fg(color_of(civ));
            if is_selected {
                name_style = name_style
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED);
                let band_end = (list_x(area) + 2 + BAND_WIDTH).min(area.right());
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
                civ.display_name(),
                name_style,
            );
        }

        let div_x = divider_x(area);
        for y in top..bottom.min(area.bottom()) {
            if let Some(cell) = buf.cell_mut((div_x, y)) {
                cell.set_symbol("│");
                cell.set_fg(DIM);
            }
        }

        if let Some(civ) = Civilization::iter().nth(selected) {
            let dx = detail_x(area);
            draw_text(
                buf,
                area.right(),
                dx,
                top,
                "LEADER",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            );
            draw_text(
                buf,
                area.right(),
                dx,
                top + 1,
                civ.ruler().display_name(),
                Style::default().fg(Color::White),
            );
            let motto_y = top + 3;
            if motto_y < area.bottom() {
                draw_text(
                    buf,
                    area.right(),
                    dx,
                    motto_y,
                    "MOTTO",
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                );
                draw_text(
                    buf,
                    area.right(),
                    dx,
                    motto_y + 1,
                    civ.motto(),
                    Style::default().fg(DIM),
                );
            }
        }

        if let Some(chosen) = self.chosen {
            let note = format!("✦ [{}] chosen", chosen.display_name());
            draw_text(
                buf,
                area.right(),
                detail_x(area),
                top + 6,
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

    fn render(selected: usize, chosen: Option<Civilization>) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(CivSelector::new(selected, chosen), frame.area()))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn area() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn all_civilizations_are_listed() {
        let buf = render(0, None);
        assert_eq!(Civilization::iter().count(), 14);
        for (i, civ) in Civilization::iter().enumerate() {
            let y = list_top(area()) + i as u16;
            let start = list_x(area()) + 2;
            for (j, ch) in civ.display_name().chars().enumerate() {
                assert_eq!(
                    buf.cell((start + j as u16, y)).unwrap().symbol(),
                    ch.to_string()
                );
            }
        }
    }

    #[test]
    fn selected_row_is_marked() {
        let buf = render(2, None);
        let marker = buf.cell((list_x(area()), list_top(area()) + 2)).unwrap();
        assert_eq!(marker.symbol(), "▸");
        assert_eq!(marker.style().fg, Some(ACCENT));
        let unselected = buf.cell((list_x(area()), list_top(area()))).unwrap();
        assert_eq!(unselected.symbol(), " ");
    }

    #[test]
    fn detail_shows_the_selected_civs_leader() {
        let buf = render(3, None);
        let y = list_top(area()) + 1;
        let leader = Civilization::iter().nth(3).unwrap().ruler().display_name();
        for (j, ch) in leader.chars().enumerate() {
            assert_eq!(
                buf.cell((detail_x(area()) + j as u16, y)).unwrap().symbol(),
                ch.to_string()
            );
        }
    }

    #[test]
    fn chosen_civ_is_marked() {
        let buf = render(4, Some(Civilization::Chinese));
        let note = "✦ [Chinese] chosen";
        for (j, ch) in note.chars().enumerate() {
            assert_eq!(
                buf.cell((detail_x(area()) + j as u16, list_top(area()) + 6))
                    .unwrap()
                    .symbol(),
                ch.to_string()
            );
        }
    }
}
