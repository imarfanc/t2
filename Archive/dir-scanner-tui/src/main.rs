mod app;
mod report;
mod scan;

use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use crate::scan::{real_root, scan_tree, ScanOptions, DEFAULT_SCAN_ROOT};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

fn main() -> io::Result<()> {
    let root = scan_root_from_env()?;
    let mut terminal = setup_terminal()?;
    let tick_rate = Duration::from_millis(80);

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let options = ScanOptions::with_defaults();
        let result = real_root(&root).map(|path| scan_tree(&path, &options));
        let _ = tx.send(result);
    });

    let mut app = App::new_scanning();
    let mut scan_done = false;

    loop {
        terminal.draw(|frame| app::render(frame, &app))?;

        if !scan_done {
            if let Ok(result) = rx.try_recv() {
                scan_done = true;
                match result {
                    Ok(data) => app.set_data(data),
                    Err(err) => app.set_error(err.to_string()),
                }
            } else {
                app.tick();
            }
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char('s') | KeyCode::Char('S')
                        if matches!(app.mode, app::AppMode::Ready) =>
                    {
                        app.save();
                    }
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('j') | KeyCode::Down => app.scroll_down(1),
                    KeyCode::Char('k') | KeyCode::Up => app.scroll_up(1),
                    KeyCode::PageDown => app.scroll_down(10),
                    KeyCode::PageUp => app.scroll_up(10),
                    KeyCode::Char(c @ '1'..='7') => {
                        let index = (c as u8 - b'1') as usize;
                        if let Some(tab) = app::Tab::ALL.get(index) {
                            app.tab = *tab;
                            app.scroll = 0;
                        }
                    }
                    _ if key.modifiers.contains(KeyModifiers::SHIFT)
                        && matches!(key.code, KeyCode::Tab) =>
                    {
                        app.prev_tab();
                    }
                    _ => {}
                }
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn scan_root_from_env() -> io::Result<PathBuf> {
    let arg = env::args()
        .nth(1)
        .or_else(|| env::var("SCAN_ROOT").ok())
        .unwrap_or_else(|| DEFAULT_SCAN_ROOT.into());
    Ok(PathBuf::from(arg))
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
