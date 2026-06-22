---
title: "brew"
sort: 5
category: "macOS reset"
description: "install brew"
date: 2026-5-1
tags:
    - macOS
    - reset
    - brew
    - install
---

# install brew

## brew

- <https://brew.sh>

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
afplay /System/Library/Sounds/Funk.aiff
```

```sh
echo >> "$HOME/.zprofile"
echo 'eval "$(/opt/homebrew/bin/brew shellenv zsh)"' >> "$HOME/.zprofile"
eval "$(/opt/homebrew/bin/brew shellenv zsh)"
source ~/.zshrc
cat "$HOME/.zprofile"
cat "$HOME/.zshrc"
```
