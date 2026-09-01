use civterm::tui::splash::SplashScreen;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn main() {
    let mut terminal = Terminal::new(TestBackend::new(100, 60)).unwrap();
    terminal.draw(|frame| frame.render_widget(SplashScreen::new(), frame.area())).unwrap();
    let buf = terminal.backend().buffer();
    let gy = 60 / 2 - (61 / 2);
    for y in 26..=33 {
        let line: String = (0..100u16)
            .map(|x| buf.cell((x, y)).map(|c| c.symbol().chars().next().unwrap_or(' ')).unwrap_or(' '))
            .collect();
        println!("{line}");
    }
}
