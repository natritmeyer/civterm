use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::splash::SplashScreen;
use super::status_bar::{ITEMS, StatusBar};

pub const STATUS_DELAY: Duration = Duration::from_secs(2);
pub const STATUS_FADE: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub struct App {
    started_at: Instant,
    selected: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            selected: 0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| Self::draw(frame, self.started_at, self.selected))?;
            if event::poll(POLL_INTERVAL)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key(key)
            {
                return Ok(());
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'q' => true,
                'n' => {
                    self.selected = 0;
                    activate(self.selected)
                }
                'l' => {
                    self.selected = 1;
                    activate(self.selected)
                }
                _ => false,
            },
            KeyCode::Right => {
                self.selected = advance(self.selected, ITEMS.len());
                false
            }
            KeyCode::Left => {
                self.selected = retreat(self.selected, ITEMS.len());
                false
            }
            KeyCode::Enter => activate(self.selected),
            KeyCode::Esc => true,
            _ => false,
        }
    }

    fn draw(frame: &mut Frame, started_at: Instant, selected: usize) {
        frame.render_widget(SplashScreen::new(), frame.area());

        let progress = fade_progress(started_at.elapsed());
        if progress > 0.0 {
            let area = Rect {
                x: frame.area().x,
                y: frame.area().bottom().saturating_sub(1),
                width: frame.area().width,
                height: 1,
            };
            frame.render_widget(StatusBar::new(progress, selected), area);
        }
    }
}

fn advance(selected: usize, total: usize) -> usize {
    (selected + 1) % total
}

fn retreat(selected: usize, total: usize) -> usize {
    (selected + total - 1) % total
}

fn activate(selected: usize) -> bool {
    selected == ITEMS.len() - 1
}

fn fade_progress(elapsed: Duration) -> f32 {
    let after_delay = elapsed.saturating_sub(STATUS_DELAY);
    (after_delay.as_secs_f32() / STATUS_FADE.as_secs_f32()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn pressing_q_marks_the_app_for_shutdown() {
        let mut app = App::new();
        assert!(app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)));
        let mut app = App::new();
        assert!(app.handle_key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE)));
        let mut app = App::new();
        assert!(app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    }

    #[test]
    fn arrows_move_the_selection() {
        let mut app = App::new();
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.selected, 1);
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.selected, 2);
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.selected, 0);
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.selected, 2);
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn mnemonic_keys_jump_to_their_item() {
        let mut app = App::new();
        app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        assert_eq!(app.selected, 0);
        app.handle_key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE));
        assert_eq!(app.selected, 1);
        app.handle_key(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE));
        assert_eq!(app.selected, 0);
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn enter_runs_only_the_quit_action() {
        let mut app = App::new();
        assert!(!app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert!(app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    }

    #[test]
    fn status_bar_hides_for_two_seconds() {
        assert_eq!(fade_progress(Duration::from_secs(1)), 0.0);
        assert_eq!(fade_progress(Duration::from_secs(2)), 0.0);
        assert_eq!(fade_progress(Duration::from_millis(2500)), 0.5);
    }

    #[test]
    fn status_bar_finishes_fading_after_three_seconds() {
        assert_eq!(fade_progress(Duration::from_secs(3)), 1.0);
        assert_eq!(fade_progress(Duration::from_secs(10)), 1.0);
    }

    #[test]
    fn selection_wraps_around_the_menu() {
        assert_eq!(advance(0, 3), 1);
        assert_eq!(advance(2, 3), 0);
        assert_eq!(retreat(0, 3), 2);
        assert_eq!(retreat(1, 3), 0);
    }

    #[test]
    fn only_quit_activates() {
        assert!(!activate(0));
        assert!(!activate(1));
        assert!(activate(2));
    }
}
