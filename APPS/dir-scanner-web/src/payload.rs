use serde::Serialize;

use crate::scan::{
    self, ScanData, SCAN_EVERYTHING, MAX_DEPTH, TOP_N_DIRS, TOP_N_FILES,
};

#[derive(Serialize)]
pub struct KeyValue {
    pub key: &'static str,
    pub value: String,
}

#[derive(Serialize)]
pub struct EntryRow {
    pub index: usize,
    pub kind: String,
    pub name: String,
    pub size: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct RankedPath {
    pub index: usize,
    pub name: String,
    pub path: String,
    pub size: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct RankedDirCount {
    pub index: usize,
    pub name: String,
    pub path: String,
    pub file_count: usize,
}

#[derive(Serialize)]
pub struct ScanSummary {
    pub depth: String,
    pub dir_count: usize,
    pub file_count: usize,
    pub scanned_file_bytes: u64,
    pub scanned_file_size: String,
    pub root_total_size: u64,
    pub root_total_size_human: String,
    pub root_item_count: usize,
    pub max_depth_reached: usize,
}

#[derive(Serialize)]
pub struct IgnoredRow {
    pub index: usize,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub size: String,
    pub size_bytes: u64,
    pub file_count: usize,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub root_path: String,
    pub summary: ScanSummary,
    pub root_entries: Vec<EntryRow>,
    pub top_files: Vec<RankedPath>,
    pub top_dirs: Vec<RankedPath>,
    pub top_dirs_by_files: Vec<RankedDirCount>,
    pub ignored: Vec<IgnoredRow>,
    pub tree_lines: Vec<String>,
    pub report_text: String,
}

impl ScanResponse {
    pub fn from_scan(data: &ScanData) -> Self {
        let depth_label = if SCAN_EVERYTHING {
            format!("{} (actual)", data.max_depth_reached)
        } else {
            format!("{MAX_DEPTH} (max)")
        };

        let root_entries = data
            .root_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let (kind, name, size_bytes) = if entry.is_dir {
                    (
                        "dir".to_string(),
                        format!("{}/", entry.name),
                        *data.dir_sizes.get(&entry.path).unwrap_or(&0),
                    )
                } else {
                    ("file".to_string(), entry.name.clone(), entry.size)
                };
                EntryRow {
                    index: index + 1,
                    kind,
                    name,
                    size: scan::human_size(size_bytes),
                    size_bytes,
                }
            })
            .collect();

        let top_files = data
            .largest_files
            .iter()
            .take(TOP_N_FILES)
            .enumerate()
            .map(|(index, file)| RankedPath {
                index: index + 1,
                name: file
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                path: scan::relative_path(&data.root_path, &file.path),
                size: scan::human_size(file.size),
                size_bytes: file.size,
            })
            .collect();

        let mut top_dirs: Vec<_> = data
            .dir_sizes
            .iter()
            .filter(|(path, _)| *path != &data.root_path)
            .map(|(path, size)| (path.clone(), *size))
            .collect();
        top_dirs.sort_by(|a, b| b.1.cmp(&a.1));
        top_dirs.truncate(TOP_N_DIRS);

        let top_dirs = top_dirs
            .iter()
            .enumerate()
            .map(|(index, (path, size))| RankedPath {
                index: index + 1,
                name: format!(
                    "{}/",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                path: scan::relative_path(&data.root_path, path),
                size: scan::human_size(*size),
                size_bytes: *size,
            })
            .collect();

        let mut top_dirs_by_files: Vec<_> = data
            .dir_file_counts
            .iter()
            .filter(|(path, _)| *path != &data.root_path)
            .map(|(path, count)| (path.clone(), *count))
            .collect();
        top_dirs_by_files.sort_by(|a, b| b.1.cmp(&a.1));
        top_dirs_by_files.truncate(TOP_N_DIRS);

        let top_dirs_by_files = top_dirs_by_files
            .iter()
            .enumerate()
            .map(|(index, (path, count))| RankedDirCount {
                index: index + 1,
                name: format!(
                    "{}/",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                path: scan::relative_path(&data.root_path, path),
                file_count: *count,
            })
            .collect();

        let ignored = data
            .ignored_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| IgnoredRow {
                index: index + 1,
                kind: (if entry.is_dir { "dir" } else { "file" }).to_string(),
                name: if entry.is_dir {
                    format!("{}/", entry.name)
                } else {
                    entry.name.clone()
                },
                path: scan::relative_path(&data.root_path, &entry.path),
                size: scan::human_size(entry.size),
                size_bytes: entry.size,
                file_count: entry.file_count,
            })
            .collect();

        let summary = ScanSummary {
            depth: depth_label,
            dir_count: data.dir_count,
            file_count: data.file_count,
            scanned_file_bytes: data.scanned_file_bytes,
            scanned_file_size: scan::human_size(data.scanned_file_bytes),
            root_total_size: data.root_total_size,
            root_total_size_human: scan::human_size(data.root_total_size),
            root_item_count: data.root_entries.len(),
            max_depth_reached: data.max_depth_reached,
        };

        Self {
            root_path: data.root_path.display().to_string(),
            report_text: crate::report::render_report(data),
            summary,
            root_entries,
            top_files,
            top_dirs,
            top_dirs_by_files,
            ignored,
            tree_lines: data.tree_lines.clone(),
        }
    }
}

pub fn system_info() -> Vec<KeyValue> {
    crate::report::system_rows()
        .into_iter()
        .map(|(key, value)| KeyValue { key, value })
        .collect()
}
