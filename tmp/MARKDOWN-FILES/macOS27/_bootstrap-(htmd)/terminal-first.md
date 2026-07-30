---
name: terminal-first
sort: "3"
icon: "glyphs-poly:three"
category: rare
subcategory: macOS-reset-26.7b
subsubcategory: _bootstrap
description: "Terminal configuration and Command Line Tools installation"
date: 2026-5-1
tags:
  - macOS
  - reset
  - terminal
  - xcode-clt
visibility: public
---

# Terminal First Setup

## Terminal

- Open `Terminal`

## Install Xcode Command Line Tools

Rust and build tools require Apple's C toolchain and linker:

```sh
xcode-select --install
```

## Silent Alternative via `softwareupdate` (No GUI Dialog)

- will need `sudo pass`

Paste this heredoc to install Command Line Tools silently without GUI prompts:

```sh
zsh <<'ZSH'
set -e

# Skip if already installed
if xcode-select -p >/dev/null 2>&1; then
  echo "✓ CLT already installed: $(xcode-select -p)"
  exit 0
fi

# Trick the system into listing the CLT package
TRIGGER=/tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress
touch "$TRIGGER"
trap 'rm -f "$TRIGGER"' EXIT

# Find the newest CLT label
CLT=$(softwareupdate -l 2>/dev/null \
  | grep -o 'Label: Command Line Tools for Xcode.*' \
  | sed 's/^Label: //' \
  | sort -V | tail -1)

if [ -z "$CLT" ]; then
  echo "✗ No CLT package found in softwareupdate catalog." >&2
  echo "  Fallback: run  xcode-select --install" >&2
  exit 1
fi

echo "Installing: $CLT"

# Install silently (needs sudo, no GUI)
sudo softwareupdate -i "$CLT" --verbose

# Verify
xcode-select -p >/dev/null 2>&1 \
  && echo "✓ Installed: $(xcode-select -p)" \
  || { echo "✗ Install failed." >&2; exit 1; }
ZSH
```
