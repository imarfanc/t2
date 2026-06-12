//! Repo snapshot generator.
//! On server start, scans the served root and writes Admin/repo-info/repo-info.yaml.
//! Behaviour is configured in Admin/repo-info/config.yaml (ignore list, history depth).
//! Admin/repo-info/repo-info.html fetches the snapshot and renders it.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Entry {
    name: String,
    is_dir: bool,
    bytes: u64,
    files: u64,
}

struct Config {
    ignore_folders: Vec<String>,
    ignore_files: Vec<String>,
    history_limit: usize,
    history_expanded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ignore_folders: vec![".git".into()],
            ignore_files: Vec::new(),
            history_limit: 20,
            history_expanded: false,
        }
    }
}

/// Minimal parser for Admin/repo-info/config.yaml — supports `key: value` scalars
/// and flat `ignore_folders:` / `ignore_files:` lists. Missing file → defaults.
fn load_config(root: &Path) -> Config {
    let mut cfg = Config::default();
    let Ok(text) = fs::read_to_string(root.join("Admin/repo-info/config.yaml")) else {
        return cfg;
    };
    let mut list_target: Option<&mut Vec<String>> = None;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if let Some(item) = line.strip_prefix("  - ").or(line.strip_prefix("- ")) {
            if let Some(list) = list_target.as_deref_mut() {
                list.push(item.trim().trim_matches('"').to_string());
            }
            continue;
        }
        list_target = None;
        if line == "ignore_folders:" {
            cfg.ignore_folders.clear();
            list_target = Some(&mut cfg.ignore_folders);
        } else if line == "ignore_files:" {
            cfg.ignore_files.clear();
            list_target = Some(&mut cfg.ignore_files);
        } else if let Some(v) = line.strip_prefix("history_limit:") {
            if let Ok(n) = v.trim().parse() {
                cfg.history_limit = n;
            }
        } else if let Some(v) = line.strip_prefix("history_expanded:") {
            cfg.history_expanded = matches!(v.trim(), "true" | "yes" | "1");
        }
    }
    cfg
}

/// True if `rel` (path relative to repo root) should be skipped.
/// Entries containing '/' match the relative path exactly; bare names match at any depth.
fn matches_ignore_pattern(rel: &str, name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|i| {
        let i = i.trim_matches('/');
        if i.contains('/') { i == rel } else { i == name }
    })
}

fn is_ignored(rel: &str, name: &str, is_dir: bool, folders: &[String], files: &[String]) -> bool {
    if is_dir {
        matches_ignore_pattern(rel, name, folders)
    } else {
        matches_ignore_pattern(rel, name, files)
    }
}

/// Sum size + file count under `path` with no ignore filtering
/// (used to report how big an ignored entry is).
fn raw_walk(path: &Path) -> (u64, u64) {
    let mut bytes = 0;
    let mut files = 0;
    if let Ok(rd) = fs::read_dir(path) {
        for e in rd.flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => {
                    let (b, f) = raw_walk(&e.path());
                    bytes += b;
                    files += f;
                }
                Ok(m) if m.is_file() => {
                    bytes += m.len();
                    files += 1;
                }
                _ => {}
            }
        }
    }
    (bytes, files)
}

/// Recursively sum size + file count under `path`, skipping ignored entries.
/// Skipped entries are recorded in `ignored` (name = path relative to root).
/// `rel` is `path` relative to the repo root, "" at the root itself.
fn walk(
    path: &Path,
    rel: &str,
    folders: &[String],
    files: &[String],
    ignored: &mut Vec<Entry>,
) -> (u64, u64) {
    let mut bytes = 0;
    let mut files_count = 0;
    if let Ok(rd) = fs::read_dir(path) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            let p = e.path();
            let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
            if is_ignored(&child_rel, &name, is_dir, folders, files) {
                let (b, f) = if is_dir { raw_walk(&p) } else {
                    (e.metadata().map(|m| m.len()).unwrap_or(0), 1)
                };
                ignored.push(Entry { name: child_rel, is_dir, bytes: b, files: f });
                continue;
            }
            match e.metadata() {
                Ok(m) if m.is_dir() => {
                    let (b, f) = walk(&p, &child_rel, folders, files, ignored);
                    bytes += b;
                    files_count += f;
                }
                Ok(m) if m.is_file() => {
                    bytes += m.len();
                    files_count += 1;
                }
                _ => {}
            }
        }
    }
    (bytes, files_count)
}

fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn yaml_quote(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "")
    )
}

/// Scan `root` and write `root/Admin/repo-info/repo-info.yaml`.
/// Returns Err message on failure (non-fatal for the server).
pub fn write_snapshot(root: &Path) -> Result<(), String> {
    let cfg = load_config(root);

    let mut entries: Vec<Entry> = Vec::new();
    let mut ignored: Vec<Entry> = Vec::new();
    let rd = fs::read_dir(root).map_err(|e| e.to_string())?;
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
        if is_ignored(&name, &name, is_dir, &cfg.ignore_folders, &cfg.ignore_files) {
            let (b, f) = if is_dir { raw_walk(&p) } else {
                (e.metadata().map(|m| m.len()).unwrap_or(0), 1)
            };
            ignored.push(Entry { name, is_dir, bytes: b, files: f });
            continue;
        }
        match e.metadata() {
            Ok(m) if m.is_dir() => {
                let (bytes, files) =
                    walk(&p, &name, &cfg.ignore_folders, &cfg.ignore_files, &mut ignored);
                entries.push(Entry { name, is_dir: true, bytes, files });
            }
            Ok(m) if m.is_file() => {
                entries.push(Entry { name, is_dir: false, bytes: m.len(), files: 1 });
            }
            _ => {}
        }
    }
    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    ignored.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    let total_bytes: u64 = entries.iter().map(|e| e.bytes).sum();
    let total_files: u64 = entries.iter().map(|e| e.files).sum();

    let generated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut y = String::new();
    y.push_str("# generated by repo-server on startup — do not edit (configure via Admin/repo-info/config.yaml)\n");
    y.push_str(&format!("generated_unix: {generated}\n"));
    y.push_str(&format!("repo_path: {}\n", yaml_quote(&root.display().to_string())));
    y.push_str(&format!("git_remote: {}\n", yaml_quote(&git(root, &["remote", "get-url", "origin"]))));
    y.push_str(&format!("git_branch: {}\n", yaml_quote(&git(root, &["branch", "--show-current"]))));
    y.push_str(&format!("git_commit: {}\n", yaml_quote(&git(root, &["log", "-1", "--format=%h %s"]))));
    y.push_str(&format!(
        "ignore_folders: [{}]\n",
        cfg.ignore_folders
            .iter()
            .map(|i| yaml_quote(i))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    y.push_str(&format!(
        "ignore_files: [{}]\n",
        cfg.ignore_files
            .iter()
            .map(|i| yaml_quote(i))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    y.push_str(&format!("total_bytes: {total_bytes}\n"));
    y.push_str(&format!("total_files: {total_files}\n"));
    y.push_str(&format!("history_expanded: {}\n", cfg.history_expanded));

    y.push_str("entries:\n");
    for e in &entries {
        y.push_str(&format!(
            "  - name: {}\n    is_dir: {}\n    bytes: {}\n    files: {}\n",
            yaml_quote(&e.name),
            e.is_dir,
            e.bytes,
            e.files
        ));
    }

    y.push_str("ignored:\n");
    for e in &ignored {
        y.push_str(&format!(
            "  - name: {}\n    is_dir: {}\n    bytes: {}\n    files: {}\n",
            yaml_quote(&e.name),
            e.is_dir,
            e.bytes,
            e.files
        ));
    }

    // Recent git history. Fields separated by 0x1f, commits by 0x1e,
    // so multiline bodies survive intact.
    y.push_str("history:\n");
    let limit = format!("-{}", cfg.history_limit.max(1));
    let log = git(
        root,
        &["log", &limit, "--date=short", "--format=%h%x1f%ad%x1f%an%x1f%s%x1f%b%x1e"],
    );
    for record in log.split('\u{1e}') {
        let mut f = record.trim_start_matches(['\n', '\r']).split('\u{1f}');
        let (hash, date, author, subject, body) = (
            f.next().unwrap_or("").trim(),
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
            f.next().unwrap_or("").trim(),
        );
        if hash.is_empty() {
            continue;
        }
        y.push_str(&format!(
            "  - hash: {}\n    date: {}\n    author: {}\n    subject: {}\n    body: {}\n",
            yaml_quote(hash),
            yaml_quote(date),
            yaml_quote(author),
            yaml_quote(subject),
            yaml_quote(body)
        ));
    }

    let dir = root.join("Admin/repo-info");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("repo-info.yaml"), y).map_err(|e| e.to_string())
}
