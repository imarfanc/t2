# warp (set dock & some casks)

## step 1

`- un-hide dock & log out to activate previous settings`

```sh
defaults write com.apple.dock autohide -bool false
osascript -e 'tell application "System Events" to log out'
```

## step 2

`- install more apps & open apps in order`

`- group by modified in finder`

`- log into chrome & pass manager`

```sh
open -a Warp
open -a Google\ Chrome
open -a Helium
zed Developer/tmp1
mkdir -p Developer/tmp1/p1
touch Developer/tmp1/tmp1.md
touch Developer/tmp1/tmp2.md
touch Developer/tmp1/tmp.html
touch Developer/tmp1/p1/1.md
touch Developer/tmp1/p1/2.md
touch Developer/tmp1/p1/3.md
touch Developer/reset-macos/dir-list.sh

#
touch ~/Desktop/empty.md
touch ~/Developer/empty.md
touch ~/Documents/empty.md
touch ~/Downloads/empty.md
open Desktop
open Developer
open Documents
open Downloads

#
defaults write com.apple.dock persistent-apps -array-add \
  '{ "tile-type" = "spacer-tile"; }'
defaults write com.apple.dock persistent-apps -array-add \
  '{ "tile-type" = "small-spacer-tile"; }'
killall Dock

#
brew install lsd
brew install tree
brew install --cask cmux
brew install --cask ghostty
brew install --cask keyboard-maestro
brew install --cask font-hack-nerd-font
brew install --cask telegram-desktop@beta
brew install --cask whatsapp
brew install --cask setapp
brew install --cask zen
brew install --cask visual-studio-code
brew install --cask obsidian
brew install --cask github
```
