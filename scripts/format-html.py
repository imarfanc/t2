#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = ["rich"]
# ///

"""Format HTML in compact style: short elements on one line, longer ones expanded."""

from __future__ import annotations

import argparse
import re
import sys
from html.parser import HTMLParser
from pathlib import Path

from rich.console import Console
from rich.table import Table

DEFAULT_WIDTH = 250
INDENT = "  "

VOID_ELEMENTS = {
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
}
RAW_TEXT_ELEMENTS = {"script", "style", "pre", "textarea"}


class Node:
    def __init__(self, kind: str, data: str = "", attrs: list | None = None):
        self.kind = kind  # "element", "text", "comment", "doctype"
        self.data = data  # tag name or text content
        self.attrs = attrs or []
        self.children: list[Node] = []


class TreeBuilder(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=False)
        self.root = Node("element", "#root")
        self.stack = [self.root]

    def handle_decl(self, decl: str) -> None:
        self.stack[-1].children.append(Node("doctype", decl))

    def handle_comment(self, data: str) -> None:
        self.stack[-1].children.append(Node("comment", data))

    def handle_starttag(self, tag: str, attrs: list) -> None:
        node = Node("element", tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in VOID_ELEMENTS:
            self.stack.append(node)

    def handle_startendtag(self, tag: str, attrs: list) -> None:
        self.stack[-1].children.append(Node("element", tag, attrs))

    def handle_endtag(self, tag: str) -> None:
        for i in range(len(self.stack) - 1, 0, -1):
            if self.stack[i].data == tag:
                del self.stack[i:]
                return

    def handle_data(self, data: str) -> None:
        self.stack[-1].children.append(Node("text", data))

    def handle_entityref(self, name: str) -> None:
        self.handle_data(f"&{name};")

    def handle_charref(self, name: str) -> None:
        self.handle_data(f"&#{name};")


def render_attrs(attrs: list) -> str:
    parts = []
    for name, value in attrs:
        if value is None:
            parts.append(name)
        else:
            parts.append(f'{name}="{value}"')
    return (" " + " ".join(parts)) if parts else ""


def open_tag(node: Node) -> str:
    return f"<{node.data}{render_attrs(node.attrs)}>"


def collapse_text(text: str) -> str:
    return re.sub(r"\s+", " ", text)


def render_inline(node: Node) -> str | None:
    """Render node to a single line, or None if it must be multiline."""
    if node.kind == "text":
        return collapse_text(node.data).strip()
    if node.kind == "comment":
        return None if "\n" in node.data else f"<!--{node.data}-->"
    if node.kind == "doctype":
        return f"<!{node.data}>"
    if node.data in RAW_TEXT_ELEMENTS:
        raw = "".join(c.data for c in node.children)
        if "\n" in raw.strip() or not raw.strip() and not node.children:
            if raw.strip():
                return None
        return f"{open_tag(node)}{raw.strip()}</{node.data}>"
    if node.data in VOID_ELEMENTS:
        return open_tag(node)

    inner = ""
    for child in node.children:
        if child.kind == "text":
            inner += collapse_text(child.data)
            continue
        rendered = render_inline(child)
        if rendered is None:
            return None
        inner += rendered
    return f"{open_tag(node)}{inner.strip()}</{node.data}>"


def render_raw_block(node: Node, indent: str) -> list[str]:
    raw = "".join(c.data for c in node.children)
    content = raw.strip("\n").rstrip()
    lines = [f"{indent}{open_tag(node)}"]
    if content.strip():
        lines.extend(content.splitlines())
    lines.append(f"{indent}</{node.data}>")
    return lines


def render_block(node: Node, indent: str, max_width: int) -> list[str]:
    inline = render_inline(node)
    if inline is not None and len(indent) + len(inline) <= max_width:
        return [f"{indent}{inline}"] if inline else []

    if node.kind == "text":
        text = collapse_text(node.data).strip()
        return [f"{indent}{text}"] if text else []
    if node.kind == "comment":
        return [f"{indent}<!--{node.data}-->"]
    if node.kind == "doctype":
        return [f"{indent}<!{node.data}>"]
    if node.data in RAW_TEXT_ELEMENTS:
        return render_raw_block(node, indent)
    if node.data in VOID_ELEMENTS:
        return [f"{indent}{open_tag(node)}"]

    lines = [f"{indent}{open_tag(node)}"]
    for child in node.children:
        lines.extend(render_block(child, indent + INDENT, max_width))
    lines.append(f"{indent}</{node.data}>")
    return lines


def format_html(source: str, max_width: int = DEFAULT_WIDTH) -> str:
    builder = TreeBuilder()
    builder.feed(source)
    builder.close()
    lines: list[str] = []
    for child in builder.root.children:
        lines.extend(render_block(child, "", max_width))
    body = "\n".join(lines)
    return body + ("\n" if body else "")


def normalize_source(source: str) -> str:
    return source.replace("\r\n", "\n")


def stable_format(source: str, max_width: int = DEFAULT_WIDTH) -> str:
    current = normalize_source(source)
    for _ in range(8):
        nxt = format_html(current, max_width=max_width)
        if nxt == current:
            return current
        current = nxt
    return current


def iter_html_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file():
            if path.suffix.lower() in (".html", ".htm"):
                files.append(path)
            continue
        if path.is_dir():
            files.extend(sorted(p for p in path.rglob("*") if p.suffix.lower() in (".html", ".htm")))
    return files


def resolve_paths(raw_paths: list[Path | str]) -> list[Path]:
    if not raw_paths:
        return [Path(".")]
    return [Path(p) for p in raw_paths]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="HTML files or directories (default: .)")
    parser.add_argument("--width", type=int, default=DEFAULT_WIDTH, help="Max line width for compact elements")
    parser.add_argument("--check", action="store_true", help="Exit 1 if any file would change")
    parser.add_argument("-i", "--in-place", action="store_true", help="Rewrite files in place")
    args = parser.parse_args()

    files = iter_html_files(resolve_paths(args.paths))
    if not files:
        print("No HTML files found", file=sys.stderr)
        return 1

    changed = 0
    rows: list[tuple[Path, str]] = []
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
    table = Table(title="format-html --check" if args.check else "format-html")
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
