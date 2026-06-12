---
title: "rust"
sort: 4
category: "macOS reset"
description: "install rust"
date: 2026-6-12
tags:
    - macOS
    - reset
    - rust
    - install
    - rustup
---

# install rust

## using curl

The official Rust installer is `rustup`. Paste the complete heredoc below into
zsh so comments and shell metacharacters are handled safely.

Paste this whole thing:

```sh
zsh <<'ZSH'
set -euo pipefail
setopt interactivecomments 2>/dev/null || true

bold=$'\033[1m'
green=$'\033[32m'
yellow=$'\033[33m'
red=$'\033[31m'
blue=$'\033[34m'
reset=$'\033[0m'

ok()   { printf "%s✓%s %s\n" "$green" "$reset" "$*"; }
warn() { printf "%s!%s %s\n" "$yellow" "$reset" "$*"; }
fail() { printf "%s✗%s %s\n" "$red" "$reset" "$*"; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

need_cmd curl
need_cmd uname

ARCH="$(uname -m)"

case "$ARCH" in
  arm64)
    RUST_HOST="aarch64-apple-darwin"
    ;;
  x86_64)
    RUST_HOST="x86_64-apple-darwin"
    ;;
  *)
    fail "Unsupported Mac architecture: $ARCH"
    ;;
esac

workdir="$HOME/Developer/rust-tmp"
mkdir -p "$workdir"

printf "\n%sRust Official Installer for macOS%s\n" "$bold" "$reset"
printf "%s────────────────────────────────%s\n\n" "$blue" "$reset"

ok "Detected architecture: $ARCH → $RUST_HOST"

INSTALLER="$workdir/rustup-init.sh"
INSTALLER_URL="https://sh.rustup.rs"

printf "\nDownloading:\n%s\n\n" "$INSTALLER_URL"

curl --proto '=https' --tlsv1.2 -fL \
  --retry 3 --connect-timeout 15 \
  "$INSTALLER_URL" -o "$INSTALLER"

test -s "$INSTALLER" || fail "Download failed or installer is empty."
ok "Downloaded rustup installer."

printf "\nInstalling the stable Rust toolchain...\n"
sh "$INSTALLER" -y --default-toolchain stable --profile default

RUST_ENV="$HOME/.cargo/env"
test -f "$RUST_ENV" || fail "Rust environment file was not created."

# shellcheck disable=SC1090
source "$RUST_ENV"
hash -r 2>/dev/null || true
rehash 2>/dev/null || true

rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy

RUSTC_BIN="$(command -v rustc || true)"
CARGO_BIN="$(command -v cargo || true)"
RUSTUP_BIN="$(command -v rustup || true)"
RUSTC_VERSION="$(rustc --version 2>/dev/null || true)"
CARGO_VERSION="$(cargo --version 2>/dev/null || true)"
RUSTUP_VERSION="$(rustup --version 2>/dev/null | head -n 1 || true)"
ACTIVE_TOOLCHAIN="$(rustup show active-toolchain 2>/dev/null || true)"
HOST_TRIPLE="$(rustc -vV 2>/dev/null | awk '/^host:/ { print $2 }')"

printf "\n%sVisual Sanity Check%s\n" "$bold" "$reset"
printf "%s────────────────────%s\n" "$blue" "$reset"

printf "%-22s %s\n" "Expected host:" "$RUST_HOST"
printf "%-22s %s\n" "Active rustc:" "${RUSTC_BIN:-not found}"
printf "%-22s %s\n" "Active cargo:" "${CARGO_BIN:-not found}"
printf "%-22s %s\n" "Active rustup:" "${RUSTUP_BIN:-not found}"
printf "%-22s %s\n" "rustc version:" "${RUSTC_VERSION:-failed}"
printf "%-22s %s\n" "cargo version:" "${CARGO_VERSION:-failed}"
printf "%-22s %s\n" "rustup version:" "${RUSTUP_VERSION:-failed}"
printf "%-22s %s\n" "Toolchain:" "${ACTIVE_TOOLCHAIN:-failed}"
printf "%-22s %s\n" "Host:" "${HOST_TRIPLE:-failed}"

printf "\n%sPATH priority check%s\n" "$bold" "$reset"
printf "%s──────────────────%s\n" "$blue" "$reset"
which -a rustc 2>/dev/null | awk '{ printf "%2d. %s\n", NR, $0 }' || true
which -a cargo 2>/dev/null | awk '{ printf "%2d. %s\n", NR, $0 }' || true

printf "\n"

if [ "$RUSTC_BIN" = "$HOME/.cargo/bin/rustc" ]; then
  ok "Good: rustup-managed rustc is first in PATH."
else
  warn "Rust installed, but another rustc may be first in PATH."
  warn "Open a new terminal tab, then run: which rustc && rustc --version"
fi

if [ "$HOST_TRIPLE" = "$RUST_HOST" ]; then
  ok "Host architecture check passed."
else
  fail "Host mismatch. Expected $RUST_HOST but got: ${HOST_TRIPLE:-nothing}"
fi

printf "\n%sDone.%s Open a new terminal tab and run:\n\n" "$green" "$reset"
printf "  rustc --version\n"
printf "  cargo --version\n"
printf "  rustup show\n\n"
ZSH
```

### Why this is better

- Keeps the downloaded installer under **`~/Developer/rust-tmp`**
- Works safely when pasted into **zsh**
- Detects Apple Silicon vs Intel Mac
- Uses the official `rustup` installer
- Installs the stable toolchain, Cargo, rustfmt, and Clippy
- Uses rustup-managed binaries under `~/.cargo/bin`
- Shows a sanity-check table at the end
- Shows all `rustc` and `cargo` binaries found so Homebrew or old installs are
  easy to spot

## rust terminal stack (optional)

Rust alternatives to the Charm Go stack:

| Crate | What it is | When you use it |
| --- | --- | --- |
| **[console](https://github.com/console-rs/console)** | Terminal colors and styling | Styled headings and status messages |
| **[indicatif](https://github.com/console-rs/indicatif)** | Progress bars and spinners | Long-running scans and downloads |
| **[comfy-table](https://github.com/Nukesor/comfy-table)** | Styled terminal tables | Summaries, file lists, and system information |
| **[ratatui](https://github.com/ratatui/ratatui)** | Full terminal UI framework | Interactive dashboards, menus, and applications |
| **[crossterm](https://github.com/crossterm-rs/crossterm)** | Cross-platform terminal control | Keyboard events, raw mode, cursor control, and screen updates |

**This doc:** the backup scanner uses **console**, **indicatif**, and
**comfy-table**. Cargo downloads the crates automatically on the first build.

## backup dir

Same directory-scanner idea as `go.md`. `OUTPUT_DIR` is resolved from the
current user's home directory. The setup snippet writes a complete Cargo
project under **`~/Developer/rust-tmp`** and leaves it there for subsequent
runs.

```sh
TMP="$HOME/Developer/rust-tmp"
mkdir -p "$TMP/src"

cat > "$TMP/Cargo.toml" <<'TOML'
[package]
name = "dir-scanner"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = "0.4"
comfy-table = "7"
console = "0.15"
indicatif = "0.17"
TOML

cat > "$TMP/src/main.rs" <<'RUST'
use chrono::{DateTime, Local};
use comfy_table::{presets::UTF8_ROUND_CORNERS, Attribute, Cell, Color, Table};
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const DEFAULT_SCAN_ROOT: &str = ".";
const OUTPUT_BASENAME: &str = "tree_output";
const OUTPUT_EXT: &str = ".txt";
const SHOW_HIDDEN: bool = true;
const FOLLOW_SYMLINKS: bool = false;
const SCAN_EVERYTHING: bool = false;
const MAX_DEPTH: usize = 50;
const TOP_N_FILES: usize = 25;
const TOP_N_DIRS: usize = 50;
const SAVE_OUTPUT: bool = true;
const APPEND_TIMESTAMP_SUFFIX: bool = true;
const HARD_DEPTH_LIMIT: usize = 200;

#[derive(Clone)]
struct EntryInfo {
    path: PathBuf,
    name: String,
    is_dir: bool,
    size: u64,
}

#[derive(Clone)]
struct FileSize {
    path: PathBuf,
    size: u64,
}

struct ScanData {
    root_path: PathBuf,
    root_total_size: u64,
    root_entries: Vec<EntryInfo>,
    tree_lines: Vec<String>,
    file_count: usize,
    dir_count: usize,
    scanned_file_bytes: u64,
    max_depth_reached: usize,
    largest_files: Vec<FileSize>,
    dir_sizes: HashMap<PathBuf, u64>,
}

fn main() {
    let root_arg = env::var("SCAN_ROOT").unwrap_or_else(|_| DEFAULT_SCAN_ROOT.into());
    let root_path = match real_root(Path::new(root_arg.trim())) {
        Ok(path) => path,
        Err(_) => {
            eprintln!(
                "{}",
                Style::new()
                    .red()
                    .bold()
                    .apply_to(format!("Error: {root_arg:?} is not a valid directory."))
            );
            std::process::exit(1);
        }
    };

    let mode = if SCAN_EVERYTHING {
        "scan everything".to_string()
    } else {
        format!("max depth {MAX_DEPTH}")
    };

    let panel = Style::new().magenta().bold();
    println!();
    println!("{}", panel.apply_to("╭──────────────────────────────╮"));
    println!("{}", panel.apply_to("│ ◆ dir scanner                │"));
    println!("│ □ root  {:<21} │", root_path.display());
    println!("│ ◌ mode  {:<21} │", mode);
    println!("{}", panel.apply_to("╰──────────────────────────────╯"));
    println!();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .expect("valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message("scanning...");
    spinner.enable_steady_tick(Duration::from_millis(80));
    let data = scan_tree(&root_path);
    spinner.finish_and_clear();

    let output_path = SAVE_OUTPUT.then(make_output_path);
    let depth_label = if SCAN_EVERYTHING {
        format!("{} (actual)", data.max_depth_reached)
    } else {
        format!("{MAX_DEPTH} (max)")
    };

    let mut top_files = data.largest_files.clone();
    top_files.truncate(TOP_N_FILES);

    let mut top_dirs: Vec<_> = data
        .dir_sizes
        .iter()
        .filter(|(path, _)| *path != &data.root_path)
        .map(|(path, size)| (path.clone(), *size))
        .collect();
    top_dirs.sort_by(|a, b| b.1.cmp(&a.1));
    top_dirs.truncate(TOP_N_DIRS);

    let system_rows = vec![
        vec!["User".into(), run("whoami", &[])],
        vec!["Hostname".into(), run("hostname", &[])],
        vec!["OS".into(), run("sw_vers", &["-productName"])],
        vec!["OS Version".into(), run("sw_vers", &["-productVersion"])],
        vec!["Build".into(), run("sw_vers", &["-buildVersion"])],
        vec![
            "Platform".into(),
            format!("{} {}", env::consts::OS, env::consts::ARCH),
        ],
        vec!["User Lang".into(), env_or("LANG", "—")],
        vec![
            "Processor".into(),
            run("sysctl", &["-n", "machdep.cpu.brand_string"]),
        ],
        vec!["Cores".into(), run("sysctl", &["-n", "hw.ncpu"])],
        vec![
            "Memory".into(),
            run("sysctl", &["-n", "hw.memsize"])
                .parse::<u64>()
                .map(human_size)
                .unwrap_or_else(|_| "—".into()),
        ],
        vec!["Uptime".into(), run("uptime", &[])],
        vec!["Date".into(), Local::now().to_rfc2822()],
        vec!["Home".into(), env_or("HOME", "—")],
        vec!["Shell".into(), env_or("SHELL", "—")],
        vec!["Term".into(), env_or("TERM", "—")],
        vec!["Rust".into(), run("rustc", &["--version"])],
        vec!["Cargo".into(), run("cargo", &["--version"])],
    ];

    let output_dir = output_dir();
    let summary_rows = vec![
        vec!["Root".into(), data.root_path.display().to_string()],
        vec!["Depth".into(), depth_label],
        vec!["Directories".into(), data.dir_count.to_string()],
        vec!["Files".into(), data.file_count.to_string()],
        vec![
            "Scanned File Size".into(),
            human_size(data.scanned_file_bytes),
        ],
        vec!["Root Total Size".into(), human_size(data.root_total_size)],
        vec!["Root Items".into(), data.root_entries.len().to_string()],
        vec!["Show Hidden".into(), yes_no(SHOW_HIDDEN).into()],
        vec!["Follow Symlinks".into(), yes_no(FOLLOW_SYMLINKS).into()],
        vec!["Top Files Count".into(), TOP_N_FILES.to_string()],
        vec!["Top Dirs Count".into(), TOP_N_DIRS.to_string()],
        vec!["Save Output".into(), yes_no(SAVE_OUTPUT).into()],
        vec!["Output Dir".into(), output_dir.display().to_string()],
        vec![
            "Output File".into(),
            output_path
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "—".into()),
        ],
        vec!["Generated".into(), Local::now().to_rfc2822()],
    ];

    let root_rows = data
        .root_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let (kind, name, size) = if entry.is_dir {
                (
                    "□ dir".to_string(),
                    format!("{}/", entry.name),
                    human_size(*data.dir_sizes.get(&entry.path).unwrap_or(&0)),
                )
            } else {
                (
                    "▪ file".to_string(),
                    entry.name.clone(),
                    human_size(entry.size),
                )
            };
            vec![(index + 1).to_string(), kind, name, size]
        })
        .collect();

    let top_file_rows = top_files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            vec![
                (index + 1).to_string(),
                format!(
                    "▪ {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                ),
                relative_path(&data.root_path, &file.path),
                human_size(file.size),
            ]
        })
        .collect();

    let top_dir_rows = top_dirs
        .iter()
        .enumerate()
        .map(|(index, (path, size))| {
            vec![
                (index + 1).to_string(),
                format!(
                    "□ {}/",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                relative_path(&data.root_path, path),
                human_size(*size),
            ]
        })
        .collect();

    let sections = vec![
        render_section("▾ System Info", &["Field", "Value"], system_rows),
        render_section("▾ Summary", &["Field", "Value"], summary_rows),
        render_section(
            "▾ Root Contents",
            &["#", "Type", "Name", "Size"],
            root_rows,
        ),
        render_section(
            &format!("▾ Top {TOP_N_FILES} Largest Files"),
            &["#", "File", "Relative Path", "Size"],
            top_file_rows,
        ),
        render_section(
            &format!("▾ Top {TOP_N_DIRS} Largest Directories"),
            &["#", "Directory", "Relative Path", "Total Size"],
            top_dir_rows,
        ),
        format!(
            "{}\n{}\n{}",
            Style::new().magenta().bold().apply_to("▾ Full Tree"),
            Style::new().magenta().bold().apply_to(format!(
                "◆ {}  ◌ {}",
                data.root_path.display(),
                human_size(data.root_total_size)
            )),
            data.tree_lines.join("\n")
        ),
    ];

    let final_output = sections.join("\n\n");
    println!("{final_output}");

    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create output directory");
        }
        fs::write(&path, console::strip_ansi_codes(&final_output).as_bytes())
            .expect("write output file");
        println!();
        println!(
            "{}",
            Style::new()
                .green()
                .bold()
                .apply_to(format!("Saved: {}", path.display()))
        );
    }
}

fn scan_tree(root_path: &Path) -> ScanData {
    let mut dir_sizes = HashMap::new();
    let mut size_visited = HashSet::new();
    let root_total_size =
        scan_dir_size(root_path, &mut dir_sizes, &mut size_visited).unwrap_or(0);
    let root_entries = list_dir(root_path);

    let mut data = ScanData {
        root_path: root_path.to_path_buf(),
        root_total_size,
        root_entries,
        tree_lines: Vec::new(),
        file_count: 0,
        dir_count: 0,
        scanned_file_bytes: 0,
        max_depth_reached: 0,
        largest_files: Vec::new(),
        dir_sizes,
    };

    let mut tree_visited = HashSet::new();
    let tree_lines = build_tree_lines(root_path, 0, "", &mut tree_visited, &mut data);
    data.tree_lines = tree_lines;
    data.largest_files.sort_by(|a, b| b.size.cmp(&a.size));
    data
}

fn scan_dir_size(
    dir: &Path,
    dir_sizes: &mut HashMap<PathBuf, u64>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<u64> {
    let real = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !visited.insert(real) {
        return Ok(0);
    }

    let mut total = 0;
    for entry in list_dir(dir) {
        if entry.is_dir {
            total += scan_dir_size(&entry.path, dir_sizes, visited).unwrap_or(0);
        } else {
            total += entry.size;
        }
    }
    dir_sizes.insert(dir.to_path_buf(), total);
    Ok(total)
}

fn build_tree_lines(
    dir: &Path,
    depth: usize,
    prefix: &str,
    visited: &mut HashSet<PathBuf>,
    data: &mut ScanData,
) -> Vec<String> {
    if depth >= HARD_DEPTH_LIMIT || (!SCAN_EVERYTHING && depth >= MAX_DEPTH) {
        return Vec::new();
    }

    data.max_depth_reached = data.max_depth_reached.max(depth);
    let entries = list_dir(dir);
    let mut lines = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        let is_last = index + 1 == entries.len();
        let connector = if is_last { "└── " } else { "├── " };
        let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });

        if entry.is_dir {
            let real = fs::canonicalize(&entry.path).unwrap_or_else(|_| entry.path.clone());
            if !visited.insert(real) {
                continue;
            }
            data.dir_count += 1;
            let size = *data.dir_sizes.get(&entry.path).unwrap_or(&0);
            lines.push(format!(
                "{prefix}{connector}□ {}/  ◌ {}",
                entry.name,
                human_size(size)
            ));
            lines.extend(build_tree_lines(
                &entry.path,
                depth + 1,
                &next_prefix,
                visited,
                data,
            ));
        } else {
            data.file_count += 1;
            data.scanned_file_bytes += entry.size;
            data.largest_files.push(FileSize {
                path: entry.path.clone(),
                size: entry.size,
            });
            lines.push(format!(
                "{prefix}{connector}▪ {}  ◌ {}",
                entry.name,
                human_size(entry.size)
            ));
        }
    }
    lines
}

fn list_dir(dir: &Path) -> Vec<EntryInfo> {
    let mut entries = Vec::new();
    let Ok(children) = fs::read_dir(dir) else {
        return entries;
    };

    for child in children.flatten() {
        let name = child.file_name().to_string_lossy().into_owned();
        if !SHOW_HIDDEN && name.starts_with('.') {
            continue;
        }

        let path = child.path();
        let metadata = if FOLLOW_SYMLINKS {
            fs::metadata(&path)
        } else {
            fs::symlink_metadata(&path)
        };
        let Ok(metadata) = metadata else {
            continue;
        };
        let is_dir = metadata.is_dir();

        if is_dir
            && matches!(
                name.as_str(),
                ".git" | "node_modules" | ".venv" | "__pycache__" | ".DS_Store"
            )
        {
            continue;
        }

        entries.push(EntryInfo {
            path,
            name,
            is_dir,
            size: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

fn render_section(title: &str, headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_ROUND_CORNERS);
    table.set_header(headers.iter().map(|header| {
        Cell::new(header)
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold)
    }));
    for row in rows {
        table.add_row(row);
    }
    format!(
        "{}\n{}",
        Style::new().magenta().bold().apply_to(title),
        table
    )
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

fn format_timestamp_suffix(time: DateTime<Local>) -> String {
    time.format("%y.%-m.%-d_%-I.%M%P").to_string()
}

fn output_dir() -> PathBuf {
    home_dir().join("Developer").join("macos-reset")
}

fn make_output_path() -> PathBuf {
    let suffix = if APPEND_TIMESTAMP_SUFFIX {
        format!("-{}", format_timestamp_suffix(Local::now()))
    } else {
        String::new()
    };
    output_dir().join(format!("{OUTPUT_BASENAME}{suffix}{OUTPUT_EXT}"))
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME is not set")
}

fn relative_path(root: &Path, target: &Path) -> String {
    target
        .strip_prefix(root)
        .ok()
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".into())
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        "No"
    }
}

fn env_or(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.into())
}

fn run(command: &str, args: &[&str]) -> String {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "—".into())
}

fn real_root(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let resolved = fs::canonicalize(absolute)?;
    if resolved.is_dir() {
        Ok(resolved)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "not a directory"))
    }
}
RUST
```

## first run

```sh
cd "$HOME/Developer/rust-tmp" && cargo run --release
```

Cargo downloads and compiles the dependencies on the first run.

## subsequent run

```sh
cd "$HOME/Developer/rust-tmp" && cargo run --release
```

To scan another directory:

```sh
cd "$HOME/Developer/rust-tmp" && \
SCAN_ROOT="$HOME/Developer" cargo run --release
```
