use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct SplashScreen;

const GLOBE_COLS: u16 = 61;
const GLOBE_ROWS: u16 = 31;
const TEXT_ROW: u16 = 13;

const SCROLL_WIDTH: u16 = 46;
const SCROLL_HEIGHT: u16 = 8;
const SCROLL_TEXT_OFFSET: u16 = 4;
const SCROLL_TOP_ROW: u16 = 11;

const PARCH_BG: Color = Color::Rgb(222, 196, 140);
const PARCH_FG: Color = Color::Rgb(80, 60, 25);
const SCROLL_EDGE: Color = Color::Rgb(140, 110, 65);
const ATTRIBUTION: &str = "...by Nat Ritmeyer";

const CIVTERM: [&str; 4] = [
    "        ▀         █                    ",
    "▄▀▀▀▄ ▀█  █   █ ▀█▀▀ ▄▀▀▀▄ ▀█▄▀▄ █▀▄▀▄",
    "█   ▄  █  ▀▄ ▄▀  █ ▄ █▀▀▀▀  █  ▀ █ █ █",
    " ▀▀▀  ▀▀▀   ▀     ▀   ▀▀▀  ▀▀▀   ▀ ▀ ▀",
];

pub(crate) const SPACE_BG: Color = Color::Rgb(12, 12, 40);
const OCEAN_FG: Color = Color::Rgb(70, 140, 210);
const OCEAN_BG: Color = Color::Rgb(25, 60, 150);
const TEXT_FG: Color = Color::Rgb(80, 60, 25);
const GLOBE_CX: f32 = 30.0;
const GLOBE_CY: f32 = 15.0;
const GLOBE_RX: f32 = 30.0;
const GLOBE_RY: f32 = 15.0;

const LAND: [(f32, f32, f32, f32); 7] = [
    (13.5, 9.0, 5.2, 4.6),
    (19.5, 19.5, 3.9, 5.1),
    (26.0, 5.5, 3.9, 2.4),
    (28.5, 17.0, 5.4, 6.0),
    (42.0, 7.5, 7.8, 5.4),
    (37.5, 15.0, 3.3, 3.3),
    (45.5, 22.5, 3.0, 2.3),
];

const CLOUDS: [(usize, usize); 4] = [(9, 18), (18, 6), (36, 24), (50, 14)];

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

        let scroll_x = area
            .x
            .saturating_add(area.width / 2)
            .saturating_sub(SCROLL_WIDTH / 2);
        let scroll_y = gy.saturating_add(SCROLL_TOP_ROW);
        for row in 0..SCROLL_HEIGHT {
            let y = scroll_y + row;
            if y >= area.bottom() {
                continue;
            }
            for col in 0..SCROLL_WIDTH {
                let x = scroll_x + col;
                if x >= area.right() {
                    continue;
                }
                let symbol = if row == 0 {
                    '▀'
                } else if row == 1 || row == 6 {
                    '█'
                } else if row == 7 {
                    '▄'
                } else if col == 0 || col == SCROLL_WIDTH - 1 {
                    '█'
                } else {
                    ' '
                };
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_symbol(&symbol.to_string());
                    cell.set_fg(if symbol == ' ' { PARCH_FG } else { SCROLL_EDGE });
                    cell.set_bg(PARCH_BG);
                }
            }
        }

        let attrib_len = ATTRIBUTION.len() as u16;
        let attrib_x = scroll_x + (SCROLL_WIDTH - attrib_len) / 2;
        let attrib_y = scroll_y + 6;
        if attrib_y < area.bottom() {
            for (j, ch) in ATTRIBUTION.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = attrib_x + j as u16;
                if x >= area.right() {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x, attrib_y)) {
                    cell.reset();
                    cell.set_symbol(&ch.to_string());
                    cell.set_fg(PARCH_FG);
                    cell.set_bg(SCROLL_EDGE);
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
                let x = scroll_x + SCROLL_TEXT_OFFSET + j as u16;
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
        let gx = 100 / 2 - GLOBE_COLS / 2; // 20
        let gy = 60 / 2 - GLOBE_ROWS / 2; // 15
        let sx = 100 / 2 - SCROLL_WIDTH / 2; // 27
        let sy = gy + SCROLL_TOP_ROW; // 26
        assert_eq!(
            buf.cell((gx + 20, gy + 25)).unwrap().style().bg,
            Some(OCEAN_BG)
        );
        let land_at_am = buf.cell((gx + 13, gy + 9)).unwrap();
        assert_eq!(land_at_am.symbol(), "#");
        assert_eq!(land_at_am.style().fg, Some(Color::Green));
        let first_text = buf.cell((sx + SCROLL_TEXT_OFFSET + 8, sy + 2)).unwrap();
        assert_eq!(first_text.symbol(), "▀");
        assert_eq!(first_text.style().fg, Some(TEXT_FG));
        assert_eq!(first_text.style().bg, Some(PARCH_BG));
        let gap = buf.cell((sx + SCROLL_TEXT_OFFSET, sy + 2)).unwrap();
        assert_eq!(gap.symbol(), " ");
        assert_eq!(gap.style().bg, Some(PARCH_BG));
    }

    #[test]
    fn scroll_frames_the_banner() {
        let buf = render_area(100, 60);
        let sx = 100 / 2 - SCROLL_WIDTH / 2;
        let sy = 60 / 2 - GLOBE_ROWS / 2 + SCROLL_TOP_ROW;
        let corner = buf.cell((sx, sy)).unwrap();
        assert_eq!(corner.symbol(), "▀");
        assert_eq!(corner.style().fg, Some(SCROLL_EDGE));
        let roll = buf.cell((sx, sy + 1)).unwrap();
        assert_eq!(roll.symbol(), "█");
        let dowel = buf.cell((sx, sy + 2)).unwrap();
        assert_eq!(dowel.symbol(), "█");
        let attrib = buf
            .cell((
                sx + (SCROLL_WIDTH - ATTRIBUTION.len() as u16) / 2 + 7,
                sy + 6,
            ))
            .unwrap();
        assert_eq!(attrib.symbol(), "a");
        assert_eq!(attrib.style().fg, Some(PARCH_FG));
        assert_eq!(attrib.style().bg, Some(SCROLL_EDGE));
        let underside = buf.cell((sx, sy + 7)).unwrap();
        assert_eq!(underside.symbol(), "▄");
    }

    #[test]
    fn small_window_still_renders() {
        let buf = render_area(10, 10);
        assert_eq!(buf.cell((0, 0)).unwrap().style().bg, Some(SPACE_BG));
    }
}
