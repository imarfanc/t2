---
name: cleanup-and-repo
sort: "5"
icon: "icon-park:five"
category: rare
subcategory: macOS-reset-26.7b
subsubcategory: _bootstrap
description: "Workspace setup, repo download, placeholder creation, and Finder layout setup"
date: 2026-5-1
tags:
  - macOS
  - reset
  - cleanup
  - repository
visibility: public
---

# Cleanup and Repo Bootstrap

File and directory deletion is covered in [terminal-second.md](./terminal-second.md); this file picks up from there with workspace setup.

## Setup Workspace & Create Placeholders

- [t2 Repo](https://github.com/imarfanc/t2)
- [t2 Zip Download](https://github.com/imarfanc/t2/archive/refs/heads/main.zip)
- [_arfan-vals-list.val.run_](https://arfan-vals-list.val.run)
- [_MARKDOWN-FILES/1/1.1 _ one.md_](https://github.com/imarfanc/warp-251224/blob/main/MARKDOWN-FILES/1/1.1%20_%20one.md)

```sh
mkdir -p ~/Developer/macos-reset
mkdir -p ~/Developer/gh

touch ~/.zprofile
touch ~/.zshrc

cat ~/.zprofile
cat ~/.zshrc

cd ~/Developer/gh
curl -fsSL \
  https://github.com/imarfanc/t2/archive/refs/heads/main.zip \
  -o t2.zip
unzip t2.zip
rm t2.zip

touch ~/Desktop/empty.md
touch ~/Developer/empty.md
touch ~/Documents/empty.md
touch ~/Downloads/empty.md

open ~/Desktop
open ~/Developer
open ~/Documents
open ~/Downloads
open ~
cd
```

## Fix Finder Toolbar & Layout

Customize the Finder toolbar by adding **Trash** and removing duplicate icons. Then, quickly apply view settings across open Finder windows with these keyboard shortcuts:

- `command + 3` (Column View)
- `ctrl + command + 0` (Group by None)
- `ctrl + command + 5` (Sort by Date Modified)
- `command + W` (Close Window)
