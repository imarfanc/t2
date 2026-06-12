#!/bin/sh
//bin/sh -c 'ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd); BIN="$ROOT/builds/test"; mkdir -p "$ROOT/builds"; [ "$0" -nt "$BIN" ] && rustc "$0" -o "$BIN"; exec "$BIN" "$@"' "$0" "$@"; exit $?

// Single-file Rust TUI (std only, no Cargo.toml / Cargo.lock required).
// Make it executable once: chmod +x scripts/test-tui/test.rs
// Then run it directly or with: just test-tui
//
// The launcher recompiles changed source to builds/test.

use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::Command;

/// Content lives in scripts/test-tui/content.md and is read at runtime,
/// so editing it needs no recompile. The binary runs from ROOT/builds/test,
/// so ROOT is the executable's grandparent directory.
fn load_markdown() -> String {
    let from_exe = std::env::current_exe().ok().and_then(|exe| {
        let root = exe.parent()?.parent()?.to_path_buf();
        std::fs::read_to_string(root.join("scripts/test-tui/content.md")).ok()
    });
    from_exe
        .or_else(|| std::fs::read_to_string(PathBuf::from("scripts/test-tui/content.md")).ok())
        .unwrap_or_else(|| "# Markdown TUI\n\ncontent.md not found.\n".into())
}

fn main() -> io::Result<()> {
    let markdown = load_markdown();
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    let _terminal = interactive.then(TerminalGuard::enter);

    let mut stdout = io::stdout().lock();
    if interactive {
        write!(stdout, "\x1b[2J\x1b[H\x1b[?25l")?;
    }

    // In raw mode '\n' no longer implies a carriage return, so every
    // interactive line must end with "\r\n" or output drifts rightward.
    let eol = if interactive { "\r\n" } else { "\n" };

    render_markdown(&mut stdout, &markdown, interactive, eol)?;

    if interactive {
        write!(
            stdout,
            "{eol}\x1b[2m  q quit  \x1b[2m|\x1b[2m  Ctrl-C quit\x1b[0m{eol}"
        )?;
        stdout.flush()?;

        for byte in io::stdin().lock().bytes() {
            if matches!(byte?, b'q' | b'Q' | 3) {
                break;
            }
        }
    }

    Ok(())
}

fn render_markdown(out: &mut impl Write, markdown: &str, color: bool, eol: &str) -> io::Result<()> {

    for line in markdown.lines() {
        let rendered = if let Some(text) = line.strip_prefix("# ") {
            format!("\x1b[1;36m {}\x1b[0m", inline(text, color))
        } else if let Some(text) = line.strip_prefix("## ") {
            format!("\x1b[1;34m  {}\x1b[0m", inline(text, color))
        } else if let Some(text) = line.strip_prefix("- ") {
            format!("    \x1b[36m•\x1b[0m {}", inline(text, color))
        } else if let Some(text) = line.strip_prefix("> ") {
            format!("  \x1b[33m│\x1b[0m \x1b[3m{}\x1b[0m", inline(text, color))
        } else if line.is_empty() {
            String::new()
        } else {
            format!("  {}", inline(line, color))
        };

        if color {
            write!(out, "{rendered}{eol}")?;
        } else {
            write!(out, "{}{eol}", strip_ansi(&rendered))?;
        }
    }

    Ok(())
}

fn inline(text: &str, color: bool) -> String {
    if !color {
        return text.replace("**", "").replace('`', "");
    }

    let mut result = String::new();
    let mut rest = text;
    let mut bold = false;
    let mut code = false;

    while !rest.is_empty() {
        if let Some(next) = rest.strip_prefix("**") {
            bold = !bold;
            result.push_str(if bold { "\x1b[1m" } else { "\x1b[22m" });
            rest = next;
        } else if let Some(next) = rest.strip_prefix('`') {
            code = !code;
            result.push_str(if code {
                "\x1b[38;5;215m"
            } else {
                "\x1b[39m"
            });
            rest = next;
        } else {
            let ch = rest.chars().next().expect("rest is not empty");
            result.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }

    result
}

fn strip_ansi(text: &str) -> String {
    let mut clean = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            clean.push(ch);
        }
    }

    clean
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Self {
        let _ = Command::new("stty").args(["raw", "-echo"]).status();
        Self
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = Command::new("stty").args(["sane"]).status();
        print!("\x1b[0m\x1b[?25h\n");
        let _ = io::stdout().flush();
    }
}
