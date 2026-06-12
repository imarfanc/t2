use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs};
use ratatui::Frame;

use crate::report;
use crate::scan::{self, ScanData, SCAN_EVERYTHING, MAX_DEPTH, TOP_N_DIRS, TOP_N_FILES};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Summary,
    System,
    Root,
    TopFiles,
    TopDirs,
    TopDirsByFiles,
    Tree,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Summary,
        Tab::System,
        Tab::Root,
        Tab::TopFiles,
        Tab::TopDirs,
        Tab::TopDirsByFiles,
        Tab::Tree,
    ];

    fn title(self) -> &'static str {
        match self {
            Tab::Summary => "Summary",
            Tab::System => "System",
            Tab::Root => "Root",
            Tab::TopFiles => "Top Files",
            Tab::TopDirs => "Top Dirs",
            Tab::TopDirsByFiles => "Top by Files",
            Tab::Tree => "Tree",
        }
    }

    fn next(self) -> Self {
        let index = Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        let index = Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub enum AppMode {
    Scanning { tick: u8 },
    Ready,
    Saved(PathBuf),
    Error(String),
}

pub struct App {
    pub mode: AppMode,
    pub data: Option<ScanData>,
    pub tab: Tab,
    pub scroll: u16,
    pub status: String,
}

impl App {
    pub fn new_scanning() -> Self {
        Self {
            mode: AppMode::Scanning { tick: 0 },
            data: None,
            tab: Tab::Summary,
            scroll: 0,
            status: "Scanning…".into(),
        }
    }

    pub fn set_data(&mut self, data: ScanData) {
        self.data = Some(data);
        self.mode = AppMode::Ready;
        self.status = "q quit  Tab next  Shift+Tab prev  s save  ↑↓ scroll".into();
    }

    pub fn set_error(&mut self, message: String) {
        self.mode = AppMode::Error(message);
        self.status = "q quit".into();
    }

    pub fn tick(&mut self) {
        if let AppMode::Scanning { tick } = &mut self.mode {
            *tick = tick.wrapping_add(1);
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = self.tab.next();
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        self.tab = self.tab.prev();
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn save(&mut self) {
        let Some(data) = self.data.as_ref() else {
            return;
        };
        match report::write_report(data) {
            Ok(path) => {
                self.mode = AppMode::Saved(path.clone());
                self.status = format!("Saved {} — q quit  Tab browse", path.display());
            }
            Err(err) => {
                self.mode = AppMode::Error(err.to_string());
                self.status = "q quit".into();
            }
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    match &app.mode {
        AppMode::Scanning { tick } => render_scanning(frame, chunks[0], *tick),
        AppMode::Error(message) => render_error(frame, chunks[0], message),
        AppMode::Saved(path) => {
            if let Some(data) = &app.data {
                render_ready(frame, chunks[0], app, data);
            }
            render_status(frame, chunks[1], &format!("Saved {}", path.display()));
            return;
        }
        AppMode::Ready => {
            if let Some(data) = &app.data {
                render_ready(frame, chunks[0], app, data);
            }
        }
    }

    render_status(frame, chunks[1], &app.status);
}

fn render_scanning(frame: &mut Frame, area: Rect, tick: u8) {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = frames[usize::from(tick) % frames.len()];
    let block = Block::default()
        .title(" dir scanner ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{spinner} "),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw("scanning directory tree…"),
    ]))
    .block(block);
    frame.render_widget(paragraph, area);
}

fn render_error(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::default()
        .title(" error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let paragraph = Paragraph::new(message)
        .style(Style::default().fg(Color::Red))
        .block(block);
    frame.render_widget(paragraph, area);
}

fn render_ready(frame: &mut Frame, area: Rect, app: &App, data: &ScanData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|tab| {
            let prefix = if *tab == app.tab { "▸ " } else { "  " };
            Line::from(format!("{prefix}{}", tab.title()))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");
    frame.render_widget(tabs, chunks[0]);

    let panel = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = panel.inner(chunks[1]);
    frame.render_widget(panel, chunks[1]);

    match app.tab {
        Tab::Summary => render_summary(frame, inner, data),
        Tab::System => render_system(frame, inner, app.scroll),
        Tab::Root => render_root(frame, inner, data, app.scroll),
        Tab::TopFiles => render_top_files(frame, inner, data, app.scroll),
        Tab::TopDirs => render_top_dirs(frame, inner, data, app.scroll),
        Tab::TopDirsByFiles => render_top_dirs_by_files(frame, inner, data, app.scroll),
        Tab::Tree => render_tree(frame, inner, data, app.scroll),
    }
}

fn render_summary(frame: &mut Frame, area: Rect, data: &ScanData) {
    let depth_label = if SCAN_EVERYTHING {
        format!("{} (actual)", data.max_depth_reached)
    } else {
        format!("{MAX_DEPTH} (max)")
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("root ", Style::default().fg(Color::Gray)),
            Span::raw(data.root_path.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("total ", Style::default().fg(Color::Gray)),
            Span::raw(scan::human_size(data.root_total_size)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("depth      ", Style::default().fg(Color::Gray)),
            Span::raw(depth_label),
        ]),
        Line::from(vec![
            Span::styled("dirs       ", Style::default().fg(Color::Gray)),
            Span::raw(data.dir_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("files      ", Style::default().fg(Color::Gray)),
            Span::raw(data.file_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("scanned    ", Style::default().fg(Color::Gray)),
            Span::raw(scan::human_size(data.scanned_file_bytes)),
        ]),
        Line::from(vec![
            Span::styled("root items ", Style::default().fg(Color::Gray)),
            Span::raw(data.root_entries.len().to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn slice_rows<T>(rows: Vec<T>, scroll: u16) -> Vec<T> {
    rows.into_iter().skip(scroll as usize).collect()
}

fn render_system(frame: &mut Frame, area: Rect, scroll: u16) {
    let rows = slice_rows(
        report::system_rows()
            .into_iter()
            .map(|(key, value)| {
                Row::new(vec![
                    Cell::from(key).style(Style::default().fg(Color::Cyan)),
                    Cell::from(value),
                ])
            })
            .collect::<Vec<_>>(),
        scroll,
    );

    let table = Table::new(rows, [Constraint::Length(18), Constraint::Min(20)]).column_spacing(2);
    frame.render_widget(table, area);
}

fn render_root(frame: &mut Frame, area: Rect, data: &ScanData, scroll: u16) {
    let rows = slice_rows(
        data.root_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let (kind, name, size) = if entry.is_dir {
                    (
                        "dir",
                        format!("{}/", entry.name),
                        scan::human_size(*data.dir_sizes.get(&entry.path).unwrap_or(&0)),
                    )
                } else {
                    ("file", entry.name.clone(), scan::human_size(entry.size))
                };
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(kind),
                    Cell::from(name),
                    Cell::from(size),
                ])
            })
            .collect::<Vec<_>>(),
        scroll,
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(20),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["#", "Type", "Name", "Size"])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .bottom_margin(1),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_top_files(frame: &mut Frame, area: Rect, data: &ScanData, scroll: u16) {
    let rows = slice_rows(
        data.largest_files
            .iter()
            .take(TOP_N_FILES)
            .enumerate()
            .map(|(index, file)| {
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(
                        file.path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    ),
                    Cell::from(scan::relative_path(&data.root_path, &file.path)),
                    Cell::from(scan::human_size(file.size)),
                ])
            })
            .collect::<Vec<_>>(),
        scroll,
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(18),
            Constraint::Min(20),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["#", "File", "Path", "Size"])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .bottom_margin(1),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_top_dirs(frame: &mut Frame, area: Rect, data: &ScanData, scroll: u16) {
    let mut top_dirs: Vec<_> = data
        .dir_sizes
        .iter()
        .filter(|(path, _)| *path != &data.root_path)
        .map(|(path, size)| (path.clone(), *size))
        .collect();
    top_dirs.sort_by(|a, b| b.1.cmp(&a.1));
    top_dirs.truncate(TOP_N_DIRS);

    let rows = slice_rows(
        top_dirs
            .iter()
            .enumerate()
            .map(|(index, (path, size))| {
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(format!(
                        "{}/",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )),
                    Cell::from(scan::relative_path(&data.root_path, path)),
                    Cell::from(scan::human_size(*size)),
                ])
            })
            .collect::<Vec<_>>(),
        scroll,
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(18),
            Constraint::Min(20),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["#", "Directory", "Path", "Size"])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .bottom_margin(1),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_top_dirs_by_files(frame: &mut Frame, area: Rect, data: &ScanData, scroll: u16) {
    let mut top_dirs: Vec<_> = data
        .dir_file_counts
        .iter()
        .filter(|(path, _)| *path != &data.root_path)
        .map(|(path, count)| (path.clone(), *count))
        .collect();
    top_dirs.sort_by(|a, b| b.1.cmp(&a.1));
    top_dirs.truncate(TOP_N_DIRS);

    let rows = slice_rows(
        top_dirs
            .iter()
            .enumerate()
            .map(|(index, (path, count))| {
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(format!(
                        "{}/",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )),
                    Cell::from(scan::relative_path(&data.root_path, path)),
                    Cell::from(count.to_string()),
                ])
            })
            .collect::<Vec<_>>(),
        scroll,
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(18),
            Constraint::Min(20),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["#", "Directory", "Path", "Files"])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .bottom_margin(1),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_tree(frame: &mut Frame, area: Rect, data: &ScanData, scroll: u16) {
    let header = Line::from(vec![
        Span::styled("◆ ", Style::default().fg(Color::Magenta)),
        Span::raw(data.root_path.display().to_string()),
        Span::raw("  "),
        Span::styled(
            scan::human_size(data.root_total_size),
            Style::default().fg(Color::Gray),
        ),
    ]);
    let mut lines = vec![header, Line::from("")];
    lines.extend(data.tree_lines.iter().map(|line| Line::from(line.as_str())));

    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, area: Rect, text: &str) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Gray));
    frame.render_widget(paragraph, block.inner(area));
    frame.render_widget(block, area);
}
