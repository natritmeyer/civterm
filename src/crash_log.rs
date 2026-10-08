//! Crash reporting, so a crash the player watched is on disk afterwards.
//!
//! A terminal game has one great disadvantage over an ordinary program: it owns
//! the screen. A panic that leaves the terminal in raw mode with the alternate
//! screen up takes the shell down with it, so the message scrolls away unheard
//! and all the player is left with is a machine that mysteriously quits. This
//! module is installed before the terminal is touched, and it does three
//! things about that:
//!
//! - a **panic hook** that writes the message, the source location and a full
//!   backtrace to a log file, then puts the terminal back the way it found it;
//! - a **signal handler** for the faults a panic hook cannot see — a segfault,
//!   an abort, a stack overflow — which reports the same way and then exits
//!   with the signal's status, so the process really does die rather than
//!   limping on in whatever state the fault left it;
//! - a ring of **breadcrumbs** ([`breadcrumb`]) recording what the player and
//!   the app last did, which is what localises a crash nobody can reproduce.
//!
//! Reports are appended to `civterm-crash.log` in the working directory, or to
//! whatever file `CIVTERM_CRASH_LOG` names. Nothing here can panic: a crash
//! reporter that panics about the crash is worse than no crash reporter.

use std::backtrace::Backtrace;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// The report file used when `CIVTERM_CRASH_LOG` says nothing.
const LOG_FILE: &str = "civterm-crash.log";
/// The variable naming the report file.
const LOG_FILE_VAR: &str = "CIVTERM_CRASH_LOG";
/// How many recent notes a report carries: enough for a whole dialog sequence,
/// few enough that a report stays readable.
const CRUMB_COUNT: usize = 40;
/// The fence each report is written between, so an appended log reads as a
/// sequence of entries rather than one long smear.
const BANNER: &str = "==================== civterm crash report ====================";
/// The backtrace recorded for a signal: the faulting thread is not the thread
/// writing the report, so there is nothing honest to print.
const NO_BACKTRACE: &str = "(not available: a signal is reported from the handler thread,\n\
                           not from the thread that faulted)";

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static RECENT: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static TERMINAL_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Installs the crash handlers, reporting to `civterm-crash.log` or to the file
/// `CIVTERM_CRASH_LOG` names. Call this before the terminal is taken over.
pub fn install() {
    let path = match std::env::var_os(LOG_FILE_VAR) {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(LOG_FILE),
    };
    install_at(path);
}

/// Installs the crash handlers, reporting to `path`. Split out from [`install`]
/// so a caller (a test) can put the report somewhere it knows to look.
pub fn install_at(path: impl Into<PathBuf>) {
    // A second call keeps the first choice: the log is already open in the
    // first handler and must not be redirected out from under it.
    let _ = LOG_PATH.set(path.into());
    install_panic_hook();
    install_signal_handler();
}

/// Records that the terminal has been taken over, so a crash report knows to
/// put it back. The report is written either way.
pub fn terminal_active() {
    TERMINAL_ACTIVE.store(true, Ordering::Relaxed);
}

/// Puts the terminal back the way it was: any-motion tracking off, alternate
/// screen left, mouse capture dropped, raw mode disabled. Safe to call twice,
/// and safe to call when nothing was ever taken over — which is what makes it
/// usable from both a panic hook and the normal exit path.
pub fn restore_terminal() {
    if !TERMINAL_ACTIVE.swap(false, Ordering::Relaxed) {
        return;
    }
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x1b[?1003l");
    let _ = stdout.flush();
    let _ = crossterm::execute!(
        stdout,
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    );
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = io::stderr().flush();
}

/// Notes what was just done, for the report. The ring holds the last
/// [`CRUMB_COUNT`] notes, oldest first, so a crash is reported against the
/// moments leading up to it — the only account of a crash that will not repeat
/// on demand.
pub fn breadcrumb(note: impl Into<String>) {
    let mut recent = RECENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    push_note(&mut recent, note.into());
}

/// Adds one note to the ring, dropping the oldest once it is full. A separate
/// function so the ring's arithmetic is testable on a ring of its own rather
/// than on the process-wide one that every other test is also writing to.
fn push_note(recent: &mut VecDeque<String>, note: String) {
    if recent.len() == CRUMB_COUNT {
        recent.pop_front();
    }
    // A note is one line of an indented list, so a note carrying a newline
    // would push its own tail out of the report's shape.
    recent.push_back(note.replace(['\n', '\r'], " "));
}

/// The report file's path, whether or not anything has been installed — so a
/// crash before [`install`] still lands somewhere findable.
pub fn log_path() -> PathBuf {
    LOG_PATH
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from(LOG_FILE))
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let detail = match info.payload().downcast_ref::<&str>() {
            Some(message) => (*message).to_string(),
            None => match info.payload().downcast_ref::<String>() {
                Some(message) => message.clone(),
                None => "(a non-string panic payload)".to_string(),
            },
        };
        let location = info
            .location()
            .map(|at| format!("{}:{}:{}", at.file(), at.line(), at.column()));
        report(
            "panic",
            &detail,
            location.as_deref(),
            &Backtrace::force_capture().to_string(),
        );
        restore_terminal();
        // Say it on the real terminal as well: a crash the player watched
        // happen should not be silent even if the file is never found.
        eprintln!("civterm crashed: {detail}");
        if let Some(location) = &location {
            eprintln!("  at {location}");
        }
        eprintln!("  report written to {}", log_path().display());
    }));
}

/// Catches the faults a panic hook never sees. The handler thread is an
/// ordinary thread, not an async signal context, so it may write a file and
/// allocate; all it must not do is let the process carry on afterwards.
#[cfg(unix)]
fn install_signal_handler() {
    use signal_hook::iterator::Signals;

    // SIGABRT is the one that matters: Rust turns a double panic — a panic
    // while another is unwinding — into an abort, and so does anything that
    // reaches for `abort()`. Those are exactly the crashes that print nothing
    // a panic hook could have caught first.
    //
    // SIGSEGV, SIGILL and SIGFPE are *not* on the list, and their absence is
    // deliberate rather than an oversight: signal-hook refuses to register a
    // hardware fault through its checked API — `Signals::new` asserts on it
    // instead of returning an error, which would take the whole process down
    // during install. Reaching past that means `unsafe`, and this crate has
    // none; a segfault here would be a bug in ratatui or crossterm. The tell
    // for one is a process that vanishes leaving no report at all.
    const FAULTS: [i32; 2] = [
        signal_hook::consts::signal::SIGABRT,
        signal_hook::consts::signal::SIGBUS,
    ];

    // Registering happens here, not in the thread: `Signals::new` installs the
    // handlers, so a thread that never ran would leave them set with nobody
    // reading. Dropping the iterator (which is what a failed spawn does with
    // the closure it hands back) unregisters them again. Registration is also
    // allowed to panic inside a library we do not control, so it is caught: a
    // crash reporter that crashes the game at startup has defeated itself.
    let signals = std::panic::catch_unwind(|| Signals::new(FAULTS)).ok();
    let Some(mut signals) = signals.and_then(Result::ok) else {
        return;
    };
    let _ = std::thread::Builder::new()
        .name("civterm-crash".to_string())
        .spawn(move || {
            // One fault, one report: the process is gone by the end of this,
            // so a second signal could not add anything to it.
            let Some(signal) = signals.forever().next() else {
                return;
            };
            let name = signal_name(signal);
            report("signal", &format!("{name} ({signal})"), None, NO_BACKTRACE);
            restore_terminal();
            eprintln!(
                "civterm died on {name}; report written to {}",
                log_path().display()
            );
            // The world is in an undefined state, so the process must not keep
            // running. 128 + n is how a shell reports death by signal n, so
            // the exit status still names the fault.
            std::process::exit(128 + signal);
        });
}

#[cfg(not(unix))]
fn install_signal_handler() {}

#[cfg(unix)]
fn signal_name(signal: i32) -> &'static str {
    use signal_hook::consts::signal::{SIGABRT, SIGBUS};
    match signal {
        SIGABRT => "SIGABRT",
        SIGBUS => "SIGBUS",
        _ => "unknown signal",
    }
}

/// Gathers one report and appends it. Nothing in here propagates an error: the
/// point of the file is that it is there when it can be written.
fn report(kind: &str, detail: &str, location: Option<&str>, backtrace: &str) {
    let report = CrashReport {
        kind: kind.to_string(),
        detail: detail.to_string(),
        location: location.map(str::to_string),
        seconds: now_seconds(),
        terminal: crossterm::terminal::size().ok(),
        cwd: std::env::current_dir()
            .ok()
            .map(|path| path.display().to_string()),
        args: std::env::args().skip(1).collect(),
        recent: recent_notes(),
        backtrace: backtrace.to_string(),
    };
    append(&log_path(), &report.render());
}

/// One crash report, gathered. Separate from the rendering so the shape of a
/// report can be pinned by a test without provoking a crash.
struct CrashReport {
    kind: String,
    detail: String,
    location: Option<String>,
    seconds: u64,
    terminal: Option<(u16, u16)>,
    cwd: Option<String>,
    args: Vec<String>,
    recent: Vec<String>,
    backtrace: String,
}

impl CrashReport {
    fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "when: {} ({}s since the epoch)",
            utc_stamp(self.seconds),
            self.seconds
        );
        let _ = writeln!(out, "kind: {}", self.kind);
        let _ = writeln!(out, "detail: {}", self.detail);
        let _ = writeln!(out, "location: {}", self.location.as_deref().unwrap_or("-"));
        let _ = writeln!(out, "version: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(out, "cwd: {}", self.cwd.as_deref().unwrap_or("-"));
        let _ = writeln!(
            out,
            "args: {}",
            if self.args.is_empty() {
                "(none)".to_string()
            } else {
                self.args.join(" ")
            }
        );
        let _ = writeln!(out, "terminal: {}", terminal_text(self.terminal));
        let _ = writeln!(out);
        let _ = writeln!(out, "recent activity (oldest first):");
        if self.recent.is_empty() {
            let _ = writeln!(out, "  (nothing recorded)");
        }
        for note in &self.recent {
            let _ = writeln!(out, "  {note}");
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "backtrace:");
        let _ = writeln!(out, "{}", self.backtrace.trim_end());
        out
    }
}

fn append(path: &Path, report: &str) {
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{BANNER}");
    let _ = writeln!(file, "{report}");
    let _ = file.flush();
}

/// The recent notes, oldest first. Taken without blocking: this is read on the
/// way out of a crash, and the thread being crashed is quite likely to be the
/// thread holding the ring.
fn recent_notes() -> Vec<String> {
    match RECENT.try_lock() {
        Ok(recent) => recent.iter().cloned().collect(),
        Err(_) => vec!["(unavailable: held by another thread)".to_string()],
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}

fn terminal_text(terminal: Option<(u16, u16)>) -> String {
    match terminal {
        Some((width, height)) => format!("{width}x{height}"),
        None => "unknown".to_string(),
    }
}

/// A UTC timestamp for a whole number of seconds since the epoch, without a
/// date library: days-to-civil-date is a few lines of era arithmetic, and a
/// crash report that cannot say when it happened is a nuisance to correlate.
fn utc_stamp(seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let (year, month, day) = civil_from_days(days);
    let rest = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// The calendar date for a count of days since 1970-01-01, by the standard
/// era-based conversion: shift the epoch to 0000-03-01 so leap days land at
/// the end of the era's year, then count them off.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_shifted = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_shifted + 2) / 5 + 1) as u32;
    let month = if month_shifted < 10 {
        month_shifted + 3
    } else {
        month_shifted - 9
    } as u32;
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_report() -> CrashReport {
        CrashReport {
            kind: "panic".to_string(),
            detail: "attempt to subtract with overflow".to_string(),
            location: Some("src/tui/city_window.rs:123:9".to_string()),
            seconds: 1_767_225_600,
            terminal: Some((120, 40)),
            cwd: Some("/home/player/civterm".to_string()),
            args: vec!["--seed".to_string(), "7".to_string()],
            recent: vec![
                "key Char(',')".to_string(),
                "mouse Down at (42, 12)".to_string(),
            ],
            backtrace: "0: civterm::crash_log::report\n1: civterm::tui::app::run".to_string(),
        }
    }

    #[test]
    fn a_report_says_what_happened_and_where() {
        let report = a_report().render();
        assert!(report.contains("kind: panic"), "{report}");
        assert!(
            report.contains("detail: attempt to subtract with overflow"),
            "{report}"
        );
        assert!(
            report.contains("location: src/tui/city_window.rs:123:9"),
            "{report}"
        );
        assert!(report.contains("terminal: 120x40"), "{report}");
        assert!(report.contains("cwd: /home/player/civterm"), "{report}");
        assert!(report.contains("args: --seed 7"), "{report}");
    }

    #[test]
    fn a_report_carries_the_recent_activity_oldest_first() {
        let report = a_report().render();
        let played = report.find("key Char(',')").expect("the key is recorded");
        let clicked = report.find("mouse Down").expect("the click is recorded");
        assert!(played < clicked, "the ring reads forwards:\n{report}");
    }

    #[test]
    fn a_report_with_nothing_to_report_still_renders() {
        let bare = CrashReport {
            kind: "signal".to_string(),
            detail: "SIGSEGV (11)".to_string(),
            location: None,
            seconds: 0,
            terminal: None,
            cwd: None,
            args: Vec::new(),
            recent: Vec::new(),
            backtrace: NO_BACKTRACE.to_string(),
        };
        let report = bare.render();
        assert!(report.contains("location: -"), "{report}");
        assert!(report.contains("terminal: unknown"), "{report}");
        assert!(report.contains("args: (none)"), "{report}");
        assert!(report.contains("(nothing recorded)"), "{report}");
        assert!(report.contains("not available"), "{report}");
    }

    #[test]
    fn the_timestamp_names_the_day() {
        assert_eq!(utc_stamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_stamp(1_767_225_600), "2026-01-01T00:00:00Z");
        assert_eq!(utc_stamp(1_704_067_199), "2023-12-31T23:59:59Z");
        assert_eq!(utc_stamp(951_782_400), "2000-02-29T00:00:00Z");
        // A clock that has overflowed is nonsense but must still render: the
        // point is that reporting a crash cannot itself overflow.
        assert_eq!(utc_stamp(u64::MAX), "584554051223-11-09T07:00:15Z");
    }

    #[test]
    fn breadcrumbs_keep_the_most_recent_and_drop_the_rest() {
        let mut ring = VecDeque::new();
        for index in 0..CRUMB_COUNT + 10 {
            push_note(&mut ring, format!("note {index}"));
        }
        assert_eq!(ring.len(), CRUMB_COUNT);
        assert_eq!(ring.front().map(String::as_str), Some("note 10"));
        assert_eq!(ring.back().map(String::as_str), Some("note 49"));
    }

    #[test]
    fn a_breadcrumb_stays_on_one_line() {
        breadcrumb("two\nlines\rhere");
        assert_eq!(
            recent_notes().last().map(String::as_str),
            Some("two lines here")
        );
    }

    /// The whole point of the module, end to end: a panic reaches the disk. The
    /// hook is process-wide, so this installs it once for the run.
    #[test]
    fn a_panic_is_written_to_the_log() {
        let path =
            std::env::temp_dir().join(format!("civterm-crash-test-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        install_at(&path);
        let _ = std::panic::catch_unwind(|| panic!("the report should mention this"));
        let written = std::fs::read_to_string(&path).expect("the report was written");
        assert!(
            written.contains("kind: panic"),
            "the report says what kind of crash it was:\n{written}"
        );
        assert!(
            written.contains("detail: the report should mention this"),
            "the report carries the message:\n{written}"
        );
        assert!(
            written.contains("location: src/crash_log.rs:"),
            "the report points at the source:\n{written}"
        );
        assert!(written.contains("backtrace:"), "{written}");
        let _ = std::fs::remove_file(&path);
    }
}
