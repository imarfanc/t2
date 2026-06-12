//! Minimal static file server with pretty terminal output.
//! Settings live in server/config.json. Hotkeys: o = open in browser,
//! Ctrl+C / Ctrl+D / q = quit.

mod debug;

use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use debug::Diagnostics;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
#[serde(default)]
struct Config {
    port: u16,
    host: String,
    open_on_start: bool,
    log_requests: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 8008,
            host: "127.0.0.1".into(),
            open_on_start: false,
            log_requests: true,
        }
    }
}

fn load_config() -> Config {
    // config.json next to the crate (server/config.json), regardless of cwd.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            eprintln!(
                "{} bad config.json ({e}), using defaults",
                "!".yellow().bold()
            );
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

fn open_browser(url: &str) {
    let _ = Command::new("open").arg(url).spawn();
}

fn main() {
    let cfg = load_config();
    let diagnostics = Diagnostics::from_env();
    let root: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap());
    let root = root.canonicalize().expect("root dir not found");

    diagnostics.startup(&root, &cfg.host, cfg.port, cfg.log_requests);

    let listener = TcpListener::bind((cfg.host.as_str(), cfg.port)).unwrap_or_else(|e| {
        eprintln!(
            "{} cannot bind {}:{}: {}",
            "✗".red().bold(),
            cfg.host,
            cfg.port.to_string().yellow(),
            e
        );
        std::process::exit(1);
    });

    let url = format!("http://localhost:{}", cfg.port);
    banner(&root, &url);

    if cfg.open_on_start {
        open_browser(&url);
    }

    // Accept connections in a background thread.
    let log = cfg.log_requests;
    {
        let root = root.clone();
        thread::spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(stream) => {
                        let root = root.clone();
                        let connection_id = diagnostics.next_connection_id();
                        thread::spawn(move || {
                            handle(stream, &root, log, diagnostics, connection_id)
                        });
                    }
                    Err(error) => {
                        diagnostics.log(0, 0, &format!("accept error: {error}"));
                    }
                }
            }
        });
    }

    // Hotkey loop on the main thread (raw mode so single keys register).
    if terminal::enable_raw_mode().is_err() {
        // Not a TTY (e.g. piped) — just park forever.
        loop {
            thread::park();
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
                    open_browser(&url);
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

/// In raw mode '\n' doesn't return the carriage; use \r\n explicitly.
fn print_raw(s: &str) {
    let mut out = std::io::stdout();
    let _ = out.write_all(s.as_bytes());
    let _ = out.flush();
}

fn banner(root: &Path, url: &str) {
    use unicode_width::UnicodeWidthStr;

    let root_s = root.display().to_string();

    // (colored content, plain content for width measurement). None = separator.
    let rows: Vec<Option<(String, String)>> = vec![
        Some((
            format!("{}  {}", "🦀 repo-server".bold(), "ready".green().bold()),
            "🦀 repo-server  ready".into(),
        )),
        None,
        Some((
            format!("{} {}", "Serving".dimmed(), root_s.cyan()),
            format!("Serving {root_s}"),
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

    // Inner width = widest row + 2 spaces padding each side.
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
            Some((colored_s, plain)) => {
                let pad = " ".repeat(content_w - plain.as_str().width() + 2);
                println!(
                    "{}  {}{}{}",
                    "│".bright_black(),
                    colored_s,
                    pad,
                    "│".bright_black()
                );
            }
        }
    }
    println!("{}\n", format!("╰{line}╯").bright_black());
}

fn handle(
    mut stream: TcpStream,
    root: &Path,
    log_requests: bool,
    diagnostics: Diagnostics,
    connection_id: u64,
) {
    let peer = stream
        .peer_addr()
        .map(|address| address.to_string())
        .unwrap_or_else(|error| format!("unknown ({error})"));
    diagnostics.log(connection_id, 0, &format!("accepted peer={peer}"));

    // No Nagle: header+body must not sit in the kernel waiting for an ACK.
    if let Err(error) = stream.set_nodelay(true) {
        diagnostics.log(connection_id, 0, &format!("set_nodelay failed: {error}"));
    }
    // Generous idle timeout so keep-alive connections (incl. browser
    // preconnects) stay usable; browsers reuse idle sockets for ~60s+.
    if let Err(error) = stream.set_read_timeout(Some(Duration::from_secs(120))) {
        diagnostics.log(
            connection_id,
            0,
            &format!("set_read_timeout failed: {error}"),
        );
    }
    if let Err(error) = stream.set_write_timeout(Some(Duration::from_secs(5))) {
        diagnostics.log(
            connection_id,
            0,
            &format!("set_write_timeout failed: {error}"),
        );
    }

    let reader_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(error) => {
            diagnostics.log(connection_id, 0, &format!("socket clone failed: {error}"));
            return;
        }
    };
    let mut reader = BufReader::new(reader_stream);
    let mut request_id = 0_u64;

    // Keep-alive loop: serve multiple requests per connection.
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                diagnostics.log(connection_id, request_id, "client closed connection");
                return;
            }
            Err(error) => {
                diagnostics.log(
                    connection_id,
                    request_id,
                    &format!("request-line read ended: {error}"),
                );
                return;
            }
            Ok(_) => {}
        }
        request_id += 1;
        let start = Instant::now();
        let request_line = line.trim_end_matches(['\r', '\n']).to_string();
        let mut parts = line.split_whitespace();
        let method = parts.next().unwrap_or("GET").to_string();
        let raw_path = parts.next().unwrap_or("/");
        let path = raw_path.split(['?', '#']).next().unwrap_or("/").to_string();
        diagnostics.log(
            connection_id,
            request_id,
            &format!("request-line={request_line:?}"),
        );

        // Read remaining headers; honour the client's Connection preference.
        let mut client_close = false;
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    diagnostics.log(connection_id, request_id, "closed while reading headers");
                    return;
                }
                Err(error) => {
                    diagnostics.log(
                        connection_id,
                        request_id,
                        &format!("header read failed: {error}"),
                    );
                    return;
                }
                Ok(_) if line == "\r\n" || line == "\n" => break,
                Ok(_) => {
                    let lower = line.to_ascii_lowercase();
                    if lower.starts_with("connection:") && lower.contains("close") {
                        client_close = true;
                    }
                }
            }
        }

        let rel = Path::new(path.trim_start_matches('/'));
        let keep_alive = !client_close;
        let (status, response_bytes, resolved_path, response_result) = if rel
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        {
            let body = b"Forbidden";
            let result = respond(&mut stream, &method, 403, "text/plain", body, keep_alive);
            (403, body.len(), None, result)
        } else {
            let mut file = root.join(rel);
            if file.is_dir() {
                file = file.join("index.html");
            }
            match fs::read(&file) {
                Ok(body) => {
                    let body_len = body.len();
                    let result = respond(&mut stream, &method, 200, mime(&file), &body, keep_alive);
                    (200, body_len, Some(file), result)
                }
                Err(_) => {
                    let body = b"404 Not Found";
                    let result = respond(&mut stream, &method, 404, "text/plain", body, keep_alive);
                    (404, body.len(), Some(file), result)
                }
            }
        };
        let file = resolved_path
            .as_deref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".into());
        diagnostics.log(
            connection_id,
            request_id,
            &format!(
                "response status={status} method={method} path={path} file={file} body_bytes={response_bytes} keep_alive={keep_alive}"
            ),
        );
        if let Err(error) = response_result {
            diagnostics.log(
                connection_id,
                request_id,
                &format!("response write failed: {error}"),
            );
            return;
        }
        if log_requests {
            log(&method, &path, status, start.elapsed().as_micros());
        }
        if !keep_alive {
            diagnostics.log(connection_id, request_id, "closing at client request");
            return;
        }
    }
}

fn log(method: &str, path: &str, status: u16, micros: u128) {
    let status_str = match status {
        200 => status.to_string().green().bold(),
        403 => status.to_string().red().bold(),
        _ => status.to_string().yellow().bold(),
    };
    // \r\n because the terminal may be in raw mode.
    print_raw(&format!(
        "  {} {:6} {} {}\r\n",
        status_str,
        method.magenta(),
        path.white(),
        format!("({:.1} ms)", micros as f64 / 1000.0).dimmed()
    ));
}

fn respond(
    stream: &mut TcpStream,
    method: &str,
    status: u16,
    ctype: &str,
    body: &[u8],
    keep_alive: bool,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        _ => "Not Found",
    };
    let conn = if keep_alive { "keep-alive" } else { "close" };
    // HTML: always revalidate (instant back-nav still works via bfcache).
    // Assets: cache briefly so back/forward doesn't refetch everything.
    let cache = if ctype.starts_with("text/html") {
        "no-cache"
    } else {
        "max-age=60"
    };
    // Build the whole response in one buffer → one write, no Nagle/delayed-ACK stall.
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: {cache}\r\nConnection: {conn}\r\n\r\n",
        body.len()
    );
    let mut buf = Vec::with_capacity(header.len() + body.len());
    buf.extend_from_slice(header.as_bytes());
    if method != "HEAD" {
        buf.extend_from_slice(body);
    }
    stream.write_all(&buf)?;
    stream.flush()
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "md" | "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}
