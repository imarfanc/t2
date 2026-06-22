# article-26.2.7_Sat_8.03pm

## open Safari

- https://arfan.cc.cc
- https://arfan-fast-md.val.run/article/article-4.6

## open terminal

```bash
xcode-select --install

open "x-apple.systempreferences:com.apple.SystemAbout-Settings.extension"
open "/System/Applications/Utilities/Activity Monitor.app"

open -a Safari \
  "https://arfan-fast-md.val.run/article/article-2.1" \
  "https://desktop.github.com/download/" \
  "https://www.warp.dev" \
  "https://duck.ai/" \
  "https://t3.chat/" \
  "https://github.com/imarfanc?tab=repositories" \
  "https://github.com/imarfanc/macOS-25.10.15/tree/main/projects/reset-macOS" \
  
```

## open Warp

* spotlight search for `warp`
* sign in using Google (look into changing auth)

## create dev folder & sh file

```bash
mkdir -p ~/Developer
mkdir -p ~/Developer/gh
mkdir -p ~/Developer/vt
mkdir -p ~/Developer/init-macOS
cd ~/Developer/init-macOS
touch dir-list.sh
# nano dir-list.sh
# open dir-list.sh
touch ~/Desktop/empty.md
touch ~/Developer/empty.md
touch ~/Documents/empty.md
touch ~/Downloads/empty.md
```

## finder

```bash
open -a Finder
open ~/
open ~/Desktop
open ~/Developer 
open ~/Documents
open ~/Downloads
```

- change view to column View
- change sort to date modified
- add trash icon in toolbar
- [ ] set Finder Settings

## install brew

- https://brew.sh

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

## install brew and apps using brew

### 1. install brew
[Homebrew](https://brew.sh/)

```bash
echo -e "ETA: ~4 mins."
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

add to .zshrc

```bash

echo >> /Users/arfan/.zprofile
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> /Users/arfan/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

### 2. check tools & versions

```bash
brew update
brew outdated
brew cleanup
brew upgrade
brew cleanup
brew --version
brew doctor
brew list -1
lsd -1a /Applications
```

### 3. install warp & keyboard maestro

[warp](https://www.warp.dev/)
- make notifications persistent

```bash
brew install lsd
brew install tree
# brew install --cask warp
brew install --cask keyboard-maestro
brew install --cask whatsapp
# brew install --cask chromium
# brew install --cask shortcat
brew install --cask setapp
# brew install --cask hyperkey
# brew install --cask hammerspoon
brew install --cask font-hack-nerd-font
brew install --cask zed
brew install --cask zen
brew install --cask visual-studio-code
brew install --cask helium-browser
brew install --cask google-chrome
brew install --cask obsidian
```
