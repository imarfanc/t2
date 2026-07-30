---
title: "opencode"
sort: 1
category: "cli-tools"
description: "OpenCode CLI setup and directory environment initialization"
date: 2026-5-1
tags:
  - opencode
  - cli
  - setup
---

# OpenCode CLI Setup

Initialize developer workspace directories and trigger `opencode`:

```bash
printf '%s\n' '# zsh config' >> ~/.zshrc
mkdir -p ~/Developer/gh
mkdir -p ~/Developer/tmp1
mkdir -p ~/Developer/local
cd ~/Developer/gh/

mkdir -p "$HOME/Developer/macos-reset/bak1"
cp "$HOME/.zprofile" "$HOME/Developer/macos-reset/bak1/zprofile.txt"
cp "$HOME/.zshrc" "$HOME/Developer/macos-reset/bak1/zshrc.txt"
afplay /System/Library/Sounds/Funk.aiff
source ~/.zshrc
opencode
```
