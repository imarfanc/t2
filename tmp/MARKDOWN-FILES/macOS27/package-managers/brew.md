---
title: "brew"
sort: 1
category: "package-managers"
description: "Homebrew installation and configuration"
date: 2026-5-1
tags:
  - macOS
  - brew
  - homebrew
  - install
---

# Homebrew Installation

- [Homebrew Official Site](https://brew.sh)

## Installation

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
afplay /System/Library/Sounds/Funk.aiff
```

## Shell Configuration

```sh
echo >> "$HOME/.zprofile"
echo 'eval "$(/opt/homebrew/bin/brew shellenv zsh)"' >> "$HOME/.zprofile"
eval "$(/opt/homebrew/bin/brew shellenv zsh)"
source ~/.zshrc
cat "$HOME/.zprofile"
cat "$HOME/.zshrc"
```
