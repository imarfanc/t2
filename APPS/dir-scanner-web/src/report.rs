use chrono::{DateTime, Local};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::scan::{
    self, ScanData, APPEND_TIMESTAMP_SUFFIX, OUTPUT_BASENAME, OUTPUT_EXT, SCAN_EVERYTHING,
    SHOW_HIDDEN, FOLLOW_SYMLINKS, MAX_DEPTH, SAVE_OUTPUT, TOP_N_DIRS, TOP_N_FILES,
};

pub fn write_report(data: &ScanData) -> std::io::Result<PathBuf> {
    let path = make_output_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, render_report(data))?;
    Ok(path)
}

pub fn render_report(data: &ScanData) -> String {
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

    let mut top_dirs_by_files: Vec<_> = data
        .dir_file_counts
        .iter()
        .filter(|(path, _)| *path != &data.root_path)
        .map(|(path, count)| (path.clone(), *count))
        .collect();
    top_dirs_by_files.sort_by(|a, b| b.1.cmp(&a.1));
    top_dirs_by_files.truncate(TOP_N_DIRS);

    let output_path = SAVE_OUTPUT.then(make_output_path);
    let output_dir = output_dir();

    let sections = vec![
        render_kv_section("▾ System Info", system_rows()),
        render_kv_section(
            "▾ Summary",
            vec![
                ("Root", data.root_path.display().to_string()),
                ("Depth", depth_label),
                ("Directories", data.dir_count.to_string()),
                ("Files", data.file_count.to_string()),
                (
                    "Scanned File Size",
                    scan::human_size(data.scanned_file_bytes),
                ),
                (
                    "Root Total Size",
                    scan::human_size(data.root_total_size),
                ),
                ("Root Items", data.root_entries.len().to_string()),
                ("Show Hidden", yes_no(SHOW_HIDDEN)),
                ("Follow Symlinks", yes_no(FOLLOW_SYMLINKS)),
                ("Top Files Count", TOP_N_FILES.to_string()),
                ("Top Dirs Count", TOP_N_DIRS.to_string()),
                ("Save Output", yes_no(SAVE_OUTPUT)),
                ("Output Dir", output_dir.display().to_string()),
                (
                    "Output File",
                    output_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "—".into()),
                ),
                ("Generated", Local::now().to_rfc2822()),
            ],
        ),
        render_table_section(
            "▾ Root Contents",
            &["#", "Type", "Name", "Size"],
            data.root_entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let (kind, name, size) = if entry.is_dir {
                        (
                            "□ dir".to_string(),
                            format!("{}/", entry.name),
                            scan::human_size(*data.dir_sizes.get(&entry.path).unwrap_or(&0)),
                        )
                    } else {
                        (
                            "▪ file".to_string(),
                            entry.name.clone(),
                            scan::human_size(entry.size),
                        )
                    };
                    vec![
                        (index + 1).to_string(),
                        kind,
                        name,
                        size,
                    ]
                })
                .collect(),
        ),
        render_table_section(
            &format!("▾ Top {TOP_N_FILES} Largest Files"),
            &["#", "File", "Relative Path", "Size"],
            top_files
                .iter()
                .enumerate()
                .map(|(index, file)| {
                    vec![
                        (index + 1).to_string(),
                        format!(
                            "▪ {}",
                            file.path.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        scan::relative_path(&data.root_path, &file.path),
                        scan::human_size(file.size),
                    ]
                })
                .collect(),
        ),
        render_table_section(
            &format!("▾ Top {TOP_N_DIRS} Largest Directories"),
            &["#", "Directory", "Relative Path", "Total Size"],
            top_dirs
                .iter()
                .enumerate()
                .map(|(index, (path, size))| {
                    vec![
                        (index + 1).to_string(),
                        format!(
                            "□ {}/",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        scan::relative_path(&data.root_path, path),
                        scan::human_size(*size),
                    ]
                })
                .collect(),
        ),
        render_table_section(
            &format!("▾ Top {TOP_N_DIRS} Directories by File Count"),
            &["#", "Directory", "Relative Path", "Files"],
            top_dirs_by_files
                .iter()
                .enumerate()
                .map(|(index, (path, count))| {
                    vec![
                        (index + 1).to_string(),
                        format!(
                            "□ {}/",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        scan::relative_path(&data.root_path, path),
                        count.to_string(),
                    ]
                })
                .collect(),
        ),
        format!(
            "▾ Full Tree\n◆ {}  ◌ {}\n{}",
            data.root_path.display(),
            scan::human_size(data.root_total_size),
            data.tree_lines.join("\n")
        ),
    ];

    sections.join("\n\n")
}

fn render_kv_section(title: &str, rows: Vec<(&str, String)>) -> String {
    let body = rows
        .into_iter()
        .map(|(key, value)| format!("{key:<22} {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{title}\n{body}")
}

fn render_table_section(title: &str, headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            rows.iter()
                .map(|row| row.get(index).map(String::len).unwrap_or(0))
                .chain(std::iter::once(header.len()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    let header_line = headers
        .iter()
        .enumerate()
        .map(|(index, header)| format!("{header:<width$}", width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ");

    let body = rows
        .into_iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(index, cell)| format!("{cell:<width$}", width = widths[index]))
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{title}\n{header_line}\n{body}")
}

pub fn system_rows() -> Vec<(&'static str, String)> {
    vec![
        ("User", run("whoami", &[])),
        ("Hostname", run("hostname", &[])),
        ("OS", run("sw_vers", &["-productName"])),
        ("OS Version", run("sw_vers", &["-productVersion"])),
        ("Build", run("sw_vers", &["-buildVersion"])),
        (
            "Platform",
            format!("{} {}", env::consts::OS, env::consts::ARCH),
        ),
        ("User Lang", env_or("LANG", "—")),
        (
            "Processor",
            run("sysctl", &["-n", "machdep.cpu.brand_string"]),
        ),
        ("Cores", run("sysctl", &["-n", "hw.ncpu"])),
        (
            "Memory",
            run("sysctl", &["-n", "hw.memsize"])
                .parse::<u64>()
                .map(scan::human_size)
                .unwrap_or_else(|_| "—".into()),
        ),
        ("Uptime", run("uptime", &[])),
        ("Date", Local::now().to_rfc2822()),
        ("Home", env_or("HOME", "—")),
        ("Shell", env_or("SHELL", "—")),
        ("Term", env_or("TERM", "—")),
        ("Rust", run("rustc", &["--version"])),
        ("Cargo", run("cargo", &["--version"])),
    ]
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

fn yes_no(value: bool) -> String {
    if value {
        "Yes".into()
    } else {
        "No".into()
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
