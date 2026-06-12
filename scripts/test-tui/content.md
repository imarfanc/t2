# Markdown TUI

A **small terminal application** built from one Rust file.

## Features

- Renders headings, lists, and inline emphasis
- Uses only the Rust standard library
- Builds to `builds/test`
- Loads this content from `scripts/test-tui/content.md` at runtime

## How it works

The first two lines of `test.rs` are a shell launcher. When you execute
the file, `sh` runs the launcher, which recompiles the source with
`rustc` only if it changed, then runs the cached binary.

- No `Cargo.toml`, no `Cargo.lock`, no `target/` directory
- Recompiles only when the source is newer than the binary
- The binary lives in `builds/` and starts instantly

## Markdown support

This tiny renderer understands a small subset of Markdown:

- `#` and `##` headings
- Bullet lists with `-`
- **Bold** with double asterisks
- Inline `code` with backticks
- Block quotes with `>`

## Editing

Edit `content.md` and rerun — no recompile needed, since the file is
read at runtime.

> Press **q** to close this view.
