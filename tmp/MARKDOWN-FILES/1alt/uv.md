---
title: "uv"
sort: 4
category: "macOS reset"
description: "install uv"
date: 2026-5-1
tags:
  - macOS
  - reset
  - uv
  - install
  - python
---

# install uv

## using curl

```sh
curl -LsSf https://astral.sh/uv/install.sh | sh
source "$HOME/.local/bin/env"
```

## using brew

```sh
brew install uv
```

## backup dir

Same directory scanner as [deno.md](./deno.md); uses Rich for tables, panels, and tree output. `OUTPUT_DIR` accepts `~` or `$HOME` via `Path.expanduser()`.

```sh
uv run --with rich python3 - <<'PY'
from __future__ import annotations

import os
import platform
import stat
import subprocess
import sys
from contextlib import nullcontext
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

from rich.console import Console, Group
from rich.panel import Panel
from rich.rule import Rule
from rich.status import Status
from rich.table import Table
from rich.tree import Tree

console = Console(record=True)

ROOT = "."
OUTPUT_DIR = Path("~/Developer/macos-reset").expanduser()  # ~ or $HOME
SHOW_HIDDEN = True
FOLLOW_SYMLINKS = False
SCAN_EVERYTHING = False  # True = scan everything (cycle-safe), False = respect MAX_DEPTH
MAX_DEPTH = 3

TOP_N_FILES = 25
TOP_N_DIRS = 50

SAVE_OUTPUT = True
OUTPUT_BASENAME = "tree_output"
OUTPUT_EXT = ".txt"
APPEND_TIMESTAMP_SUFFIX = True  # tree-output-yy.m.d_h.mmam.txt

SHOW_SPINNER = True

EXCLUDE_DIR_NAMES = {
    ".git",
    "node_modules",
    ".venv",
    "__pycache__",
    ".DS_Store",
}

ICON = {
    "section": "▾",
    "dir": "□",
    "file": "▪",
    "size": "◌",
    "root": "◆",
    "live": "●",
    "ok": "■",
}


def human_size(num_bytes: int) -> str:
    units = ["B", "KB", "MB", "GB", "TB", "PB"]
    size = float(num_bytes)
    i = 0
    while size >= 1024 and i < len(units) - 1:
        size /= 1024
        i += 1
    if i == 0:
        return f"{int(size)} {units[i]}"
    return f"{size:.1f} {units[i]}"


def is_hidden_name(name: str) -> bool:
    return name.startswith(".")


def format_timestamp_suffix(when: datetime | None = None) -> str:
    when = when or datetime.now()
    yy = str(when.year)[-2:]
    m = str(when.month)
    d = str(when.day)
    hour = when.hour % 12 or 12
    minute = f"{when.minute:02d}"
    ampm = "pm" if when.hour >= 12 else "am"
    return f"{yy}.{m}.{d}_{hour}.{minute}{ampm}"


def rel_from_root(root_path: Path, target_path: Path) -> str:
    try:
        rel = target_path.relative_to(root_path)
        return "." if rel == Path(".") else rel.as_posix()
    except ValueError:
        return target_path.as_posix()


def make_output_path() -> Path:
    suffix = f"-{format_timestamp_suffix()}" if APPEND_TIMESTAMP_SUFFIX else ""
    return OUTPUT_DIR / f"{OUTPUT_BASENAME}{suffix}{OUTPUT_EXT}"


@dataclass
class EntryInfo:
    path: Path
    name: str
    is_dir: bool
    size: int


def real_root(p: str) -> Path | None:
    path = Path(p).expanduser().resolve()
    try:
        if not path.is_dir():
            return None
        return path.resolve()
    except OSError:
        return None


def get_entry_info(full_path: Path, name: str) -> EntryInfo | None:
    try:
        st = full_path.stat() if FOLLOW_SYMLINKS else full_path.lstat()
        return EntryInfo(
            path=full_path,
            name=name,
            is_dir=stat_is_dir(st),
            size=st.st_size if stat_is_file(st) else 0,
        )
    except OSError:
        return None


def stat_is_dir(st: os.stat_result) -> bool:
    return stat.S_ISDIR(st.st_mode)


def stat_is_file(st: os.stat_result) -> bool:
    return stat.S_ISREG(st.st_mode)


def list_dir(dir_path: Path) -> list[EntryInfo]:
    out: list[EntryInfo] = []
    try:
        for child in dir_path.iterdir():
            if not SHOW_HIDDEN and is_hidden_name(child.name):
                continue
            info = get_entry_info(child, child.name)
            if info is None:
                continue
            if info.is_dir and info.name in EXCLUDE_DIR_NAMES:
                continue
            out.append(info)
    except OSError:
        return []
    out.sort(key=lambda e: (not e.is_dir, e.name.lower()))
    return out


def make_table(title: str, headers: list[str], rows: list[list[str | int]]) -> Table:
    table = Table(
        title=f"[bold magenta]{ICON['section']} {title}[/bold magenta]",
        show_header=True,
        header_style="bold cyan",
        border_style="bright_black",
        title_justify="left",
    )
    for i, header in enumerate(headers):
        justify = "right" if i == 0 and header == "#" else "left"
        style = "dim" if header in {"Relative Path", "Field"} else None
        table.add_column(header, justify=justify, style=style, overflow="fold")
    for row in rows:
        table.add_row(*[str(c) for c in row])
    return table


def run_cmd(cmd: list[str]) -> str:
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "—"


# -----------------------------------------------------------------------------
# Scan
# -----------------------------------------------------------------------------

root_path = real_root(ROOT)
if root_path is None:
    console.print(f"[bold red]Error:[/bold red] '{ROOT}' is not a valid directory.", stderr=True)
    sys.exit(1)

file_count = 0
dir_count = 0
scanned_file_bytes = 0
max_depth_reached = 0

largest_files: list[tuple[Path, int]] = []
dir_sizes: dict[Path, int] = {}
visited_dirs: set[Path] = set()


def scan_dir_size(dir_path: Path) -> int:
    try:
        real = dir_path.resolve()
    except OSError:
        real = dir_path
    if real in visited_dirs:
        return 0
    visited_dirs.add(real)

    total = 0
    for entry in list_dir(dir_path):
        if entry.is_dir:
            total += scan_dir_size(entry.path)
        else:
            total += entry.size
    dir_sizes[dir_path] = total
    return total


HARD_DEPTH_LIMIT = 200


def build_rich_tree(
    parent: Tree,
    dir_path: Path,
    depth: int,
    visited: set[Path] | None = None,
) -> None:
    global file_count, dir_count, scanned_file_bytes, max_depth_reached
    if visited is None:
        visited = set()
    if depth >= HARD_DEPTH_LIMIT:
        return
    if not SCAN_EVERYTHING and depth >= MAX_DEPTH:
        return
    max_depth_reached = max(max_depth_reached, depth)

    for entry in list_dir(dir_path):
        if entry.is_dir:
            try:
                real = entry.path.resolve()
            except OSError:
                real = entry.path
            if real in visited:
                continue
            visited.add(real)

            dir_count += 1
            size = dir_sizes.get(entry.path, 0)
            branch = parent.add(
                f"[bold cyan]{ICON['dir']} {entry.name}/[/bold cyan] "
                f"[dim]{ICON['size']} {human_size(size)}[/dim]"
            )
            build_rich_tree(branch, entry.path, depth + 1, visited)
        else:
            file_count += 1
            scanned_file_bytes += entry.size
            largest_files.append((entry.path, entry.size))
            parent.add(
                f"[white]{ICON['file']} {entry.name}[/white] "
                f"[dim]{ICON['size']} {human_size(entry.size)}[/dim]"
            )


console.print()
console.print(
    Panel(
        f"[bold magenta]{ICON['root']} dir scanner[/bold magenta]\n"
        f"[dim]{ICON['dir']} root[/dim]  [bold]{root_path}[/bold]\n"
        f"[dim]{ICON['size']} mode[/dim]  "
        f"[green]{'scan everything' if SCAN_EVERYTHING else f'max depth {MAX_DEPTH}'}[/green]",
        border_style="magenta",
        padding=(0, 2),
    )
)

scan_status = (
    Status(
        f"[green]{ICON['live']}[/green] scanning [bold]{root_path}[/bold]",
        spinner="dots",
        console=console,
    )
    if SHOW_SPINNER
    else nullcontext()
)

with scan_status:
    root_total_size = scan_dir_size(root_path)
    root_entries = list_dir(root_path)
    file_tree = Tree(
        f"[bold magenta]{ICON['root']} {root_path}[/bold magenta] "
        f"[dim]{ICON['size']} {human_size(root_total_size)}[/dim]",
        guide_style="bright_black",
    )
    build_rich_tree(file_tree, root_path, 0)

if SHOW_SPINNER:
    console.print(f"[green]{ICON['ok']} done[/green]")

# -----------------------------------------------------------------------------
# Rankings
# -----------------------------------------------------------------------------

largest_files.sort(key=lambda x: x[1], reverse=True)
top_files = largest_files[:TOP_N_FILES]

top_dirs = sorted(
    ((p, s) for p, s in dir_sizes.items() if p != root_path),
    key=lambda x: x[1],
    reverse=True,
)[:TOP_N_DIRS]

output_path = make_output_path() if SAVE_OUTPUT else None

mem_raw = run_cmd(["sysctl", "-n", "hw.memsize"])
mem_bytes = int(mem_raw) if mem_raw.isdigit() else 0

system_info_table = make_table(
    "System Info",
    ["Field", "Value"],
    [
        ["User", f"[white]{run_cmd(['whoami'])}[/white]"],
        ["Hostname", f"[white]{run_cmd(['hostname'])}[/white]"],
        ["OS", f"[white]{run_cmd(['sw_vers', '-productName'])}[/white]"],
        ["OS Version", f"[white]{run_cmd(['sw_vers', '-productVersion'])}[/white]"],
        ["Build", f"[white]{run_cmd(['sw_vers', '-buildVersion'])}[/white]"],
        ["Platform", f"[white]{platform.system()} {platform.machine()}[/white]"],
        ["User Lang", f"[dim]{os.environ.get('LANG', '—')}[/dim]"],
        ["Processor", f"[white]{run_cmd(['sysctl', '-n', 'machdep.cpu.brand_string'])}[/white]"],
        ["Cores", f"[green]{run_cmd(['sysctl', '-n', 'hw.ncpu'])}[/green]"],
        ["Memory", f"[green]{human_size(mem_bytes)}[/green]"],
        ["Uptime", f"[dim]{run_cmd(['uptime']).lstrip()}[/dim]"],
        ["Date", f"[dim]{datetime.now().strftime('%c')}[/dim]"],
        ["Home", f"[white]{os.environ.get('HOME', '—')}[/white]"],
        ["Shell", f"[white]{os.environ.get('SHELL', '—')}[/white]"],
        ["Term", f"[dim]{os.environ.get('TERM', '—')}[/dim]"],
        ["Python", f"[cyan]{sys.version.split()[0]}[/cyan]"],
    ],
)

summary_table = make_table(
    "Summary",
    ["Field", "Value"],
    [
        ["Root", f"[white]{root_path}[/white]"],
        ["Depth", f"{max_depth_reached} (actual)" if SCAN_EVERYTHING else f"{MAX_DEPTH} (max)"],
        ["Directories", f"[cyan]{dir_count}[/cyan]"],
        ["Files", f"[white]{file_count}[/white]"],
        ["Scanned File Size", f"[green]{human_size(scanned_file_bytes)}[/green]"],
        ["Root Total Size", f"[green]{human_size(root_total_size)}[/green]"],
        ["Root Items", str(len(root_entries))],
        ["Show Hidden", "Yes" if SHOW_HIDDEN else "No"],
        ["Follow Symlinks", "Yes" if FOLLOW_SYMLINKS else "No"],
        ["Top Files Count", str(TOP_N_FILES)],
        ["Top Dirs Count", str(TOP_N_DIRS)],
        ["Save Output", "Yes" if SAVE_OUTPUT else "No"],
        ["Output Dir", str(OUTPUT_DIR)],
        ["Output File", str(output_path) if output_path else "—"],
        ["Generated", datetime.now().strftime("%c")],
    ],
)

root_table = make_table(
    "Root Contents",
    ["#", "Type", "Name", "Size"],
    [
        [
            f"{i + 1:>2}",
            f"[cyan]{ICON['dir']} dir[/cyan]" if e.is_dir else f"[white]{ICON['file']} file[/white]",
            f"[bold cyan]{e.name}/[/bold cyan]" if e.is_dir else e.name,
            f"[green]{human_size(dir_sizes.get(e.path, 0) if e.is_dir else e.size)}[/green]",
        ]
        for i, e in enumerate(root_entries)
    ],
)

top_files_table = make_table(
    f"Top {TOP_N_FILES} Largest Files",
    ["#", "File", "Relative Path", "Size"],
    [
        [
            f"{i + 1:>2}",
            f"[white]{ICON['file']} {f[0].name}[/white]",
            rel_from_root(root_path, f[0]),
            f"[green]{human_size(f[1])}[/green]",
        ]
        for i, f in enumerate(top_files)
    ],
)

top_dirs_table = make_table(
    f"Top {TOP_N_DIRS} Largest Directories",
    ["#", "Directory", "Relative Path", "Total Size"],
    [
        [
            f"{i + 1:>2}",
            f"[bold cyan]{ICON['dir']} {p.name}/[/bold cyan]",
            rel_from_root(root_path, p),
            f"[green]{human_size(size)}[/green]",
        ]
        for i, (p, size) in enumerate(top_dirs)
    ],
)

console.print(
    Group(
        system_info_table,
        "",
        summary_table,
        "",
        root_table,
        "",
        top_files_table,
        "",
        top_dirs_table,
        "",
        Rule(f"[bold magenta]{ICON['section']} Full Tree[/bold magenta]", style="bright_black"),
        file_tree,
    )
)

if SAVE_OUTPUT and output_path is not None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(console.export_text(), encoding="utf-8")
    console.print()
    console.print(f"[bold green]Saved:[/bold green] {output_path}")
PY

afplay /System/Library/Sounds/Funk.aiff
afplay /System/Library/Sounds/Ping.aiff
```
