#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = ["tinycss2", "rich"]
# ///

"""Format CSS in compact style: short rules on one line, longer rules expanded."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import tinycss2
from rich.console import Console
from rich.table import Table
from tinycss2 import ast

DEFAULT_WIDTH = 250
ROOT_PACK_WIDTH = 85
INDENT = "  "


def normalize_value(value: str) -> str:
    value = re.sub(r"\s+", " ", value.strip())
    value = re.sub(r"(?<![0-9.])0\.(\d+)(rem|em)\b", r".\1\2", value)
    return value


def serialize(tokens: list) -> str:
    return tinycss2.serialize(tokens).strip()


def declarations(rule_content: list) -> list[ast.Declaration]:
    decls: list[ast.Declaration] = []
    for block in tinycss2.parse_declaration_list(rule_content, skip_comments=True, skip_whitespace=True):
        if isinstance(block, ast.Declaration):
            decls.append(block)
    return decls


def split_selectors(selector: str) -> str:
    if ", " not in selector:
        return selector
    return ",\n".join(part.strip() for part in selector.split(","))


def format_decl_block(decl_strings: list[str], max_width: int, prefix: str) -> str:
    if not decl_strings:
        return f"{prefix} {{}}"

    single_body = "; ".join(decl_strings) + ";"
    single_line = f"{prefix} {{ {single_body} }}"
    if len(single_line) <= max_width:
        return single_line

    selector = split_selectors(prefix)
    lines = [f"{selector} {{"]
    for decl in decl_strings:
        lines.append(f"{INDENT}{decl};")
    lines.append("}")
    return "\n".join(lines)


def format_root_rule(decl_strings: list[str]) -> str:
    if not decl_strings:
        return ":root {}"

    lines = [":root {"]
    current = INDENT
    for decl in decl_strings:
        piece = f"{decl}; "
        if current == INDENT and len(lines) == 1:
            current += piece
        elif len(current) + len(piece) > ROOT_PACK_WIDTH:
            lines.append(current.rstrip())
            current = INDENT + piece
        else:
            current += piece
    lines.append(current.rstrip())
    lines.append("}")
    return "\n".join(lines)


def format_qualified_rule(rule: ast.QualifiedRule, max_width: int) -> str:
    selector = serialize(rule.prelude)
    decl_strings = [
        f"{decl.lower_name}: {normalize_value(serialize(decl.value))}"
        + (" !important" if decl.important else "")
        for decl in declarations(rule.content)
    ]

    if selector.strip() == ":root":
        return format_root_rule(decl_strings)

    return format_decl_block(decl_strings, max_width, selector)


def format_at_rule(rule: ast.AtRule, max_width: int) -> str:
    prelude = serialize(rule.prelude)
    header = f"@{rule.at_keyword} {prelude}".strip() if prelude else f"@{rule.at_keyword}"

    if rule.content is None:
        return f"{header};"

    if isinstance(rule.content, list):
        inner = format_rules(rule.content, max_width, indent=INDENT)
        indented = "\n".join(f"{INDENT}{line}" if line else line for line in inner.splitlines())
        return f"{header} {{\n{indented}\n}}"

    return format_decl_block(
        [
            f"{decl.lower_name}: {normalize_value(serialize(decl.value))}"
            + (" !important" if decl.important else "")
            for decl in declarations(rule.content)
        ],
        max_width,
        header,
    )


def format_rules(rules: list, max_width: int, indent: str = "") -> str:
    chunks: list[str] = []
    prev_was_root = False

    for rule in rules:
        if isinstance(rule, ast.Comment):
            chunks.append(f"/*{rule.value}*/")
            prev_was_root = False
            continue
        if isinstance(rule, ast.QualifiedRule):
            formatted = format_qualified_rule(rule, max_width)
            if prev_was_root:
                chunks.append("")
            chunks.append(formatted)
            prev_was_root = serialize(rule.prelude).strip() == ":root"
            continue
        if isinstance(rule, ast.AtRule):
            chunks.append(format_at_rule(rule, max_width))
            prev_was_root = False

    body = "\n".join(chunks)
    if indent:
        return body
    return body


def format_css(source: str, max_width: int = DEFAULT_WIDTH) -> str:
    rules = tinycss2.parse_stylesheet(source, skip_comments=False, skip_whitespace=True)
    body = format_rules(rules, max_width)
    return body + ("\n" if body else "")


def normalize_source(source: str) -> str:
    return source.replace("\r\n", "\n")


def stable_format(source: str, max_width: int = DEFAULT_WIDTH) -> str:
    current = normalize_source(source)
    for _ in range(8):
        nxt = format_css(current, max_width=max_width)
        if nxt == current:
            return current
        current = nxt
    return current


def iter_css_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file():
            if path.suffix.lower() == ".css":
                files.append(path)
            continue
        if path.is_dir():
            files.extend(sorted(path.rglob("*.css")))
    return files


def resolve_paths(raw_paths: list[Path | str]) -> list[Path]:
    if not raw_paths:
        return [Path(".")]
    return [Path(p) for p in raw_paths]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="CSS files or directories (default: .)")
    parser.add_argument("--width", type=int, default=DEFAULT_WIDTH, help="Max line width for compact rules")
    parser.add_argument("--check", action="store_true", help="Exit 1 if any file would change")
    parser.add_argument("-i", "--in-place", action="store_true", help="Rewrite files in place")
    args = parser.parse_args()

    files = iter_css_files(resolve_paths(args.paths))
    if not files:
        print("No CSS files found", file=sys.stderr)
        return 1

    changed = 0
    rows: list[tuple[Path, str]] = []  # (path, status)
    for path in files:
        original = normalize_source(path.read_text(encoding="utf-8"))
        formatted = stable_format(original, max_width=args.width)
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
    table = Table(title="format-css --check" if args.check else "format-css")
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
