# notes-3.2

## Article 1: first

- ***open***
    - terminal
        - 
    - Safari
        - login to Google
        - navigate to neocities
- ***Passwords***
    - import passwords from dropbox

## macOS settings-1
[notion link for this](https://www.notion.so/1393c6d24b7c8032a1e4fedeb1f88328?v=1393c6d24b7c809eaad4000c8e160a7b&p=1393c6d24b7c80698dc8f30fd6f01653&pm=c)

```bash
defaults write com.apple.finder AppleShowAllFiles -boolean false
defaults write NSGlobalDomain AppleShowAllExtensions -bool false
defaults write com.apple.finder _FXShowPosixPathInTitle -boolean false
defaults write com.apple.finder _FXSortFoldersFirst -boolean true
defaults write com.apple.finder ShowStatusBar -bool false
defaults write com.apple.finder ShowPathbar -bool true
defaults write com.apple.finder FXPreferredViewStyle -string "clmv"
defaults write com.apple.finder NewWindowTarget -string "PfDe"

defaults -currentHost write com.apple.screensaver idleTime -int 300
sudo pmset -b displaysleep 10
sudo pmset -c displaysleep 30

sudo pmset -a lowpowermode 2

defaults write com.apple.dock autohide -bool true
defaults write com.apple.dock tilesize -integer 48
defaults write com.apple.dock largesize -int 128
killall Dock

defaults write com.apple.dock scroll-to-open -bool true
defaults write com.apple.dock show-recents -bool false
defaults write NSGlobalDomain AppleActionOnDoubleClick -string "Minimize"
killall Dock

defaults write com.apple.universalaccess closeViewHotkeysEnabled -bool true
defaults write com.apple.universalaccess closeViewScrollWheelToggle -bool true
defaults write com.apple.universalaccess closeViewZoomMode -int 3
defaults write com.apple.universalaccess closeViewScrollWheelModifiersInt -int 4
```
---
```bash
#!/bin/bash

echo "Setting defaults..."

## Lock Screen settings
### Set screensaver to start after 300 seconds (5 minutes)
defaults -currentHost write com.apple.screensaver idleTime -int 300
### Set display sleep to 10 minutes when on battery
sudo pmset -b displaysleep 10
### Set display sleep to 30 minutes when plugged in
sudo pmset -c displaysleep 30

echo -e "\nReading current settings...\n"

echo "Save Panel Settings:"
defaults read NSGlobalDomain NSNavPanelExpandedStateForSaveMode
defaults read NSGlobalDomain NSNavPanelExpandedStateForSaveMode2

echo -e "\nKeyboard Settings:"
echo "Automatic Capitalization: $(defaults read NSGlobalDomain NSAutomaticCapitalizationEnabled)"
echo "Key Repeat Rate: $(defaults read NSGlobalDomain KeyRepeat)"
echo "Initial Key Repeat: $(defaults read NSGlobalDomain InitialKeyRepeat)"
echo "Keyboard UI Mode: $(defaults read NSGlobalDomain AppleKeyboardUIMode)"
echo "Function Keys State: $(defaults read NSGlobalDomain "com.apple.keyboard.fnState")"

echo -e "\nScrolling Settings:"
echo "Show Scrollbars: $(defaults read NSGlobalDomain AppleShowScrollBars)"
echo "Scroller Paging: $(defaults read NSGlobalDomain AppleScrollerPagingBehavior)"

echo -e "\nTrackpad Settings:"
echo "Secondary Click: $(defaults read NSGlobalDomain "com.apple.trackpad.enableSecondaryClick")"
echo "Tap Behavior: $(defaults read NSGlobalDomain "com.apple.mouse.tapBehavior")"

echo -e "\nNote: Some changes may require logging out and back in or restarting to take effect."
```
