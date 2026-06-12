use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub struct ScanOptions {
    pub ignore_dir_names: HashSet<String>,
    pub ignore_file_names: HashSet<String>,
}

impl ScanOptions {
    pub fn with_defaults() -> Self {
        let mut options = Self::default();
        for name in [".git", "node_modules", ".venv", "__pycache__"] {
            options.ignore_dir_names.insert(name.into());
        }
        options.ignore_file_names.insert(".DS_Store".into());
        options
    }

    pub fn with_extra_ignores(ignore_dirs: &[String], ignore_files: &[String]) -> Self {
        let mut options = Self::with_defaults();
        for name in ignore_dirs {
            let name = name.trim();
            if !name.is_empty() {
                options.ignore_dir_names.insert(name.into());
            }
        }
        for name in ignore_files {
            let name = name.trim();
            if !name.is_empty() {
                options.ignore_file_names.insert(name.into());
            }
        }
        options
    }
}

pub fn parse_ignore_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

pub const DEFAULT_SCAN_ROOT: &str = ".";
pub const OUTPUT_BASENAME: &str = "tree_output";
pub const OUTPUT_EXT: &str = ".txt";
pub const SHOW_HIDDEN: bool = true;
pub const FOLLOW_SYMLINKS: bool = false;
pub const SCAN_EVERYTHING: bool = false;
pub const MAX_DEPTH: usize = 50;
pub const TOP_N_FILES: usize = 25;
pub const TOP_N_DIRS: usize = 50;
pub const SAVE_OUTPUT: bool = true;
pub const APPEND_TIMESTAMP_SUFFIX: bool = true;
pub const HARD_DEPTH_LIMIT: usize = 200;

#[derive(Clone)]
pub struct EntryInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Clone)]
pub struct FileSize {
    pub path: PathBuf,
    pub size: u64,
}

pub struct ScanData {
    pub root_path: PathBuf,
    pub root_total_size: u64,
    pub root_entries: Vec<EntryInfo>,
    pub tree_lines: Vec<String>,
    pub file_count: usize,
    pub dir_count: usize,
    pub scanned_file_bytes: u64,
    pub max_depth_reached: usize,
    pub largest_files: Vec<FileSize>,
    pub dir_sizes: HashMap<PathBuf, u64>,
    pub dir_file_counts: HashMap<PathBuf, usize>,
}

pub fn scan_tree(root_path: &Path, options: &ScanOptions) -> ScanData {
    let mut dir_sizes = HashMap::new();
    let mut dir_file_counts = HashMap::new();
    let mut size_visited = HashSet::new();
    let root_total_size = scan_dir_size(
        root_path,
        options,
        &mut dir_sizes,
        &mut dir_file_counts,
        &mut size_visited,
    )
    .map(|(size, _)| size)
    .unwrap_or(0);
    let root_entries = list_dir(root_path, options);

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
        dir_file_counts,
    };

    let mut tree_visited = HashSet::new();
    let tree_lines = build_tree_lines(root_path, options, 0, "", &mut tree_visited, &mut data);
    data.tree_lines = tree_lines;
    data.largest_files.sort_by(|a, b| b.size.cmp(&a.size));
    data
}

fn scan_dir_size(
    dir: &Path,
    options: &ScanOptions,
    dir_sizes: &mut HashMap<PathBuf, u64>,
    dir_file_counts: &mut HashMap<PathBuf, usize>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<(u64, usize)> {
    let real = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !visited.insert(real) {
        return Ok((0, 0));
    }

    let mut total = 0;
    let mut file_count = 0;
    for entry in list_dir(dir, options) {
        if entry.is_dir {
            let (sub_size, sub_files) =
                scan_dir_size(&entry.path, options, dir_sizes, dir_file_counts, visited)?;
            total += sub_size;
            file_count += sub_files;
        } else {
            total += entry.size;
            file_count += 1;
        }
    }
    dir_sizes.insert(dir.to_path_buf(), total);
    dir_file_counts.insert(dir.to_path_buf(), file_count);
    Ok((total, file_count))
}

fn build_tree_lines(
    dir: &Path,
    options: &ScanOptions,
    depth: usize,
    prefix: &str,
    visited: &mut HashSet<PathBuf>,
    data: &mut ScanData,
) -> Vec<String> {
    if depth >= HARD_DEPTH_LIMIT || (!SCAN_EVERYTHING && depth >= MAX_DEPTH) {
        return Vec::new();
    }

    data.max_depth_reached = data.max_depth_reached.max(depth);
    let entries = list_dir(dir, options);
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
                options,
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

fn should_skip(name: &str, is_dir: bool, options: &ScanOptions) -> bool {
    if is_dir {
        options.ignore_dir_names.contains(name)
    } else {
        options.ignore_file_names.contains(name)
    }
}

fn list_dir(dir: &Path, options: &ScanOptions) -> Vec<EntryInfo> {
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

        if should_skip(&name, is_dir, options) {
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

pub fn human_size(bytes: u64) -> String {
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

pub fn relative_path(root: &Path, target: &Path) -> String {
    target
        .strip_prefix(root)
        .ok()
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".into())
}

pub fn real_root(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let resolved = fs::canonicalize(absolute)?;
    if resolved.is_dir() {
        Ok(resolved)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        ))
    }
}
