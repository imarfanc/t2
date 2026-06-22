# shell-2.1

- https://github.com/imarfanc/warp-251224
- https://github.com/imarfanc/warp-251224/archive/refs/heads/main.zip

## after safari

- in Safari
    - Settings
        - Uncheck - `open Safe downloads`
        - change Search to `DuckDuckGo`
        - check - `show features for web developers`
        - check - `allow JS from Apple Events`

- open terminal
    - `cmd+space` , `terminal`
    - settings
        - [ ] Cursor - set to `Vertival Bar`
        - [ ] Window - set to `120` , `45`
        - [ ] check `Use Option as Meta Key`
    - open settings
        - about `open "x-apple.systempreferences:com.apple.SystemAbout-Settings.extension"`
        - change computer name
    - open Spotlight
        - enable *Clipboard History*

## install stuff using websim project

- https://desktop.github.com/download/
- https://browser-desktop--synth.on.websim.com/
- https://www.warp.dev

```bash

xcode-select --install

open "x-apple.systempreferences:com.apple.SystemAbout-Settings.extension"
open "/System/Applications/Utilities/Activity Monitor.app"

open -a Safari \
  "https://arfan-fast-md.val.run/article/article-2.1" \
  "https://desktop.github.com/download/" \
  "https://www.keyboardmaestro.com/main/" \
  "https://browser-desktop--synth.on.websim.com/" \
  "https://www.warp.dev" \
  "https://duck.ai/" \
  "https://t3.chat/" \
  "https://www.copilot.com" \
  "https://mail.google.com/mail/u/0/#all" \
  "https://github.com/imarfanc?tab=repositories" \
  "https://github.com/imarfanc/macOS-25.10.15/tree/main/projects/reset-macOS" \
  "https://chrome-extensions-directory--synth.on.websim.com" \
  "https://imarfanc-public-repositories--synth.on.websim.com" \
  "https://two-column-layout-skeleton--synth.on.websim.com"
```

## open terminal

* spotlight search for `warp`

## create dev folder & sh file

```bash
mkdir -p ~/Developer
mkdir -p ~/Developer/init-macOS
cd ~/Developer/init-macOS
touch dir-list.sh
# nano dir-list.sh
# open dir-list.sh
```

## change to columns & date modified & add trash icon

```bash
open -a Finder
open ~/
open ~/Desktop
open ~/Developer 
open ~/Documents
open ~/Downloads
```

## open some Apps

```bash
open "/Applications/GitHub Desktop.app"
open "/Applications/Keyboard Maestro.app"

open /System/Applications/Clock.app
open /System/Applications/Shortcuts.app
open "/System/Applications/System Settings.app"
open "/System/Applications/Utilities/Activity Monitor.app"
open /System/Applications/VoiceMemos.app
open /System/Applications/Messages.app
open /System/Applications/Photos.app
open /System/Applications/Notes.app
```

## clear shell

```bash
# Hard clear zsh history, then kill current shell 
#rm -f ~/.zsh_history && kill -9 $$
rm -f ~/.zsh_history

# Clear Warp’s own local history database 
rm -f "$HOME/Library/Group Containers/2BBY89MBSN.dev.warp/Library/Application Support/dev.warp.Warp-Stable/warp.sqlite"
```

## install stuff using shell

- https://brew.sh/

## edit sh script

```bash
cd ~/Developer/init-macOS

chmod +x dir-list.sh
open dir-list.sh
```

## 2. remove placeholders & run the script

```bash
cd

# sudo find ~/Movies ~/Music ~/Pictures ~/Public ~/Downloads ~/Desktop ~/Documents \
sudo find ~/Movies ~/Music ~/Public ~/Downloads ~/Desktop ~/Documents \
  -mindepth 1 -print0 \
  | sudo xargs -0 rm -rf --

$HOME/Developer/init-macOS/dir-list.sh
```

## 3. check the output

```bash
cd ~/Desktop
ls -l
open .
```

## a few shortcuts

- https://arfanc.neocities.org/MAIN/iOS_Apps-3_dynamic-ext_json

```bash
"shortcut name"      : open_app_from_neocities

`1`     : Receive Text and Apps input from Nowhere
            If there's no input:
            Continue

`2`     : Get & Shortcut Input

`3`     : Open & Shortcut Input

`4`     : Stop and output App
            If there's nowhere to output:
            Do Nothing
```

## save current plists

```bash
# Create a folder on Desktop for output
mkdir -p ~/Desktop/macOS-Settings-Backup

defaults read -g > ~/Desktop/macOS-Settings-Backup/global-defaults.txt

# Save Dock settings
defaults read com.apple.dock > ~/Desktop/macOS-Settings-Backup/dock-settings.txt

# Save Safari settings
defaults read com.apple.Safari > ~/Desktop/macOS-Settings-Backup/safari-settings.txt

# Save Finder settings
defaults read com.apple.finder > ~/Desktop/macOS-Settings-Backup/finder-settings.txt

echo "🗂️ Settings backed up to ~/Desktop/macOS-Settings-Backup/"
```

| list all Preferences - `defaults domains | tr ',' '\n'`
| filter from all Preferences- `defaults domains | tr ',' '\n' | grep -i safari`

## remove all icons from dock

```bash
# remove dock icons
defaults write com.apple.dock persistent-apps -array
defaults write com.apple.dock persistent-others -array

# change dock settings
defaults write com.apple.dock orientation -string left
defaults write com.apple.dock autohide -bool true
defaults write com.apple.dock tilesize -integer 36
defaults write com.apple.dock magnification -bool true
defaults write com.apple.dock largesize -int 48
defaults write com.apple.dock show-recents -bool false

defaults write com.apple.dock persistent-apps -array-add '{tile-data={}; tile-type="spacer-tile";}'
defaults write com.apple.dock persistent-others -array-add '{tile-data={}; tile-type="spacer-tile";}'

# from macos-defaults.com
defaults write com.apple.dock "autohide-delay" -float "0"
defaults write com.apple.dock "autohide-time-modifier" -float "0"
defaults write com.apple.dock "scroll-to-open" -bool "true"

killall Dock
```

| delete a key in plist - `defaults delete -g AppleActionOnDoubleClick`
| view all settings - `defaults read -g`
| view dock settings - `defaults read com.apple.dock`

[https://macos-defaults.com/dock/scroll-to-open.html](https://macos-defaults.com/dock/scroll-to-open.html)

### open apps in order of final dock placement

```bash
# open /System/Applications/Clock.app
open -a 'Finder'
open -a 'Clock'
open -a 'Calendar'
open -a 'Safari'
open -a 'Terminal'
open -a 'iPhone Mirroring'
open -a 'Screen Sharing'
open -a 'System Settings'
open -a 'Github Desktop'
```

[https://www.bresink.com/osx/TinkerTool.html](https://www.bresink.com/osx/TinkerTool.html)

[https://www.bresink.com/osx/0TinkerTool/download.php](https://www.bresink.com/osx/0TinkerTool/download.php)

click Download
