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
        // Greedily pack commands into rows that each fit `area.width`,
        // wrapping onto the next row when a command would overflow. Each row
        // is then centred within the area.
        let mut rows: Vec<(Vec<&(&'a str, &'a str)>, usize)> = Vec::new();
        for cmd in self.commands {
            let width = cmd.0.len() + 1 + cmd.1.len();
            if let Some((items, used)) = rows.last_mut() {
                let with_gap = if *used == 0 {
                    width
                } else {
                    *used + GAP + width
                };
                if with_gap <= area.width as usize {
                    *used = with_gap;
                    items.push(cmd);
                    continue;
                }
            }
            rows.push((vec![cmd], width));
        }

        for (row, (items, _)) in rows.iter().enumerate().take(area.height as usize) {
            let row_total = items
                .iter()
                .map(|c| c.0.len() + 1 + c.1.len())
                .sum::<usize>()
                + GAP * items.len().saturating_sub(1);
            let mut x = area.x + area.width.saturating_sub(row_total as u16) / 2;
            let y = area.y + row as u16;
            for (i, (key, desc)) in items.iter().enumerate() {
                for (j, ch) in key.chars().enumerate() {
                    if x >= area.right() {
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((x, y)) {
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
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                        cell.set_symbol(&ch.to_string());
                        cell.set_bg(BAR_BG);
                        cell.set_style(Style::default().fg(DIM));
                    }
                    x += 1;
                }
                if i + 1 < items.len() {
                    for _ in 0..GAP {
                        if x >= area.right() {
                            break;
                        }
                        if let Some(cell) = buf.cell_mut((x, y)) {
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

    #[test]
    fn commands_wrap_across_two_rows_when_the_first_is_full() {
        // Not all commands fit on one 40-wide row, so the tail wraps onto row 1.
        let mut terminal = Terminal::new(TestBackend::new(40, 2)).unwrap();
        let commands = [
            ("arrows", "move"),
            ("y/u/b/n", "diag"),
            ("tab", "next unit"),
            ("space", "end turn"),
        ];
        terminal
            .draw(|frame| frame.render_widget(PlayingHelp::new(&commands), frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let row0: String = (0..40)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol().to_string())
            .collect();
        let row1: String = (0..40)
            .map(|x| buffer.cell((x, 1)).unwrap().symbol().to_string())
            .collect();
        assert!(row0.contains("arrows"));
        assert!(
            row1.contains("end turn"),
            "overflow did not wrap onto row 1: {row1:?}"
        );
    }
}
