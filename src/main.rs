use std::io;

use civterm::crash_log;
use civterm::tui::App;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

fn main() -> io::Result<()> {
    use std::io::Write;

    // Before the terminal is touched, so a crash from here on leaves a report
    // behind and puts the terminal back the way it was found.
    crash_log::install();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    // Enable any-motion mouse tracking so hover events steer the movement
    // highlight. Terminals without support simply ignore the sequence.
    stdout.write_all(b"\x1b[?1003h")?;
    stdout.flush()?;
    crash_log::terminal_active();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = App::new().run(&mut terminal);

    // The same teardown the crash path uses, so the two cannot drift.
    crash_log::restore_terminal();
    result
}
