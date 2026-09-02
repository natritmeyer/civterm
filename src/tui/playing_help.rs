use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use super::theme::{ACCENT, DIM};

const BAR_BG: Color = Color::Rgb(20, 20, 48);
const GAP: usize = 3;

/// A bottom status bar listing the command keystrokes available in the
/// current playing context (e.g. while a unit is selected). Rendered only
/// while the player has toggled help with `?`.
pub struct PlayingHelp<'a> {
    commands: &'a [(&'a str, &'a str)],
}

impl<'a> PlayingHelp<'a> {
    pub fn new(commands: &'a [(&'a str, &'a str)]) -> Self {
        PlayingHelp { commands }
    }
}

impl<'a> Widget for PlayingHelp<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let total = self
            .commands
            .iter()
            .map(|(key, desc)| key.len() + 1 + desc.len())
            .sum::<usize>()
            + GAP * self.commands.len().saturating_sub(1);
        let mut x = area.x + area.width.saturating_sub(total as u16) / 2;
        for (i, (key, desc)) in self.commands.iter().enumerate() {
            for (j, ch) in key.chars().enumerate() {
                if x >= area.right() {
                    return;
                }
                if let Some(cell) = buf.cell_mut((x, area.y)) {
                    cell.reset();
                    cell.set_symbol(&ch.to_string());
                    cell.set_bg(BAR_BG);
                    let mut style = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);
                    if j == 0 {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    cell.set_style(style);
                }
                x += 1;
            }
            x += 1;
            for ch in desc.chars() {
                if x >= area.right() {
                    return;
                }
                if let Some(cell) = buf.cell_mut((x, area.y)) {
                    cell.reset();
                    cell.set_symbol(&ch.to_string());
                    cell.set_bg(BAR_BG);
                    cell.set_style(Style::default().fg(DIM));
                }
                x += 1;
            }
            if i + 1 < self.commands.len() {
                for _ in 0..GAP {
                    if x >= area.right() {
                        return;
                    }
                    if let Some(cell) = buf.cell_mut((x, area.y)) {
                        cell.reset();
                        cell.set_symbol(" ");
                        cell.set_bg(BAR_BG);
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

    fn render(commands: &[(&str, &str)]) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(PlayingHelp::new(commands), frame.area()))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn keys_and_descriptions_are_drawn() {
        let buf = render(&[("m", "move"), ("f", "fortify")]);
        let text: String = (0..60)
            .map(|x| buf.cell((x, 0)).unwrap().symbol().to_string())
            .collect();
        assert!(text.contains("m move"));
        assert!(text.contains("f fortify"));
    }

    #[test]
    fn key_letters_are_accented_and_bold() {
        let buf = render(&[("f", "fortify")]);
        let text: String = (0..60)
            .map(|x| buf.cell((x, 0)).unwrap().symbol().to_string())
            .collect();
        let key_x = text.find('f').unwrap() as u16;
        let cell = buf.cell((key_x, 0)).unwrap();
        assert_eq!(cell.style().fg, Some(ACCENT));
        assert!(cell.style().add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn empty_help_renders_without_panicking() {
        let _buf = render(&[]);
    }
}
