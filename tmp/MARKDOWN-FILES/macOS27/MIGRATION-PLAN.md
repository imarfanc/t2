# Migration Plan: MARKDOWN-FILES → macOS27

## Completed Migration & Deduplication Summary

1. **Directory scanner tool** — Split out from `go.md`, `deno.md`, and `uv.md` into modular files under `tools/directory-scanner/`:
   - [go-scanner.md](../tools/directory-scanner/go-scanner.md)
   - [deno-scanner.md](../tools/directory-scanner/deno-scanner.md) — Fixed hardcoded `/Users/arfan/...` path to use `Deno.env.get("HOME")`.
   - [uv-scanner.md](../tools/directory-scanner/uv-scanner.md)
   - [README.md](../tools/directory-scanner/README.md) — Created single shared documentation for scanner schema & design.
2. **Brew / curl install patterns** — Normalized into dedicated runtime files under `language-runtimes/`:
   - [go.md](../language-runtimes/go.md)
   - [deno.md](../language-runtimes/deno.md)
   - [uv.md](../language-runtimes/uv.md)
   - [rust.md](../language-runtimes/rust.md) — Rust toolchain (rustup & Xcode CLT reference) converted from macOS-reset-26.6 docs.
3. **Shell profile bootstrap (`.zprofile` / `.zshrc`)** — Extracted redundant profile bootstrap and backup patterns into [zprofile-zshrc.md](../shell-profile/zprofile-zshrc.md).
4. **Bootstrap & Setup Modules** (`_bootstrap-(htmd)/`, in `sort` order) — de-duplicated after the initial split had accidentally repeated the delete script in two files and collided sort numbers:
   - [first-boot.md](../_bootstrap-(htmd)/first-boot.md) (sort 1) — Safari, macOS desktop tweaks, quit-apps script, auto login.
   - [keyboard-trackpad.md](../_bootstrap-(htmd)/keyboard-trackpad.md) (sort 2) — macOS keyboard/trackpad `defaults write` settings (tap to click, key repeat, scroll bars, save dialogs). Renamed from `1.3 _ warp (keyboard,trackpad).md`; added frontmatter and a matching H1 (was mistitled "terminal (keyboard,trackpad)").
   - [terminal-first.md](../_bootstrap-(htmd)/terminal-first.md) (sort 3) — Open Terminal, install Xcode Command Line Tools.
   - [terminal-second.md](../_bootstrap-(htmd)/terminal-second.md) (sort 4) — Bulk directory/file deletion + `.DS_Store` cleanup (sole copy; removed the duplicate that had also been left in cleanup-and-repo.md).
   - [cleanup-and-repo.md](../_bootstrap-(htmd)/cleanup-and-repo.md) (sort 5) — Workspace creation, `t2` repo bootstrap, `empty.md` placeholders, Finder toolbar shortcuts. Also dropped a leftover `.zprofile222` typo in the profile-cat step.
   - [rust.md](../_bootstrap-(htmd)/rust.md) (sort 6) — Moved here from `language-runtimes/`; fixed its broken cross-reference (pointed at a nonexistent `../00-bootstrap/first-boot.md`, now correctly links to `./terminal-first.md`) and its off-by-one step numbering.
5. **OpenCode CLI tool** — Extracted Deno-unrelated `opencode` section into [opencode.md](../cli-tools/opencode.md).
6. **Empty stub & Source Cleanup** — Removed empty `1.3 _ three.md` and redundant directories `MARKDOWN-FILES/1` and `MARKDOWN-FILES/1alt`.

## Final Folder Structure

```text
macOS27/
  MIGRATION-PLAN.md
  _bootstrap-(htmd)/
    first-boot.md       ← sort 1: Safari, macOS Desktop, Quit Apps, Auto Login
    keyboard-trackpad.md← sort 2: Keyboard/trackpad defaults (tap to click, key repeat, etc.)
    terminal-first.md   ← sort 3: Terminal launch, Xcode CLT install
    terminal-second.md  ← sort 4: Bulk delete + .DS_Store cleanup
    cleanup-and-repo.md ← sort 5: Workspace setup, t2 repo download, Finder toolbar
    rust.md             ← sort 6: Rust toolchain (moved from language-runtimes/)
  package-managers/
    brew.md             ← Homebrew setup
  language-runtimes/
    go.md               ← Go installer
    deno.md             ← Deno installer
    uv.md               ← uv installer
  tools/directory-scanner/
    go-scanner.md       ← Go implementation
    deno-scanner.md     ← Deno implementation
    uv-scanner.md       ← Python/uv implementation
    README.md           ← Shared scanner documentation
  shell-profile/
    zprofile-zshrc.md   ← Shell profile & backup pattern
  cli-tools/
    opencode.md         ← Extracted opencode CLI setup
```
