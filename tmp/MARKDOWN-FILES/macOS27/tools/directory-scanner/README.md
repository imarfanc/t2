---
title: "directory-scanner"
sort: 1
category: "tools"
description: "Overview of directory scanning utilities in Go, Deno, and Python (uv)"
date: 2026-5-1
tags:
  - scanner
  - directory
  - tools
---

# Directory Scanner Overview

The directory scanner is implemented in three runtimes. All versions implement identical scanning parameters, data structure tables, file/dir size aggregation, and formatting.

## Language Implementations

- [Go Scanner](go-scanner.md) — Uses Charm Go libraries (`lipgloss` + `bubbles/spinner`).
- [Deno Scanner](deno-scanner.md) — Native TypeScript with ANSI color formatting.
- [uv/Python Scanner](uv-scanner.md) — Python implementation using `rich` for formatting.

## Shared Default Configuration

| Parameter | Default Value | Description |
| --- | --- | --- |
| `ROOT` | `.` | Target directory to scan |
| `OUTPUT_DIR` | `~/Developer/macos-reset` | Output destination (dynamically expands `~` / `$HOME`) |
| `MAX_DEPTH` | `3` | Maximum tree traversal depth |
| `TOP_N_FILES` | `25` | Top N largest files to report |
| `TOP_N_DIRS` | `50` | Top N largest directories to report |
| `EXCLUDE_DIR_NAMES` | `.git`, `node_modules`, `.venv`, `__pycache__`, `.DS_Store` | Excluded folder names |
