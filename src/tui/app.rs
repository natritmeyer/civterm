use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::civ_selector::CivSelector;
use super::splash::SplashScreen;
use super::status_bar::{ITEMS, StatusBar};
use crate::model::civilizations::Civilization;
use strum::IntoEnumIterator;

pub const STATUS_DELAY: Duration = Duration::from_secs(2);
pub const STATUS_FADE: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

enum Phase {
    Menu,
    ChoosingCiv,
}

pub struct App {
    started_at: Instant,
    selected: usize,
    phase: Phase,
    civ_index: usize,
    chosen_civ: Option<Civilization>,
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
            phase: Phase::Menu,
            civ_index: 0,
            chosen_civ: None,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| Self::draw(frame, self))?;
            if event::poll(POLL_INTERVAL)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key(key)
            {
                return Ok(());
            }
        }
    }

    fn draw(frame: &mut Frame, app: &App) {
        match app.phase {
            Phase::Menu => {
                frame.render_widget(SplashScreen::new(), frame.area());
                let progress = fade_progress(app.started_at.elapsed());
                if progress > 0.0 {
                    let area = Rect {
                        x: frame.area().x,
                        y: frame.area().bottom().saturating_sub(1),
                        width: frame.area().width,
                        height: 1,
                    };
                    frame.render_widget(StatusBar::new(progress, app.selected), area);
                }
            }
            Phase::ChoosingCiv => frame.render_widget(
                CivSelector::new(app.civ_index, app.chosen_civ),
                frame.area(),
            ),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.phase {
            Phase::Menu => self.handle_menu_key(key),
            Phase::ChoosingCiv => self.handle_civ_key(key),
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'q' => true,
                'n' => {
                    self.start_new_game();
                    false
                }
                'l' => {
                    self.selected = 1;
                    false
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
            KeyCode::Enter => match self.selected {
                0 => {
                    self.start_new_game();
                    false
                }
                2 => true,
                _ => false,
            },
            KeyCode::Esc => true,
            _ => false,
        }
    }

    fn handle_civ_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.civ_index = retreat(self.civ_index, Civilization::iter().count());
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.civ_index = advance(self.civ_index, Civilization::iter().count());
                false
            }
            KeyCode::Enter => {
                self.chosen_civ = Civilization::iter().nth(self.civ_index);
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::Menu;
                false
            }
            _ => false,
        }
    }

    fn start_new_game(&mut self) {
        self.phase = Phase::ChoosingCiv;
        self.civ_index = 0;
        self.chosen_civ = None;
    }
}

fn advance(selected: usize, total: usize) -> usize {
    (selected + 1) % total
}

fn retreat(selected: usize, total: usize) -> usize {
    (selected + total - 1) % total
}

fn fade_progress(elapsed: Duration) -> f32 {
    let after_delay = elapsed.saturating_sub(STATUS_DELAY);
    (after_delay.as_secs_f32() / STATUS_FADE.as_secs_f32()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn pressing_q_in_the_menu_exits() {
        for code in [KeyCode::Char('q'), KeyCode::Char('Q'), KeyCode::Esc] {
            let mut app = App::new();
            assert!(app.handle_key(key(code)));
        }
    }

    #[test]
    fn new_game_opens_the_civ_selector() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Char('n')));
        assert!(matches!(app.phase, Phase::ChoosingCiv));
        let mut app = App::new();
        app.handle_key(key(KeyCode::Enter));
        assert!(matches!(app.phase, Phase::ChoosingCiv));
    }

    #[test]
    fn load_only_highlights_the_item() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Char('l')));
        assert_eq!(app.selected, 1);
        assert!(matches!(app.phase, Phase::Menu));
    }

    #[test]
    fn arrows_move_the_menu_selection() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Right));
        assert_eq!(app.selected, 1);
        app.handle_key(key(KeyCode::Right));
        assert_eq!(app.selected, 2);
        app.handle_key(key(KeyCode::Right));
        assert_eq!(app.selected, 0);
        app.handle_key(key(KeyCode::Left));
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn enter_on_quit_exits() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Right));
        app.handle_key(key(KeyCode::Right));
        assert!(app.handle_key(key(KeyCode::Enter)));
    }

    #[test]
    fn civ_arrows_and_j_k_move_the_selection() {
        let mut app = App::new();
        app.start_new_game();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.civ_index, 1);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.civ_index, 2);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.civ_index, 1);
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.civ_index, 0);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.civ_index, Civilization::iter().count() - 1);
    }

    #[test]
    fn choosing_a_civ_stores_it() {
        let mut app = App::new();
        app.start_new_game();
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.chosen_civ, Some(Civilization::Babylonian));
    }

    #[test]
    fn esc_and_q_return_to_the_menu() {
        for code in [KeyCode::Esc, KeyCode::Char('q')] {
            let mut app = App::new();
            app.start_new_game();
            assert!(!app.handle_key(key(code)));
            assert!(matches!(app.phase, Phase::Menu));
        }
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
}
