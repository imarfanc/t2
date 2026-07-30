---
title: "zprofile-zshrc"
sort: 1
category: "shell-profile"
description: "Canonical Zsh profile initialization and backup conventions"
date: 2026-5-1
tags:
  - zsh
  - profile
  - zshrc
  - backup
---

# Shell Profile Initialization & Backup

## Initial Profile Bootstrap

Ensure standard shell profiles exist in `$HOME`:

```sh
touch ~/.zprofile
touch ~/.zshrc
```

## Profile Backup Pattern

When executing automated setup scripts, back up existing shell profiles before modification:

```sh
mkdir -p "$HOME/Developer/macos-reset/bak1"
cp "$HOME/.zprofile" "$HOME/Developer/macos-reset/bak1/zprofile.txt"
cp "$HOME/.zshrc" "$HOME/Developer/macos-reset/bak1/zshrc.txt"
afplay /System/Library/Sounds/Funk.aiff
source ~/.zshrc
```
