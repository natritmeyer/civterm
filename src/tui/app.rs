use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};

use super::civ_selector::CivSelector;
use super::competition_selector::CompetitionSelector;
use super::difficulty_selector::DifficultySelector;
use super::splash::SplashScreen;
use super::status_bar::{ITEMS, StatusBar};
use crate::model::civilizations::Civilization;
use crate::model::competition::Competition;
use crate::model::difficulty::Difficulty;
use strum::IntoEnumIterator;

pub const STATUS_DELAY: Duration = Duration::from_secs(2);
pub const STATUS_FADE: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

enum Phase {
    Menu,
    ChoosingCiv,
    ChoosingCompetition,
    ChoosingDifficulty,
}

pub struct App {
    started_at: Instant,
    selected: usize,
    phase: Phase,
    civ_index: usize,
    chosen_civ: Option<Civilization>,
    competition_index: usize,
    chosen_competition: Option<Competition>,
    difficulty_index: usize,
    chosen_difficulty: Option<Difficulty>,
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
            competition_index: 0,
            chosen_competition: None,
            difficulty_index: 0,
            chosen_difficulty: None,
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
            Phase::ChoosingCompetition => frame.render_widget(
                CompetitionSelector::new(app.competition_index, app.chosen_competition),
                frame.area(),
            ),
            Phase::ChoosingDifficulty => frame.render_widget(
                DifficultySelector::new(app.difficulty_index, app.chosen_difficulty),
                frame.area(),
            ),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.phase {
            Phase::Menu => self.handle_menu_key(key),
            Phase::ChoosingCiv => self.handle_civ_key(key),
            Phase::ChoosingCompetition => self.handle_competition_key(key),
            Phase::ChoosingDifficulty => self.handle_difficulty_key(key),
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
                self.phase = Phase::ChoosingCompetition;
                self.competition_index = 0;
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::Menu;
                false
            }
            _ => false,
        }
    }

    fn handle_competition_key(&mut self, key: KeyEvent) -> bool {
        let levels = (Competition::MAX - Competition::MIN + 1) as usize;
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.competition_index = retreat(self.competition_index, levels);
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.competition_index = advance(self.competition_index, levels);
                false
            }
            KeyCode::Enter => {
                self.chosen_competition = Some(Competition::new(
                    self.competition_index as u8 + Competition::MIN,
                ));
                self.phase = Phase::ChoosingDifficulty;
                self.difficulty_index = 0;
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::ChoosingCiv;
                false
            }
            _ => false,
        }
    }

    fn handle_difficulty_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => {
                self.phase = Phase::Menu;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.difficulty_index = retreat(self.difficulty_index, Difficulty::iter().count());
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.difficulty_index = advance(self.difficulty_index, Difficulty::iter().count());
                false
            }
            KeyCode::Enter => {
                self.chosen_difficulty = Difficulty::iter().nth(self.difficulty_index);
                false
            }
            KeyCode::Esc => {
                self.phase = Phase::ChoosingCompetition;
                false
            }
            _ => false,
        }
    }

    fn start_new_game(&mut self) {
        self.phase = Phase::ChoosingCiv;
        self.civ_index = 0;
        self.chosen_civ = None;
        self.competition_index = 0;
        self.chosen_competition = None;
        self.difficulty_index = 0;
        self.chosen_difficulty = None;
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
    fn choosing_a_civ_stores_it_and_moves_to_competition() {
        let mut app = App::new();
        app.start_new_game();
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.chosen_civ, Some(Civilization::Babylonian));
        assert!(matches!(app.phase, Phase::ChoosingCompetition));
    }

    fn at_competition(app: &mut App) {
        app.start_new_game();
        app.handle_key(key(KeyCode::Enter)); // accept default civ, on to competition
    }

    fn at_difficulty(app: &mut App) {
        at_competition(app);
        app.handle_key(key(KeyCode::Enter)); // accept default competition, on to difficulty
    }

    #[test]
    fn competition_arrows_and_j_k_move_the_selection() {
        let mut app = App::new();
        at_competition(&mut app);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.competition_index, 1);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.competition_index, 2);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.competition_index, 1);
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.competition_index, 0);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(
            app.competition_index,
            (Competition::MAX - Competition::MIN) as usize
        );
    }

    #[test]
    fn choosing_a_competition_level_stores_it_and_moves_to_difficulty() {
        let mut app = App::new();
        at_competition(&mut app);
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.chosen_competition, Some(Competition::new(3)));
        assert!(matches!(app.phase, Phase::ChoosingDifficulty));
    }

    #[test]
    fn esc_from_competition_returns_to_the_civ_selector() {
        let mut app = App::new();
        at_competition(&mut app);
        app.handle_key(key(KeyCode::Esc));
        assert!(matches!(app.phase, Phase::ChoosingCiv));
        assert_eq!(app.chosen_civ, Some(Civilization::American));
    }

    #[test]
    fn q_from_competition_returns_to_the_menu() {
        for code in [KeyCode::Char('q'), KeyCode::Char('Q')] {
            let mut app = App::new();
            at_competition(&mut app);
            assert!(!app.handle_key(key(code)));
            assert!(matches!(app.phase, Phase::Menu));
        }
    }

    #[test]
    fn difficulty_arrows_and_j_k_move_the_selection() {
        let mut app = App::new();
        at_difficulty(&mut app);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.difficulty_index, 1);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.difficulty_index, 2);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.difficulty_index, 1);
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.difficulty_index, 0);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.difficulty_index, Difficulty::iter().count() - 1);
    }

    #[test]
    fn choosing_a_difficulty_stores_it() {
        let mut app = App::new();
        at_difficulty(&mut app);
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.chosen_difficulty, Some(Difficulty::Normal));
    }

    #[test]
    fn esc_from_difficulty_returns_to_the_competition_selector() {
        let mut app = App::new();
        at_difficulty(&mut app);
        app.handle_key(key(KeyCode::Esc));
        assert!(matches!(app.phase, Phase::ChoosingCompetition));
        assert_eq!(app.chosen_civ, Some(Civilization::American));
    }

    #[test]
    fn q_from_difficulty_returns_to_the_menu() {
        for code in [KeyCode::Char('q'), KeyCode::Char('Q')] {
            let mut app = App::new();
            at_difficulty(&mut app);
            assert!(!app.handle_key(key(code)));
            assert!(matches!(app.phase, Phase::Menu));
        }
    }

    #[test]
    fn starting_a_new_game_resets_setup() {
        let mut app = App::new();
        start_a_full_setup(&mut app);
        assert_eq!(app.chosen_competition, Some(Competition::new(1)));
        assert_eq!(app.chosen_difficulty, Some(Difficulty::Normal));
        app.start_new_game();
        assert_eq!(app.chosen_competition, None);
        assert_eq!(app.competition_index, 0);
        assert_eq!(app.chosen_difficulty, None);
        assert_eq!(app.difficulty_index, 0);
        assert_eq!(app.chosen_civ, None);
        assert!(matches!(app.phase, Phase::ChoosingCiv));
    }

    fn start_a_full_setup(app: &mut App) {
        at_difficulty(app);
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Enter));
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
