use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{Frame, Terminal};

pub struct App;

impl App {
    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> std::io::Result<()> {
        loop {
            terminal.draw(Self::draw)?;
            if let Event::Key(key) = event::read()?
                && Self::should_quit(key)
            {
                return Ok(());
            }
        }
    }

    fn draw(frame: &mut Frame) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new("civterm - press q to quit"),
            frame.area(),
        );
    }

    fn should_quit(key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn pressing_q_marks_the_app_for_shutdown() {
        assert!(App::should_quit(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE
        )));
        assert!(App::should_quit(KeyEvent::new(
            KeyCode::Char('Q'),
            KeyModifiers::NONE
        )));
    }
}
