# dir-size-26.4.9

## script

```sh
uv run --with rich python3 - <<'PY'
#!/usr/bin/env python3
from __future__ import annotations

import fnmatch
import shutil
import subprocess
from collections import deque
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path

from rich.console import Console
from rich.live import Live
from rich.panel import Panel
from rich.progress import (
    BarColumn,
    Progress,
    SpinnerColumn,
    TextColumn,
    TimeElapsedColumn,
)
from rich.table import Table

console = Console()

# -------------------------------------------------------------------
# Hardcoded config
# -------------------------------------------------------------------
TARGET_PATH = "~"
TOP_LIMIT = 10
WORKERS = 16
MAX_DEPTH = 2  # parallelize children + grandchildren
SKIP_HIDDEN = False
ONE_FILE_SYSTEM = False
DU_TIMEOUT = 120

IGNORE_PATTERNS = [
    "node_modules",
    ".git",
    "Library/Caches",
]

# how many active paths to show in the live panel
ACTIVE_PATHS_LIMIT = 8


# -------------------------------------------------------------------
# Models
# -------------------------------------------------------------------
@dataclass(slots=True)
class EntrySize:
    path: Path
    size_bytes: int
    kind: str
    ok: bool
    error: str = ""


# -------------------------------------------------------------------
# Helpers
# -------------------------------------------------------------------
def human_bytes(num_bytes: int) -> str:
    units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
    value = float(num_bytes)
    for unit in units:
        if value < 1024 or unit == units[-1]:
            return f"{int(value)}B" if unit == "B" else f"{value:.1f}{unit}"
        value /= 1024
    return f"{num_bytes}B"


def classify_path(path: Path) -> str:
    try:
        if path.is_symlink():
            return "link"
        if path.is_dir():
            return "dir"
        if path.is_file():
            return "file"
        return "other"
    except OSError:
        return "other"


def normalized_rel_path(path: Path, root: Path) -> str:
    try:
        rel = path.relative_to(root)
        return rel.as_posix()
    except ValueError:
        return path.as_posix()


def should_ignore(
    path: Path, root: Path, ignore_patterns: list[str], skip_hidden: bool
) -> bool:
    name = path.name
    rel = normalized_rel_path(path, root)

    if skip_hidden and name.startswith("."):
        return True

    for pattern in ignore_patterns:
        normalized_pattern = pattern.strip("/")

        # exact name match, e.g. ".git", "node_modules"
        if name == normalized_pattern:
            return True

        # exact relative path match, e.g. "Library/Caches"
        if rel == normalized_pattern:
            return True

        # prefix relative path match, e.g. "Library/Caches/..."
        if rel.startswith(normalized_pattern + "/"):
            return True

        # wildcard support
        if fnmatch.fnmatch(name, pattern) or fnmatch.fnmatch(rel, pattern):
            return True

    return False


def du_size(path: Path, one_file_system: bool = False, timeout: int = 120) -> EntrySize:
    cmd = ["du", "-sk"]
    if one_file_system:
        cmd.append("-x")
    cmd.append(str(path))

    kind = classify_path(path)

    try:
        result = subprocess.run(
            cmd,
            check=True,
            text=True,
            capture_output=True,
            timeout=timeout,
        )
        first_field = result.stdout.strip().split(None, 1)[0]
        kib = int(first_field)
        return EntrySize(path=path, size_bytes=kib * 1024, kind=kind, ok=True)
    except subprocess.TimeoutExpired:
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error="timeout")
    except subprocess.CalledProcessError as e:
        stderr = (e.stderr or "").strip() or "du failed"
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error=stderr)
    except (ValueError, OSError) as e:
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error=str(e))


def print_disk_usage(path: Path) -> None:
    try:
        total, used, free = shutil.disk_usage(path)
    except OSError as e:
        console.print(f"[red]Cannot read disk usage for {path}: {e}[/red]")
        return

    used_pct = (used / total * 100) if total else 0.0

    table = Table(title=f"Disk Usage: {path}", show_header=True, header_style="bold")
    table.add_column("Used", justify="right")
    table.add_column("Total", justify="right")
    table.add_column("Use %", justify="right")
    table.add_column("Free", justify="right")
    table.add_row(
        human_bytes(used),
        human_bytes(total),
        f"{used_pct:.1f}%",
        human_bytes(free),
    )
    console.print(table)


def get_top_level_entries(
    root: Path, skip_hidden: bool, ignore_patterns: list[str]
) -> list[Path]:
    try:
        entries = []
        for p in root.iterdir():
            if should_ignore(p, root, ignore_patterns, skip_hidden):
                continue
            entries.append(p)
        return entries
    except (PermissionError, OSError) as e:
        console.print(f"[red]Cannot read {root}: {e}[/red]")
        return []


def get_dirs_to_scan(
    root: Path, max_depth: int, skip_hidden: bool, ignore_patterns: list[str]
) -> list[Path]:
    """
    Collect directories under root up to max_depth levels deep.
    Depth 1 = direct children
    Depth 2 = grandchildren
    """
    if max_depth <= 0:
        return []

    dirs_to_scan: list[Path] = []
    queue: deque[tuple[Path, int]] = deque([(root, 0)])

    while queue:
        current, depth = queue.popleft()
        if depth >= max_depth:
            continue

        try:
            for p in current.iterdir():
                if should_ignore(p, root, ignore_patterns, skip_hidden):
                    continue
                if p.is_dir() and not p.is_symlink():
                    dirs_to_scan.append(p)
                    if depth + 1 < max_depth:
                        queue.append((p, depth + 1))
        except (PermissionError, OSError):
            continue

    # dedupe while preserving order
    seen: set[Path] = set()
    deduped: list[Path] = []
    for p in dirs_to_scan:
        if p not in seen:
            seen.add(p)
            deduped.append(p)
    return deduped


def make_live_renderable(
    root: Path,
    total: int,
    completed: int,
    active_paths: list[str],
) -> Panel:
    table = Table(show_header=False, box=None, pad_edge=False)
    table.add_column(style="cyan", no_wrap=True)
    table.add_column(style="white")

    table.add_row("Root", str(root))
    table.add_row("Progress", f"{completed}/{total}")
    table.add_row("Active", str(len(active_paths)))

    if active_paths:
        shown = active_paths[:ACTIVE_PATHS_LIMIT]
        text = "\n".join(f"• {p}" for p in shown)
        if len(active_paths) > ACTIVE_PATHS_LIMIT:
            text += f"\n• ... +{len(active_paths) - ACTIVE_PATHS_LIMIT} more"
    else:
        text = "Waiting..."

    outer = Table.grid(padding=(0, 1))
    outer.add_row(table)
    outer.add_row("")
    outer.add_row("[bold]Currently scanning[/bold]")
    outer.add_row(text)

    return Panel(outer, title="Scan Activity", border_style="blue")


def scan_top_entries(
    root: Path,
    limit: int = 10,
    workers: int = 16,
    max_depth: int = 2,
    skip_hidden: bool = False,
    one_file_system: bool = False,
    du_timeout: int = 120,
    ignore_patterns: list[str] | None = None,
) -> None:
    ignore_patterns = ignore_patterns or []

    top_level = get_top_level_entries(
        root, skip_hidden=skip_hidden, ignore_patterns=ignore_patterns
    )
    if not top_level:
        console.print(f"[yellow]No entries found in {root}[/yellow]")
        return

    # Include extra nested directories for better parallelism / visibility.
    nested_dirs = get_dirs_to_scan(
        root=root,
        max_depth=max_depth,
        skip_hidden=skip_hidden,
        ignore_patterns=ignore_patterns,
    )

    all_paths = top_level + nested_dirs

    # dedupe while preserving order
    seen: set[Path] = set()
    deduped_all: list[Path] = []
    for p in all_paths:
        if p not in seen:
            seen.add(p)
            deduped_all.append(p)
    all_paths = deduped_all

    results: dict[str, EntrySize] = {}
    active_paths: set[str] = set()

    progress = Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TextColumn("{task.completed}/{task.total}"),
        TimeElapsedColumn(),
        console=console,
    )

    with Live(
        make_live_renderable(root, len(all_paths), 0, []),
        refresh_per_second=8,
        console=console,
    ) as live:
        with progress:
            task = progress.add_task("Scanning directories", total=len(all_paths))

            with ThreadPoolExecutor(max_workers=workers) as executor:
                future_to_path = {}
                for path in all_paths:
                    future = executor.submit(du_size, path, one_file_system, du_timeout)
                    future_to_path[future] = path
                    active_paths.add(str(path))
                    live.update(
                        make_live_renderable(
                            root=root,
                            total=len(all_paths),
                            completed=progress.tasks[0].completed,
                            active_paths=sorted(active_paths),
                        )
                    )

                for future in as_completed(future_to_path):
                    path = future_to_path[future]
                    result = future.result()
                    results[str(path)] = result
                    active_paths.discard(str(path))
                    progress.update(task, advance=1)
                    live.update(
                        make_live_renderable(
                            root=root,
                            total=len(all_paths),
                            completed=progress.tasks[0].completed,
                            active_paths=sorted(active_paths),
                        )
                    )

    # Final top-level sizes:
    # keep the real top-level `du` result, since that already includes descendants.
    top_results: list[EntrySize] = []
    bad_results: list[EntrySize] = []

    for p in top_level:
        result = results.get(str(p))
        if result is None:
            result = EntrySize(
                path=p,
                size_bytes=0,
                kind=classify_path(p),
                ok=False,
                error="missing result",
            )
        if result.ok:
            top_results.append(result)
        else:
            bad_results.append(result)

    top_results.sort(key=lambda r: r.size_bytes, reverse=True)

    table = Table(title=f"Top {min(limit, len(top_results))} entries in {root}")
    table.add_column("#", justify="right", style="dim")
    table.add_column("Size", justify="right", style="magenta")
    table.add_column("Type", style="cyan")
    table.add_column("Path", style="green")

    for idx, item in enumerate(top_results[:limit], start=1):
        table.add_row(
            str(idx),
            human_bytes(item.size_bytes),
            item.kind,
            str(item.path),
        )

    console.print(table)

    summary = Table(title="Scan Summary", show_header=True, header_style="bold")
    summary.add_column("Metric")
    summary.add_column("Value", justify="right")
    summary.add_row("Top-level entries scanned", str(len(top_level)))
    summary.add_row("Parallel scan paths", str(len(all_paths)))
    summary.add_row("Nested dirs added", str(max(0, len(all_paths) - len(top_level))))
    summary.add_row("Successful top-level", str(len(top_results)))
    summary.add_row("Failed top-level", str(len(bad_results)))
    summary.add_row(
        "Largest entry",
        f"{human_bytes(top_results[0].size_bytes)}  ({top_results[0].path.name})"
        if top_results
        else "n/a",
    )
    summary.add_row("Max depth", str(max_depth))
    summary.add_row("Workers", str(workers))
    summary.add_row(
        "Ignore patterns", ", ".join(ignore_patterns) if ignore_patterns else "none"
    )
    console.print(summary)

    if bad_results:
        err_table = Table(title="Top-level entries with errors")
        err_table.add_column("Path", style="yellow")
        err_table.add_column("Error", style="red")
        for item in sorted(bad_results, key=lambda r: str(r.path)):
            err_table.add_row(str(item.path), item.error[:200])
        console.print(err_table)


def main() -> None:
    target_path = Path(TARGET_PATH).expanduser().resolve()

    console.rule("[bold blue]Disk Usage Summary[/bold blue]")

    print_disk_usage(Path("/"))
    if target_path != Path("/"):
        print_disk_usage(target_path)

    scan_top_entries(
        root=target_path,
        limit=TOP_LIMIT,
        workers=WORKERS,
        max_depth=MAX_DEPTH,
        skip_hidden=SKIP_HIDDEN,
        one_file_system=ONE_FILE_SYSTEM,
        du_timeout=DU_TIMEOUT,
        ignore_patterns=IGNORE_PATTERNS,
    )


if __name__ == "__main__":
    main()

PY
```

```py
#!/usr/bin/env python3
from __future__ import annotations

import fnmatch
import shutil
import subprocess
from collections import deque
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path

from rich.console import Console
from rich.live import Live
from rich.panel import Panel
from rich.progress import (
    BarColumn,
    Progress,
    SpinnerColumn,
    TextColumn,
    TimeElapsedColumn,
)
from rich.table import Table

console = Console()

# -------------------------------------------------------------------
# Hardcoded config
# -------------------------------------------------------------------
TARGET_PATH = "~"
TOP_LIMIT = 10
WORKERS = 16
MAX_DEPTH = 2  # parallelize children + grandchildren
SKIP_HIDDEN = False
ONE_FILE_SYSTEM = False
DU_TIMEOUT = 120

IGNORE_PATTERNS = [
    "node_modules",
    ".git",
    "Library/Caches",
]

# how many active paths to show in the live panel
ACTIVE_PATHS_LIMIT = 8


# -------------------------------------------------------------------
# Models
# -------------------------------------------------------------------
@dataclass(slots=True)
class EntrySize:
    path: Path
    size_bytes: int
    kind: str
    ok: bool
    error: str = ""


# -------------------------------------------------------------------
# Helpers
# -------------------------------------------------------------------
def human_bytes(num_bytes: int) -> str:
    units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
    value = float(num_bytes)
    for unit in units:
        if value < 1024 or unit == units[-1]:
            return f"{int(value)}B" if unit == "B" else f"{value:.1f}{unit}"
        value /= 1024
    return f"{num_bytes}B"


def classify_path(path: Path) -> str:
    try:
        if path.is_symlink():
            return "link"
        if path.is_dir():
            return "dir"
        if path.is_file():
            return "file"
        return "other"
    except OSError:
        return "other"


def normalized_rel_path(path: Path, root: Path) -> str:
    try:
        rel = path.relative_to(root)
        return rel.as_posix()
    except ValueError:
        return path.as_posix()


def should_ignore(
    path: Path, root: Path, ignore_patterns: list[str], skip_hidden: bool
) -> bool:
    name = path.name
    rel = normalized_rel_path(path, root)

    if skip_hidden and name.startswith("."):
        return True

    for pattern in ignore_patterns:
        normalized_pattern = pattern.strip("/")

        # exact name match, e.g. ".git", "node_modules"
        if name == normalized_pattern:
            return True

        # exact relative path match, e.g. "Library/Caches"
        if rel == normalized_pattern:
            return True

        # prefix relative path match, e.g. "Library/Caches/..."
        if rel.startswith(normalized_pattern + "/"):
            return True

        # wildcard support
        if fnmatch.fnmatch(name, pattern) or fnmatch.fnmatch(rel, pattern):
            return True

    return False


def du_size(path: Path, one_file_system: bool = False, timeout: int = 120) -> EntrySize:
    cmd = ["du", "-sk"]
    if one_file_system:
        cmd.append("-x")
    cmd.append(str(path))

    kind = classify_path(path)

    try:
        result = subprocess.run(
            cmd,
            check=True,
            text=True,
            capture_output=True,
            timeout=timeout,
        )
        first_field = result.stdout.strip().split(None, 1)[0]
        kib = int(first_field)
        return EntrySize(path=path, size_bytes=kib * 1024, kind=kind, ok=True)
    except subprocess.TimeoutExpired:
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error="timeout")
    except subprocess.CalledProcessError as e:
        stderr = (e.stderr or "").strip() or "du failed"
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error=stderr)
    except (ValueError, OSError) as e:
        return EntrySize(path=path, size_bytes=0, kind=kind, ok=False, error=str(e))


def print_disk_usage(path: Path) -> None:
    try:
        total, used, free = shutil.disk_usage(path)
    except OSError as e:
        console.print(f"[red]Cannot read disk usage for {path}: {e}[/red]")
        return

    used_pct = (used / total * 100) if total else 0.0

    table = Table(title=f"Disk Usage: {path}", show_header=True, header_style="bold")
    table.add_column("Used", justify="right")
    table.add_column("Total", justify="right")
    table.add_column("Use %", justify="right")
    table.add_column("Free", justify="right")
    table.add_row(
        human_bytes(used),
        human_bytes(total),
        f"{used_pct:.1f}%",
        human_bytes(free),
    )
    console.print(table)


def get_top_level_entries(
    root: Path, skip_hidden: bool, ignore_patterns: list[str]
) -> list[Path]:
    try:
        entries = []
        for p in root.iterdir():
            if should_ignore(p, root, ignore_patterns, skip_hidden):
                continue
            entries.append(p)
        return entries
    except (PermissionError, OSError) as e:
        console.print(f"[red]Cannot read {root}: {e}[/red]")
        return []


def get_dirs_to_scan(
    root: Path, max_depth: int, skip_hidden: bool, ignore_patterns: list[str]
) -> list[Path]:
    """
    Collect directories under root up to max_depth levels deep.
    Depth 1 = direct children
    Depth 2 = grandchildren
    """
    if max_depth <= 0:
        return []

    dirs_to_scan: list[Path] = []
    queue: deque[tuple[Path, int]] = deque([(root, 0)])

    while queue:
        current, depth = queue.popleft()
        if depth >= max_depth:
            continue

        try:
            for p in current.iterdir():
                if should_ignore(p, root, ignore_patterns, skip_hidden):
                    continue
                if p.is_dir() and not p.is_symlink():
                    dirs_to_scan.append(p)
                    if depth + 1 < max_depth:
                        queue.append((p, depth + 1))
        except (PermissionError, OSError):
            continue

    # dedupe while preserving order
    seen: set[Path] = set()
    deduped: list[Path] = []
    for p in dirs_to_scan:
        if p not in seen:
            seen.add(p)
            deduped.append(p)
    return deduped


def make_live_renderable(
    root: Path,
    total: int,
    completed: int,
    active_paths: list[str],
) -> Panel:
    table = Table(show_header=False, box=None, pad_edge=False)
    table.add_column(style="cyan", no_wrap=True)
    table.add_column(style="white")

    table.add_row("Root", str(root))
    table.add_row("Progress", f"{completed}/{total}")
    table.add_row("Active", str(len(active_paths)))

    if active_paths:
        shown = active_paths[:ACTIVE_PATHS_LIMIT]
        text = "\n".join(f"• {p}" for p in shown)
        if len(active_paths) > ACTIVE_PATHS_LIMIT:
            text += f"\n• ... +{len(active_paths) - ACTIVE_PATHS_LIMIT} more"
    else:
        text = "Waiting..."

    outer = Table.grid(padding=(0, 1))
    outer.add_row(table)
    outer.add_row("")
    outer.add_row("[bold]Currently scanning[/bold]")
    outer.add_row(text)

    return Panel(outer, title="Scan Activity", border_style="blue")


def scan_top_entries(
    root: Path,
    limit: int = 10,
    workers: int = 16,
    max_depth: int = 2,
    skip_hidden: bool = False,
    one_file_system: bool = False,
    du_timeout: int = 120,
    ignore_patterns: list[str] | None = None,
) -> None:
    ignore_patterns = ignore_patterns or []

    top_level = get_top_level_entries(
        root, skip_hidden=skip_hidden, ignore_patterns=ignore_patterns
    )
    if not top_level:
        console.print(f"[yellow]No entries found in {root}[/yellow]")
        return

    # Include extra nested directories for better parallelism / visibility.
    nested_dirs = get_dirs_to_scan(
        root=root,
        max_depth=max_depth,
        skip_hidden=skip_hidden,
        ignore_patterns=ignore_patterns,
    )

    all_paths = top_level + nested_dirs

    # dedupe while preserving order
    seen: set[Path] = set()
    deduped_all: list[Path] = []
    for p in all_paths:
        if p not in seen:
            seen.add(p)
            deduped_all.append(p)
    all_paths = deduped_all

    results: dict[str, EntrySize] = {}
    active_paths: set[str] = set()

    progress = Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TextColumn("{task.completed}/{task.total}"),
        TimeElapsedColumn(),
        console=console,
    )

    with Live(
        make_live_renderable(root, len(all_paths), 0, []),
        refresh_per_second=8,
        console=console,
    ) as live:
        with progress:
            task = progress.add_task("Scanning directories", total=len(all_paths))

            with ThreadPoolExecutor(max_workers=workers) as executor:
                future_to_path = {}
                for path in all_paths:
                    future = executor.submit(du_size, path, one_file_system, du_timeout)
                    future_to_path[future] = path
                    active_paths.add(str(path))
                    live.update(
                        make_live_renderable(
                            root=root,
                            total=len(all_paths),
                            completed=progress.tasks[0].completed,
                            active_paths=sorted(active_paths),
                        )
                    )

                for future in as_completed(future_to_path):
                    path = future_to_path[future]
                    result = future.result()
                    results[str(path)] = result
                    active_paths.discard(str(path))
                    progress.update(task, advance=1)
                    live.update(
                        make_live_renderable(
                            root=root,
                            total=len(all_paths),
                            completed=progress.tasks[0].completed,
                            active_paths=sorted(active_paths),
                        )
                    )

    # Final top-level sizes:
    # keep the real top-level `du` result, since that already includes descendants.
    top_results: list[EntrySize] = []
    bad_results: list[EntrySize] = []

    for p in top_level:
        result = results.get(str(p))
        if result is None:
            result = EntrySize(
                path=p,
                size_bytes=0,
                kind=classify_path(p),
                ok=False,
                error="missing result",
            )
        if result.ok:
            top_results.append(result)
        else:
            bad_results.append(result)

    top_results.sort(key=lambda r: r.size_bytes, reverse=True)

    table = Table(title=f"Top {min(limit, len(top_results))} entries in {root}")
    table.add_column("#", justify="right", style="dim")
    table.add_column("Size", justify="right", style="magenta")
    table.add_column("Type", style="cyan")
    table.add_column("Path", style="green")

    for idx, item in enumerate(top_results[:limit], start=1):
        table.add_row(
            str(idx),
            human_bytes(item.size_bytes),
            item.kind,
            str(item.path),
        )

    console.print(table)

    summary = Table(title="Scan Summary", show_header=True, header_style="bold")
    summary.add_column("Metric")
    summary.add_column("Value", justify="right")
    summary.add_row("Top-level entries scanned", str(len(top_level)))
    summary.add_row("Parallel scan paths", str(len(all_paths)))
    summary.add_row("Nested dirs added", str(max(0, len(all_paths) - len(top_level))))
    summary.add_row("Successful top-level", str(len(top_results)))
    summary.add_row("Failed top-level", str(len(bad_results)))
    summary.add_row(
        "Largest entry",
        f"{human_bytes(top_results[0].size_bytes)}  ({top_results[0].path.name})"
        if top_results
        else "n/a",
    )
    summary.add_row("Max depth", str(max_depth))
    summary.add_row("Workers", str(workers))
    summary.add_row(
        "Ignore patterns", ", ".join(ignore_patterns) if ignore_patterns else "none"
    )
    console.print(summary)

    if bad_results:
        err_table = Table(title="Top-level entries with errors")
        err_table.add_column("Path", style="yellow")
        err_table.add_column("Error", style="red")
        for item in sorted(bad_results, key=lambda r: str(r.path)):
            err_table.add_row(str(item.path), item.error[:200])
        console.print(err_table)


def main() -> None:
    target_path = Path(TARGET_PATH).expanduser().resolve()

    console.rule("[bold blue]Disk Usage Summary[/bold blue]")

    print_disk_usage(Path("/"))
    if target_path != Path("/"):
        print_disk_usage(target_path)

    scan_top_entries(
        root=target_path,
        limit=TOP_LIMIT,
        workers=WORKERS,
        max_depth=MAX_DEPTH,
        skip_hidden=SKIP_HIDDEN,
        one_file_system=ONE_FILE_SYSTEM,
        du_timeout=DU_TIMEOUT,
        ignore_patterns=IGNORE_PATTERNS,
    )


if __name__ == "__main__":
    main()
```
