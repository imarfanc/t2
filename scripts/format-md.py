#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = ["rich", "mdformat", "mdformat-gfm", "mdformat-frontmatter", "mdformat-tables"]
# ///

"""Format Markdown with mdformat (GFM tables, frontmatter)."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import mdformat
from rich.console import Console
from rich.table import Table

DEFAULT_WRAP = "keep"  # "keep", "no", or an integer column


def normalize_source(source: str) -> str:
    return source.replace("\r\n", "\n")


def format_md(source: str, wrap: str) -> str:
    wrap_opt: str | int = wrap if wrap in ("keep", "no") else int(wrap)
    return mdformat.text(
        source,
        options={"wrap": wrap_opt, "number": True},
        extensions={"gfm", "frontmatter", "tables"},
    )


def iter_md_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file():
            if path.suffix.lower() in (".md", ".markdown"):
                files.append(path)
            continue
        if path.is_dir():
            files.extend(sorted(p for p in path.rglob("*") if p.suffix.lower() in (".md", ".markdown")))
    return files


def resolve_paths(raw_paths: list[Path | str]) -> list[Path]:
    if not raw_paths:
        return [Path(".")]
    return [Path(p) for p in raw_paths]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="Markdown files or directories (default: .)")
    parser.add_argument("--wrap", default=DEFAULT_WRAP, help='Wrap mode: "keep", "no", or a column number')
    parser.add_argument("--check", action="store_true", help="Exit 1 if any file would change")
    parser.add_argument("-i", "--in-place", action="store_true", help="Rewrite files in place")
    args = parser.parse_args()

    files = iter_md_files(resolve_paths(args.paths))
    if not files:
        print("No Markdown files found", file=sys.stderr)
        return 1

    changed = 0
    rows: list[tuple[Path, str]] = []
    for path in files:
        original = normalize_source(path.read_text(encoding="utf-8"))
        formatted = format_md(original, args.wrap)
        if formatted == original:
            rows.append((path, "ok"))
            continue
        changed += 1
        if args.check:
            rows.append((path, "would reformat"))
            continue
        if args.in_place:
            path.write_text(formatted, encoding="utf-8", newline="\n")
            rows.append((path, "formatted"))
        elif len(files) == 1:
            sys.stdout.write(formatted)
            rows.append((path, "printed"))
        else:
            rows.append((path, "skipped (pass -i)"))

    styles = {
        "ok": "green",
        "formatted": "cyan",
        "would reformat": "red",
        "printed": "dim",
        "skipped (pass -i)": "yellow",
    }
    table = Table(title="format-md --check" if args.check else "format-md")
    table.add_column("File")
    table.add_column("Status")
    for path, status in rows:
        table.add_row(str(path), f"[{styles[status]}]{status}[/]")
    Console(stderr=True).print(table)

    if args.check and changed:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
