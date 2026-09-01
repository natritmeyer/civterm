use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct SplashScreen;

const GLOBE_COLS: u16 = 41;
const GLOBE_ROWS: u16 = 21;
const TEXT_ROW: u16 = 8;
const TEXT_COL: u16 = 1;

const CIVTERM: [&str; 4] = [
    "        ▀         █                    ",
    "▄▀▀▀▄ ▀█  █   █ ▀█▀▀ ▄▀▀▀▄ ▀█▄▀▄ █▀▄▀▄",
    "█   ▄  █  ▀▄ ▄▀  █ ▄ █▀▀▀▀  █  ▀ █ █ █",
    " ▀▀▀  ▀▀▀   ▀     ▀   ▀▀▀  ▀▀▀   ▀ ▀ ▀",
];

const SPACE_BG: Color = Color::Rgb(12, 12, 40);
const OCEAN_FG: Color = Color::Rgb(70, 140, 210);
const OCEAN_BG: Color = Color::Rgb(25, 60, 150);
const TEXT_FG: Color = Color::White;
const GLOBE_CX: f32 = 20.0;
const GLOBE_CY: f32 = 10.0;
const GLOBE_RX: f32 = 20.0;
const GLOBE_RY: f32 = 10.0;

const LAND: [(f32, f32, f32, f32); 7] = [
    (9.0, 6.0, 3.4, 3.0),
    (13.0, 13.0, 2.6, 3.4),
    (17.5, 3.5, 2.6, 1.6),
    (19.0, 11.5, 3.6, 4.0),
    (28.0, 5.0, 5.2, 3.6),
    (25.0, 10.0, 2.2, 2.2),
    (30.5, 15.0, 2.0, 1.5),
];

const CLOUDS: [(usize, usize); 4] = [(6, 12), (12, 4), (24, 16), (33, 9)];

impl SplashScreen {
    pub fn new() -> Self {
        SplashScreen
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_bg(SPACE_BG);
                    if let Some((sym, fg)) = star_at(x, y) {
                        cell.set_symbol(&sym.to_string());
                        cell.set_fg(fg);
                    }
                }
            }
        }

        let gx = area
            .x
            .saturating_add(area.width / 2)
            .saturating_sub(GLOBE_COLS / 2);
        let gy = area
            .y
            .saturating_add(area.height / 2)
            .saturating_sub(GLOBE_ROWS / 2);
        for row in 0..GLOBE_ROWS {
            for col in 0..GLOBE_COLS {
                let x = gx + col;
                let y = gy + row;
                if x >= area.right() || y >= area.bottom() {
                    continue;
                }
                let (sym, fg, bg) = match globe_cell(col as usize, row as usize) {
                    Some(cell) => cell,
                    None => continue,
                };
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_symbol(&sym.to_string());
                    cell.set_fg(fg);
                    cell.set_bg(bg);
                }
            }
        }

        let letter_style = Style::default().fg(TEXT_FG).add_modifier(Modifier::BOLD);
        for (i, line) in CIVTERM.iter().enumerate() {
            let y = gy + TEXT_ROW + i as u16;
            if y >= area.bottom() {
                continue;
            }
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = gx + TEXT_COL + j as u16;
                if x >= area.right() {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(ch);
                    cell.set_style(letter_style);
                }
            }
        }
    }
}

impl Default for SplashScreen {
    fn default() -> Self {
        SplashScreen::new()
    }
}

impl Widget for SplashScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        SplashScreen::render(&self, area, buf);
    }
}

fn star_at(x: u16, y: u16) -> Option<(char, Color)> {
    let mut h = (x as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add((y as u32).wrapping_mul(2_891_336_453));
    h ^= h >> 13;
    h = h.wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    match h % 23 {
        0 => Some(('*', Color::Yellow)),
        1 => Some(('.', Color::White)),
        2 => Some(('+', Color::Cyan)),
        3 => Some((':', Color::Rgb(160, 160, 200))),
        _ => None,
    }
}

fn globe_cell(x: usize, y: usize) -> Option<(char, Color, Color)> {
    let dx = x as f32 - GLOBE_CX;
    let dy = y as f32 - GLOBE_CY;
    let inside = (dx / GLOBE_RX).powi(2) + (dy / GLOBE_RY).powi(2) <= 1.0;
    if !inside {
        return None;
    }
    if is_land(x as f32, y as f32) {
        return Some(('#', Color::Green, OCEAN_BG));
    }
    if CLOUDS.contains(&(x, y)) {
        return Some(('.', Color::White, OCEAN_BG));
    }
    Some(('~', OCEAN_FG, OCEAN_BG))
}

fn is_land(x: f32, y: f32) -> bool {
    LAND.iter()
        .any(|&(cx, cy, rx, ry)| ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_area(width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(SplashScreen::new(), frame.area());
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn starfield_scales_with_the_window() {
        let buf = render_area(100, 60);
        let stars = (0..60)
            .map(|y| {
                (0..100)
                    .filter(|&x| matches!(star_at(x, y), Some((_, _))))
                    .count()
            })
            .sum::<usize>();
        assert!(stars > 0);
        assert_eq!(buf.cell((50, 0)).unwrap().style().bg, Some(SPACE_BG));
    }

    #[test]
    fn globe_and_text_are_centred() {
        let buf = render_area(100, 60);
        let gx = 100 / 2 - GLOBE_COLS / 2; // 30
        let gy = 60 / 2 - GLOBE_ROWS / 2; // 20
        assert_eq!(
            buf.cell((gx + 20, gy + 17)).unwrap().style().bg,
            Some(OCEAN_BG)
        );
        let land_at_am = buf.cell((gx + 9, gy + 6)).unwrap();
        assert_eq!(land_at_am.symbol(), "#");
        assert_eq!(land_at_am.style().fg, Some(Color::Green));
        let first_text = buf.cell((gx + TEXT_COL + 8, gy + TEXT_ROW)).unwrap();
        assert_eq!(first_text.symbol(), "▀");
        assert_eq!(first_text.style().fg, Some(TEXT_FG));
        assert_eq!(first_text.style().bg, Some(OCEAN_BG));
        let gap = buf.cell((gx + TEXT_COL, gy + TEXT_ROW)).unwrap();
        assert_eq!(gap.symbol(), "~");
        assert_eq!(gap.style().bg, Some(OCEAN_BG));
    }

    #[test]
    fn small_window_still_renders() {
        let buf = render_area(10, 10);
        assert_eq!(buf.cell((0, 0)).unwrap().style().bg, Some(SPACE_BG));
    }
}
