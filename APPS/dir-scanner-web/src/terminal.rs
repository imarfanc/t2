use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};
use std::process::Command;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub open_on_start: bool,
    pub log_requests: bool,
    pub ignore_dirs: String,
    pub ignore_files: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8011,
            host: "127.0.0.1".into(),
            open_on_start: false,
            log_requests: true,
            ignore_dirs: "target, builds".into(),
            ignore_files: "Thumbs.db, .DS_Store".into(),
        }
    }
}

pub fn load_config() -> Config {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|error| {
            eprintln!(
                "{} bad config.json ({error}), using defaults",
                "!".yellow().bold()
            );
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub fn url(cfg: &Config) -> String {
    format!("http://{}:{}", cfg.host, cfg.port)
}

pub fn open_browser(url: &str) {
    let _ = Command::new("open").arg(url).spawn();
}

pub fn banner(static_dir: &Path, url: &str) {
    let static_s = static_dir.display().to_string();
    let rows: Vec<Option<(String, String)>> = vec![
        Some((
            format!("{}  {}", "📂 dir-scanner-web".bold(), "ready".green().bold()),
            "📂 dir-scanner-web  ready".into(),
        )),
        None,
        Some((
            format!("{} {}", "Static ".dimmed(), static_s.cyan()),
            format!("Static  {static_s}"),
        )),
        Some((
            format!("{} {}", "Local  ".dimmed(), url.blue().bold().underline()),
            format!("Local   {url}"),
        )),
        Some((
            format!(
                "{} {}  {}  {}",
                "Keys   ".dimmed(),
                "o open browser".green(),
                "q quit".yellow(),
                "Ctrl+C / Ctrl+D quit".dimmed()
            ),
            "Keys    o open browser  q quit  Ctrl+C / Ctrl+D quit".into(),
        )),
    ];

    let content_w = rows
        .iter()
        .flatten()
        .map(|(_, plain)| plain.as_str().width())
        .max()
        .unwrap_or(0);
    let inner = content_w + 4;
    let line = "─".repeat(inner);

    println!("\n{}", format!("╭{line}╮").bright_black());
    for row in &rows {
        match row {
            None => println!("{}", format!("├{line}┤").bright_black()),
            Some((colored, plain)) => {
                let pad = " ".repeat(content_w - plain.as_str().width() + 2);
                println!(
                    "{}  {}{}{}",
                    "│".bright_black(),
                    colored,
                    pad,
                    "│".bright_black()
                );
            }
        }
    }
    println!("{}\n", format!("╰{line}╯").bright_black());
}

pub fn log_request(method: &str, path: &str, status: u16, micros: u128) {
    let status_str = match status {
        200 => status.to_string().green().bold(),
        400..=499 => status.to_string().yellow().bold(),
        _ => status.to_string().red().bold(),
    };
    print_raw(&format!(
        "  {} {:6} {} {}\r\n",
        status_str,
        method.magenta(),
        path.white(),
        format!("({:.1} ms)", micros as f64 / 1000.0).dimmed()
    ));
}

pub fn print_raw(message: &str) {
    let mut out = stdout();
    let _ = out.write_all(message.as_bytes());
    let _ = out.flush();
}

pub fn run_hotkeys(url: &str) -> ! {
    if terminal::enable_raw_mode().is_err() {
        loop {
            std::thread::park();
        }
    }

    loop {
        if let Ok(Event::Key(key)) = event::read() {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char('o') | KeyCode::Char('O') if !ctrl => {
                    print_raw(&format!(
                        "  {} opening {}\r\n",
                        "→".cyan(),
                        url.blue().underline()
                    ));
                    open_browser(url);
                }
                KeyCode::Char('c') | KeyCode::Char('d') if ctrl => shutdown(),
                KeyCode::Char('q') => shutdown(),
                _ => {}
            }
        }
    }
}

fn shutdown() -> ! {
    let _ = terminal::disable_raw_mode();
    println!("\n{} {}", "◉".red(), "Shutting down. Bye! 👋".dimmed());
    std::process::exit(0);
}

pub fn static_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}
